"""Entrypoint."""

from typing import cast

import requests
from absl import app, flags
from langchain_google_genai import ChatGoogleGenerativeAI
from neo4j import Driver, GraphDatabase

from python_cli.entities import AudioBookMetadata, NewSubmissionList
from rich.pretty import pprint

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


def main(argv: list[str]) -> None:
    """Main entrypoint."""
    del argv

    get_driver()
    llm = ChatGoogleGenerativeAI(
        model="gemini-2.0-flash-001",
        temperature=0,
        max_tokens=2_000_000,
        max_output_tokens=8192,  # type: ignore
    )

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
        get_response = requests.get(  # noqa: S113
            actual_url,
            allow_redirects=True,
            headers={"User-Agent": get_user_agent()},
        )
        body = get_response.text
        response: dict = cast(
            "dict", parse_new_entries_page_llm.invoke([*messages, ("human", body)])
        )

        submission_list: NewSubmissionList = response["parsed"]
        for submission in submission_list.submissions:
            book_page = f"{_BASE_URL.value}/{submission.url}"
            book_response = requests.get(
                book_page,
                allow_redirects=True,
                headers={"User-Agent": get_user_agent()},
            )
            book_metadata: dict = cast(
                "dict",
                parse_book_page_llm.invoke([*messages, ("human", book_response.text)]),
            )
            pprint(book_metadata["parsed"].model_dump())
            break
        break


def _main() -> None:
    app.run(main)
