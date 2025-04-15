"""Entrypoint."""

import time
from collections.abc import Callable
from queue import LifoQueue
from typing import TypeVar, cast

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
from random_user_agent.user_agent import UserAgent
from rich.pretty import pprint

from python_cli.db.neo4j import store_audiobook_in_neo4j
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
    driver.execute_query(
        query_=(
            "CREATE CONSTRAINT constraint_path_is_unique IF NOT EXISTS "
            "FOR (_ab:Audiobook) REQUIRE _ab.path IS UNIQUE"
        )
    )
    return driver


def get_user_agent() -> str:
    """Returns the user agent."""
    user_agent: str = user_agent_rotator.get_random_user_agent()
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


def merge_url_parts(part1: str, part2: str) -> str:
    """Merges two parts of a URL path, ensuring exactly one slash between them.

    Handles cases where:
    - part1 ends with '/' and part2 starts with '/' (removes duplicate)
    - part1 ends with '/' and part2 doesn't start with '/' (keeps single slash)
    - part1 doesn't end with '/' and part2 starts with '/' (keeps single slash)
    - part1 doesn't end with '/' and part2 doesn't start with '/' (adds single slash)
    - Either part is empty.

    Args:
      part1: The first part of the URL (e.g., 'http://example.com/api', 'path/to').
      part2: The second part of the URL (e.g., 'users', '/data', 'resource/id').

    Returns:
      The merged URL string with a single slash separator.
    """
    # Handle empty parts gracefully
    if not part1:
        # If part1 is empty, just return part2, ensuring it doesn't start with '//'
        # (though lstrip below handles this mostly)
        return (
            part2.lstrip("/") if part2 == "/" else part2
        )  # Avoid returning "" if part2 is "/"
    if not part2:
        # If part2 is empty, just return part1, ensuring it doesn't end with '//'
        # (though rstrip below handles this mostly)
        return part1

    # Strip trailing slash from part1 and leading slash from part2
    cleaned_part1 = part1.rstrip("/")
    cleaned_part2 = part2.lstrip("/")

    # Join them with a single slash
    return f"{cleaned_part1}/{cleaned_part2}"


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
                            url=submission.url,
                        )
                    )
            case QueueItemType.PAGE_WITH_AUDIOBOOK_METADATA:
                complete_url = merge_url_parts(_BASE_URL.value, queue_item.url)
                logger.info(
                    f"Parsing page with audiobook metadata. PATH = {complete_url}"
                )
                audiobook_metadata = retrieve_and_parse_page(
                    url=complete_url,
                    llm=parse_book_page_llm,
                    filtering_fn=extract_only_post_info,
                )
                if audiobook_metadata:
                    store_audiobook_in_neo4j(
                        driver=neo4j, metadata=audiobook_metadata, path=queue_item.url
                    )
                else:
                    logger.warning("Parsing of audiobook data failed.")
        time.sleep(30)

    while not queue.empty():
        pprint(queue.get())


def _main() -> None:
    app.run(main)


if __name__ == "__main__":
    _main()
