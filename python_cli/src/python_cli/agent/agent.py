"""Root agent for parsing."""

from typing import override

from google.adk.agents import BaseAgent
from google.adk.events.event import Event
from google.adk.runners import AsyncGenerator, InvocationContext
from injector import inject
from loguru import logger
from neo4j import Driver
from result import Err, Ok, Result
from tembo_pgmq_python.async_queue import PGMQueue

from python_cli.custom_types import (
    AudiobookBayURL,
    DescriptionForEmbeddingsAgent,
    ParseAudiobookPageAgent,
    ParseNewPublicationsPageAgent,
    QueueName,
    StrcturedResponseKey,
    VeryShortDescriptionAgent,
)
from python_cli.db.neo4j import does_audiobook_already_exists, store_audiobook_in_neo4j
from python_cli.entities import (
    AudioBookMetadata,
    AudioBookMetadataWithAugmentations,
    NewSubmissionList,
    QueueItem,
    QueueItemType,
)
from python_cli.utils.agent_state import add_content_to_agent_state
from python_cli.utils.html import (
    extract_only_new_submissions_table,
    extract_only_post_info,
)
from python_cli.utils.http import retrieve_and_clean_page
from python_cli.utils.urls import merge_url_parts


class ToplevelAgent(BaseAgent):
    """The top level parsing agent.

    It dispatches the work to sub agents based on whether we are analyzing the top level
    page (the one with the new entries) or the specific audiobook one.

    In the case the page is the new entries one, the subsequent pages are added to the
    pgmq queue and the agent exits.
    """

    @inject
    def __init__(  # noqa: PLR0913
        self,
        queue: PGMQueue,
        new_publicaton_page_agent: ParseNewPublicationsPageAgent,
        audiobook_page_agent: ParseAudiobookPageAgent,
        queue_name: QueueName,
        structured_response_key: StrcturedResponseKey,
        audiobookbay_url: AudiobookBayURL,
        driver: Driver,
        short_description_agent: VeryShortDescriptionAgent,
        embeddable_description_agent: DescriptionForEmbeddingsAgent,
    ) -> None:
        """Init."""
        super().__init__(name="top_level_agent")
        self._queue: PGMQueue = queue
        self._new_publication_page_agent: ParseNewPublicationsPageAgent = (
            new_publicaton_page_agent
        )
        self._audiobook_page_agent: ParseAudiobookPageAgent = audiobook_page_agent
        self._queue_name: QueueName = queue_name
        self._structured_response_key: StrcturedResponseKey = structured_response_key
        self._audiobookbay_url: AudiobookBayURL = audiobookbay_url
        self._driver: Driver = driver
        self._short_description_agent: VeryShortDescriptionAgent = (
            short_description_agent
        )
        self._embeddable_description_agent: DescriptionForEmbeddingsAgent = (
            embeddable_description_agent
        )

    async def _handle_new_entries_page(
        self,
        ctx: InvocationContext,
        queue_item: QueueItem,
    ) -> Result[None, str]:
        """Handles the page where the new audiobooks are displayed."""
        url = queue_item.url
        body = retrieve_and_clean_page(
            url, cleaning_fn=extract_only_new_submissions_table
        )
        _ = await add_content_to_agent_state(
            session_service=ctx.session_service,
            session=ctx.session,
            target_key="html",
            content=body,
        )
        async for event in self._new_publication_page_agent.run_async(ctx):
            if event.is_final_response() and event.content and event.content.parts:
                text_content = event.content.parts[0].text
                submission_list = NewSubmissionList.model_validate_json(text_content)
                for submission in submission_list.submissions:
                    final_url = merge_url_parts(self._audiobookbay_url, submission.url)
                    new_queue_item = QueueItem(
                        url=final_url,
                        queue_item_type=QueueItemType.PAGE_WITH_AUDIOBOOK_METADATA,
                    )
                    _ = await self._queue.send(
                        queue=self._queue_name,
                        message={"data": new_queue_item.model_dump_json()},
                    )

        return Ok(None)

    async def _handle_audiobook_page(
        self,
        ctx: InvocationContext,
        queue_item: QueueItem,
    ) -> Result[None, str]:
        """Handles the page where the new audiobooks are displayed."""
        url = queue_item.url
        body = retrieve_and_clean_page(url, cleaning_fn=extract_only_post_info)
        _ = await add_content_to_agent_state(
            session_service=ctx.session_service,
            session=ctx.session,
            target_key="html",
            content=body,
        )
        md: AudioBookMetadata | None = None
        async for event in self._audiobook_page_agent.run_async(ctx):
            if event.is_final_response() and event.content and event.content.parts:
                logger.info("Analyzing audiobook page.")
                text_content = event.content.parts[0].text
                md = AudioBookMetadata.model_validate_json(text_content)
        if not md:
            return Err("Couldn't get the metadata.")

        _ = await add_content_to_agent_state(
            session_service=ctx.session_service,
            session=ctx.session,
            target_key="description",
            content=md.description,
        )

        very_short_description: str | None = None
        async for event in self._short_description_agent.run_async(ctx):
            if event.is_final_response() and event.content and event.content.parts:
                very_short_description = event.content.parts[0].text
        logger.info("Very short description: {}", very_short_description)

        embeddable_description: str | None = None
        async for event in self._embeddable_description_agent.run_async(ctx):
            if event.is_final_response() and event.content and event.content.parts:
                embeddable_description = event.content.parts[0].text
        logger.info("Embeddable description: {}", embeddable_description)

        augmentd_md: AudioBookMetadataWithAugmentations = (
            AudioBookMetadataWithAugmentations.model_validate(
                {
                    **md.model_dump(),
                    "very_short_description": very_short_description,
                    "description_for_embeddings": embeddable_description,
                }
            )
        )
        store_audiobook_in_neo4j(self._driver, metadata=augmentd_md, path=url)
        return Ok(None)

    @override
    async def _run_async_impl(
        self, ctx: InvocationContext
    ) -> AsyncGenerator[Event, None]:
        # Check that there is a message.
        if (
            not ctx.user_content
            or not ctx.user_content.parts
            or not ctx.user_content.parts[0].text
        ):
            raise RuntimeError()
        raw_msg = ctx.user_content.parts[0].text
        queue_item = QueueItem.model_validate_json(raw_msg)
        match queue_item.queue_item_type:
            case QueueItemType.PAGE_WITH_NEW_ENTRIES:
                res = await self._handle_new_entries_page(ctx, queue_item)
                logger.info(res)
            case QueueItemType.PAGE_WITH_AUDIOBOOK_METADATA:
                if does_audiobook_already_exists(self._driver, queue_item.url):
                    logger.debug("Audiobook already ingested")
                else:
                    logger.info("New audiobook")
                    _ = await self._handle_audiobook_page(ctx, queue_item)
        yield Event(author=self.name)
