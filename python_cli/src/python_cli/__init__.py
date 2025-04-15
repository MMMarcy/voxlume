"""Entrypoint."""

import time
from collections.abc import Callable
from queue import LifoQueue
from typing import TypeVar, cast

import neo4j
import requests
from absl import app, flags
from bs4 import BeautifulSoup
from langchain_core.language_models import BaseLanguageModel, LanguageModelInput
from langchain_core.messages import AIMessage
from langchain_core.runnables import Runnable
from langchain_google_genai import ChatGoogleGenerativeAI
from loguru import logger
from neo4j import Driver, GraphDatabase
from pydantic import BaseModel
from rich.pretty import pprint
from random_user_agent.user_agent import UserAgent

from python_cli.entities import (
    AudioBookMetadata,
    NewSubmissionList,
    QueueItem,
    QueueItemType,
)

T = TypeVar("T", bound=BaseModel)

_BASE_URL = flags.DEFINE_string(
    name="base_url",
    help="The baseurl of audiobookbay.",
    default="http://audiobookbay.is/",
)

_DB_USERNAME = flags.DEFINE_string(
    name="neo4j_user",
    help="The username for neo4j.",
    default="neo4j",
)

_DB_PASSWORD = flags.DEFINE_string(
    name="neo4j_password",
    help="Password to use for neo4j.",
    default="password",
)

_DB_URI = flags.DEFINE_string(
    name="neo4j_uri", help="URI of the neo4j instance", default="neo4j://localhost:7687"
)

_PAGE_START = flags.DEFINE_integer(
    name="page_start",
    help="The start page to getting the audiobooks from. Inclusive.",
    default=1,
)

_PAGE_END = flags.DEFINE_integer(
    name="page_end",
    help="The end page to getting the audiobooks from. Exclusive.",
    default=2,
)

messages = [
    ("system", """You are a person in charge of extracting informations from HTML.""")
]

user_agent_rotator = UserAgent(limit=100)


def get_driver() -> Driver:
    """Returns a neo4j driver.

    Returns:
        The driver for connecting to the neo4j instance.

    Raises:
        ServiceUnavailable if the connection is not successful.
    """
    driver = GraphDatabase.driver(
        _DB_URI.value, auth=(_DB_USERNAME.value, _DB_PASSWORD.value)
    )
    driver.verify_connectivity()
    return driver


def get_user_agent() -> str:
    """Returns the user agent."""
    user_agent = user_agent_rotator.get_random_user_agent()
    logger.info("Using user agent {}", user_agent)
    return user_agent


def get_base_llm(
    model_id: str = "gemini-2.0-flash-001",
    temperature: int = 0,
    max_tokens: int = 2_000_000,
    max_output_tokens: int = 8192,
) -> BaseLanguageModel:
    """Creates a base LLM."""
    return ChatGoogleGenerativeAI(
        model=model_id,
        temperature=temperature,
        max_tokens=max_tokens,
        max_output_tokens=max_output_tokens,  # type: ignore
    )


def extract_only_post_info(body: str) -> str:
    """Returns only audiobook info."""
    soup = BeautifulSoup(body, features="html.parser")
    return str(soup.select_one(".post"))


def extract_only_new_submissions_table(body: str) -> str:
    """Returns only the HTML of the table containing the new audiobooks."""
    soup = BeautifulSoup(body, features="html.parser")
    return str(soup.select_one(".main_table"))


def retrieve_and_parse_page(
    url: str,
    llm: Runnable[
        LanguageModelInput, dict[str, AIMessage | type[T] | None] | BaseModel
    ],
    filtering_fn: Callable[[str], str] = str,
) -> T | None:
    """Sends a GET requests and then uses the runnable to get the structured data."""
    http_response = requests.get(  # noqa: S113
        url, allow_redirects=True, headers={"User-Agent": get_user_agent()}
    )

    logger.info("Gotten page {}, with status_code {}", url, http_response.status_code)
    body = filtering_fn(http_response.text)
    llm_response = cast("dict", llm.invoke([*messages, ("human", body)]))
    parsed_response = llm_response["parsed"]
    if parsed_response:
        return cast("T", parsed_response)
    return None


def _create_audiobook_and_relations_tx(
    tx: neo4j.Transaction, metadata: AudioBookMetadata
) -> None:
    """Insert the audiobook metadata.

    Neo4j transaction function to create audiobook, author, relationships,
    and optionally series information.
    Designed to be used with session.execute_write().
    """
    log = logger

    # Prepare properties for the Audiobook node, excluding relational info
    # Use model_dump for Pydantic v2+ or .dict() for v1
    try:
        audiobook_props = metadata.model_dump(
            exclude={"author", "is_part_of_series", "series", "series_volume"}
        )
    except AttributeError:  # Fallback for Pydantic v1
        audiobook_props = metadata.dict(
            exclude={"author", "is_part_of_series", "series", "series_volume"}
        )

    # Add series_volume to props only if it's relevant (part of a series)
    # This ensures it's set on the Audiobook node itself
    if metadata.is_part_of_series and metadata.series and metadata.series_volume:
        audiobook_props["series_volume"] = metadata.series_volume
    else:
        # Ensure it's explicitly null if not part of series or volume is None
        audiobook_props["series_volume"] = None

    log.debug(
        "Audiobook properties for Cypher: {audiobook_props}",
        audiobook_props=audiobook_props,
    )

    # 1. MERGE Author node (creates if not exists, matches if exists)
    # 2. CREATE Audiobook node (always create a new one)
    # 3. Set Audiobook properties
    # 4. MERGE relationships between Author and Audiobook
    query_main = """
    MERGE (author:Author {name: $author_name})
    CREATE (ab:Audiobook)
    SET ab = $audiobook_props
    MERGE (author)-[:HAS_AUTHORED]->(ab)
    MERGE (ab)-[:WRITTEN_BY]->(author)
    RETURN id(ab) AS audiobook_id, id(author) AS author_id
    """
    result = tx.run(
        query_main, author_name=metadata.author, audiobook_props=audiobook_props
    )

    # Get the internal Neo4j ID of the created audiobook for linking the series
    record = result.single()
    if not record:
        raise RuntimeError("Failed to create Audiobook node or retrieve its ID.")  # noqa: TRY003
    audiobook_id = record["audiobook_id"]
    author_id = record["author_id"]  # Keep author_id in case needed elsewhere
    log.info(
        "Created/Merged Author ID: {author_id}, Created Audiobook ID: {audiobook_id}",
        author_id=author_id,
        audiobook_id=audiobook_id,
    )

    # 5. Check if series information is present
    if metadata.is_part_of_series and metadata.series:
        log.info("Handling series: {series}", series=metadata.series)
        # 6. MERGE Series node (unique by title *for this author*)
        # 7. MERGE relationships between Author and Series
        # 8. MERGE relationships between Series and the newly created Audiobook
        # We MATCH the author and audiobook using their known identifiers/properties
        # to ensure we link the correct nodes. Using the internal ID is safest.
        query_series = """
        MATCH (author:Author) WHERE id(author) = $author_id
        MATCH (ab:Audiobook) WHERE id(ab) = $audiobook_id

        // Merge the series node - only based on title for simplicity,
        // but linking it to the author ensures context if needed later.
        MERGE (series:Series {title: $series_title})

        // Merge relationships Author <-> Series
        MERGE (author)-[:AUTHORED_SERIES]->(series)
        MERGE (series)-[:WRITTEN_BY_SERIES]->(author)

        // Merge relationships Series <-> Audiobook
        MERGE (series)-[:COMPOSED_BY]->(ab)
        MERGE (ab)-[:PART_OF_SERIES]->(series)

        // Note: series_volume was already set on the Audiobook node in the first query
        """
        tx.run(
            query_series,
            author_id=author_id,
            audiobook_id=audiobook_id,
            series_title=metadata.series,
        )
        log.info(
            "Linked Audiobook ID {audiobook_id} to Series '{series}'",
            audiobook_id=audiobook_id,
            series=metadata.series,
        )


def store_audiobook_in_neo4j(driver: neo4j.Driver, metadata: AudioBookMetadata) -> None:
    """Stores audiobook metadata in Neo4j using a managed transaction.

    Args:
        driver: An initialized neo4j.Driver instance.
        metadata: An AudioBookMetadata object containing the data.
    """
    try:
        with driver.session() as session:
            session.execute_write(_create_audiobook_and_relations_tx, metadata=metadata)  # type: ignore
        logger.info(
            "Successfully stored audiobook '{desc}...' by {author}",
            desc=metadata.description[:30],
            author=metadata.author,
        )
    except neo4j.exceptions.ServiceUnavailable as e:
        logger.error(f"Neo4j connection error: {e}")
        # Handle connection issues (e.g., retry, raise specific exception)
        raise
    except Exception as e:
        logger.exception(
            f"Failed to store audiobook metadata for author {metadata.author}: {e}"
        )
        # Handle other potential errors during the transaction
        raise  # Re-raise the exception if calling code needs to know


def main(argv: list[str]) -> None:
    """Main entrypoint."""
    del argv

    queue: LifoQueue[QueueItem] = LifoQueue()
    neo4j = get_driver()
    llm = get_base_llm()
    parse_new_entries_page_llm = llm.with_structured_output(
        NewSubmissionList, include_raw=True
    )
    parse_book_page_llm = llm.with_structured_output(
        AudioBookMetadata, include_raw=True
    )

    url_template = f"{_BASE_URL.value}/member/index?pid="

    for page in range(_PAGE_END.value, _PAGE_START.value - 1, -1):
        actual_url = f"{url_template}{page}"
        queue.put(
            QueueItem(
                queue_item_type=QueueItemType.PAGE_WITH_NEW_ENTRIES, url=actual_url
            )
        )
    logger.info("Finished adding main pages to queue.")

    while not queue.empty():
        logger.info(f"Getting item from queue. Queue size: {queue.qsize()}")
        queue_item = queue.get()
        match queue_item.queue_item_type:
            case QueueItemType.PAGE_WITH_NEW_ENTRIES:
                logger.info(f"Parsing page with new entries. URL = {queue_item.url}")
                submission_list: NewSubmissionList | None = retrieve_and_parse_page(
                    url=queue_item.url,
                    llm=parse_new_entries_page_llm,
                    filtering_fn=extract_only_new_submissions_table,
                )
                if submission_list is None:
                    raise RuntimeError()
                for submission in submission_list.submissions:
                    queue.put(
                        QueueItem(
                            queue_item_type=QueueItemType.PAGE_WITH_AUDIOBOOK_METADATA,
                            url=f"{_BASE_URL.value}/{submission.url}",
                        )
                    )
            case QueueItemType.PAGE_WITH_AUDIOBOOK_METADATA:
                logger.info(
                    f"Parsing page with audiobook metadata. URL = {queue_item.url}"
                )
                audiobook_metadata = retrieve_and_parse_page(
                    url=queue_item.url,
                    llm=parse_book_page_llm,
                    filtering_fn=extract_only_post_info,
                )
                if audiobook_metadata:
                    store_audiobook_in_neo4j(driver=neo4j, metadata=audiobook_metadata)
                else:
                    logger.warning("Parsing of audiobook data failed.")
        time.sleep(30)

    while not queue.empty():
        pprint(queue.get())


def _main() -> None:
    app.run(main)
