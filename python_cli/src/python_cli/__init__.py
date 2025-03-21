"""Entrypoint."""

from collections.abc import Callable
from typing import Any, Type, TypeVar, cast

import requests
from absl import app, flags
from langchain_core.language_models import BaseLanguageModel, LanguageModelInput
from langchain_core.messages import AIMessage
from langchain_core.runnables import Runnable
from langchain_google_genai import ChatGoogleGenerativeAI
from neo4j import Driver, GraphDatabase
from pydantic import BaseModel
from rich.pretty import pprint

from python_cli.entities import AudioBookMetadata, NewSubmissionList
from bs4 import BeautifulSoup

T = TypeVar("T", bound=BaseModel)

_BASE_URL = flags.DEFINE_string(
    name="base_url",
    help="The baseurl of audiobookbay.",
    default="https://audiobookbay.lu",
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
    return (
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 "
        "(KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36"
    )


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
    body = filtering_fn(http_response.text)
    llm_response = cast("dict", llm.invoke([*messages, ("human", body)]))
    parsed_response = llm_response["parsed"]
    return cast("T", parsed_response)


def main(argv: list[str]) -> None:
    """Main entrypoint."""
    del argv

    neo4j = get_driver()
    llm = get_base_llm()

    parse_new_entries_page_llm = llm.with_structured_output(
        NewSubmissionList, include_raw=True
    )

    parse_book_page_llm = llm.with_structured_output(
        AudioBookMetadata, include_raw=True
    )

    page_range = range(_PAGE_START.value, _PAGE_END.value)
    url_template = f"{_BASE_URL.value}/member/index?pid="

    for page in page_range:
        actual_url = f"{url_template}{page}"
        pprint(actual_url)
        submission_list: NewSubmissionList | None = retrieve_and_parse_page(
            url=actual_url,
            llm=parse_new_entries_page_llm,
            filtering_fn=extract_only_new_submissions_table,
        )
        if submission_list is None:
            raise RuntimeError()
        for submission in submission_list.submissions:
            book_page = f"{_BASE_URL.value}/{submission.url}"
            audiobook_metadata = retrieve_and_parse_page(
                url=book_page,
                llm=parse_book_page_llm,
                filtering_fn=extract_only_post_info,
            )
            if audiobook_metadata:
                pprint(audiobook_metadata.model_dump())
            break
        break


def _main() -> None:
    app.run(main)
