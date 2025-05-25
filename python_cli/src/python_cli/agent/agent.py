"""Root agent for parsing."""

from typing import override

from google.adk.agents import BaseAgent
from google.adk.events.event import Event
from google.adk.runners import AsyncGenerator, InvocationContext
from injector import inject
from loguru import logger
from result import Ok, Result
from tembo_pgmq_python.async_queue import PGMQueue

from python_cli.custom_types import (
    ParseAudiobookPageAgent,
    ParseNewPublicationsPageAgent,
    QueueName,
    StrcturedResponseKey,
)
from python_cli.entities import NewSubmissionList, QueueItem, QueueItemType
from python_cli.utils.agent_state import add_content_to_agent_state
from python_cli.utils.html import extract_only_new_submissions_table
from python_cli.utils.http import retrieve_and_clean_page


class ToplevelAgent(BaseAgent):
    """The top level parsing agent.

    It dispatches the work to sub agents based on whether we are analyzing the top level
    page (the one with the new entries) or the specific audiobook one.

    In the case the page is the new entries one, the subsequent pages are added to the
    pgmq queue and the agent exits.
    """

    @inject
    def __init__(
        self,
        queue: PGMQueue,
        new_publicaton_page_agent: ParseNewPublicationsPageAgent,
        audiobook_page_agent: ParseAudiobookPageAgent,
        queue_name: QueueName,
        structured_response_key: StrcturedResponseKey,
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

    async def _handle_new_entries_page(
        self,
        ctx: InvocationContext,
        queue_item: QueueItem,
    ) -> Result[None, str]:
        url = queue_item.url
        logger.debug("Fetching {}", url)
        body = retrieve_and_clean_page(
            url, cleaning_fn=extract_only_new_submissions_table
        )
        await add_content_to_agent_state(
            session_service=ctx.session_service,
            session=ctx.session,
            target_key="html",
            content=body,
        )
        async for event in self._new_publication_page_agent.run_async(ctx):
            if event.is_final_response() and event.content and event.content.parts:
                text_content = event.content.parts[0]
                logger.info(
                    "Text content: {}",
                    NewSubmissionList.model_validate_json(
                        text_content.text
                    ).model_dump_json(indent=2),
                )

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
                pass
        yield Event(author=self.name)
