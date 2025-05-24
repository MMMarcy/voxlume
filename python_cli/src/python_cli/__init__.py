"""Entrypoint."""
#
# from pathlib import Path
# import time
# from collections.abc import Callable
# from functools import partial
# from typing import Any, TypeVar, cast
#
# from injector import Binder, Injector, SingletonScope
# import requests
# from absl import app, flags
# from bs4 import BeautifulSoup
# from langchain_core.language_models import (
#     BaseLanguageModel,
#     LanguageModelInput,
# )
# from langchain_core.messages import AIMessage
# from langchain_core.runnables import Runnable
# from langchain_google_genai import ChatGoogleGenerativeAI
# from loguru import logger
# from neo4j import Driver, GraphDatabase
# from pydantic import BaseModel
# from random_user_agent.user_agent import UserAgent
# from rich.pretty import pprint
#
# from python_cli.configuration import ConfigurationModel, ConfigurationModule
# from python_cli.custom_types import ConfigurationPath
# from python_cli.db.neo4j import store_audiobook_in_neo4j
# from python_cli.entities import (
#     AudioBookMetadata,
#     NewSubmissionList,
#     QueueItem,
#     QueueItemType,
# )
#
# T = TypeVar("T", bound=BaseModel)
#
#
# messages = [
#     ("system", """You are a person in charge of extracting informations from HTML.""")
# ]
#
# user_agent_rotator = UserAgent(limit=100)
#
#
# def get_user_agent() -> str:
#     """Returns the user agent."""
#     user_agent: str = user_agent_rotator.get_random_user_agent()
#     logger.info("Using user agent {}", user_agent)
#     return user_agent
#
#
# def get_base_llm(
#     model_id: str = "gemini-2.0-flash-001",
#     temperature: int = 0,
#     max_tokens: int = 2_000_000,
#     max_output_tokens: int = 8192,
# ) -> BaseLanguageModel:
#     """Creates a base LLM."""
#     return ChatGoogleGenerativeAI(
#         model=model_id,
#         temperature=temperature,
#         max_tokens=max_tokens,
#         max_output_tokens=max_output_tokens,  # type: ignore
#     )
#
#
# def extract_only_post_info(body: str) -> str:
#     """Returns only audiobook info."""
#     soup = BeautifulSoup(body, features="html.parser")
#     return str(soup.select_one(".post"))
#
#
# def extract_only_new_submissions_table(body: str) -> str:
#     """Returns only the HTML of the table containing the new audiobooks."""
#     soup = BeautifulSoup(body, features="html.parser")
#     return str(soup.select_one(".main_table"))
#
#
# def retrieve_and_parse_page(
#     url: str,
#     llm: Runnable[
#         LanguageModelInput, dict[str, AIMessage | type[T] | None] | BaseModel
#     ],
#     filtering_fn: Callable[[str], str] = str,
# ) -> T | None:
#     """Sends a GET requests and then uses the runnable to get the structured data."""
#     http_response = requests.get(  # noqa: S113
#         url, allow_redirects=True, headers={"User-Agent": get_user_agent()}
#     )
#
#     logger.info("Gotten page {}, with status_code {}", url, http_response.status_code)
#     body = filtering_fn(http_response.text)
#     llm_response = cast("dict", llm.invoke([*messages, ("human", body)]))
#     parsed_response = llm_response["parsed"]
#     if parsed_response:
#         return cast("T", parsed_response)
#     return None
#
#
# def merge_url_parts(part1: str, part2: str) -> str:
#     """Merges two parts of a URL path, ensuring exactly one slash between them.
#
#     Handles cases where:
#     - part1 ends with '/' and part2 starts with '/' (removes duplicate)
#     - part1 ends with '/' and part2 doesn't start with '/' (keeps single slash)
#     - part1 doesn't end with '/' and part2 starts with '/' (keeps single slash)
#     - part1 doesn't end with '/' and part2 doesn't start with '/' (adds single slash)
#     - Either part is empty.
#
#     Args:
#       part1: The first part of the URL (e.g., 'http://example.com/api', 'path/to').
#       part2: The second part of the URL (e.g., 'users', '/data', 'resource/id').
#
#     Returns:
#       The merged URL string with a single slash separator.
#     """
#     # Handle empty parts gracefully
#     if not part1:
#         # If part1 is empty, just return part2, ensuring it doesn't start with '//'
#         # (though lstrip below handles this mostly)
#         return (
#             part2.lstrip("/") if part2 == "/" else part2
#         )  # Avoid returning "" if part2 is "/"
#     if not part2:
#         # If part2 is empty, just return part1, ensuring it doesn't end with '//'
#         # (though rstrip below handles this mostly)
#         return part1
#
#     # Strip trailing slash from part1 and leading slash from part2
#     cleaned_part1 = part1.rstrip("/")
#     cleaned_part2 = part2.lstrip("/")
#
#     # Join them with a single slash
#     return f"{cleaned_part1}/{cleaned_part2}"
#
#
# # def handler(
# #     message: dict[str, Any],
# #     redis: Redis,
# #     topic_name: str,
# #     parse_new_entries_page_llm: Any,
# #     parse_book_page_llm: Any,
# #     neo4j: Driver,
# # ):
# #     logger.info("Getting item pubsub topic. Data: {data}", data=message["data"])
# #     queue_item = QueueItem.model_validate_json(message["data"])
# #     match queue_item.queue_item_type:
# #         case QueueItemType.PAGE_WITH_NEW_ENTRIES:
# #             logger.info(f"Parsing page with new entries. URL = {queue_item.url}")
# #             submission_list: NewSubmissionList | None = retrieve_and_parse_page(
# #                 url=queue_item.url,
# #                 llm=parse_new_entries_page_llm,
# #                 filtering_fn=extract_only_new_submissions_table,
# #             )
# #             if submission_list is None:
# #                 raise RuntimeError()
# #             for submission in submission_list.submissions:
# #                 redis.publish(
# #                     topic_name,
# #                     QueueItem(
# #                         queue_item_type=QueueItemType.PAGE_WITH_AUDIOBOOK_METADATA,
# #                         url=submission.url,
# #                     ).model_dump_json(),
# #                 )
# #         case QueueItemType.PAGE_WITH_AUDIOBOOK_METADATA:
# #             complete_url = merge_url_parts(_BASE_URL.value, queue_item.url)
# #             logger.info(f"Parsing page with audiobook metadata. PATH = {complete_url}")
# #             audiobook_metadata = retrieve_and_parse_page(
# #                 url=complete_url,
# #                 llm=parse_book_page_llm,
# #                 filtering_fn=extract_only_post_info,
# #             )
# #             if audiobook_metadata:
# #                 store_audiobook_in_neo4j(
# #                     driver=neo4j, metadata=audiobook_metadata, path=queue_item.url
# #                 )
# #             else:
# #                 logger.warning("Parsing of audiobook data failed.")
# #     time.sleep(30)
#
#
# def main(argv: list[str]) -> None:
#     """Main entrypoint."""
#     del argv
#     injector = Injector([_bind_flags, ConfigurationModule])
#     config: ConfigurationModel = injector.get(ConfigurationModel)
#     logger.info("{config}", config=config.model_dump_json(indent=2))
#
#     # neo4j = get_driver()
#     # redis = Redis(
#     #     host=_REDIS_HOST.value, port=_REDIS_PORT.value, password=_REDIS_PASSOWRD.value
#     # )
#     # p = redis.pubsub()
#     # llm = get_base_llm()
#     # parse_new_entries_page_llm = llm.with_structured_output(
#     #     NewSubmissionList, include_raw=True
#     # )
#     #
#     # parse_book_page_llm = llm.with_structured_output(
#     #     AudioBookMetadata, include_raw=True
#     # )
#     # _handler = partial(
#     #     handler,
#     #     redis=redis,
#     #     topic_name="queue-channel",
#     #     parse_new_entries_page_llm=parse_new_entries_page_llm,
#     #     parse_book_page_llm=parse_book_page_llm,
#     #     neo4j=neo4j,
#     # )
#     # p.subscribe(**{"queue-channel": _handler})
#     #
#     # url_template = f"{_BASE_URL.value}/member/index?pid="
#     #
#     # for page in range(_PAGE_END.value, _PAGE_START.value - 1, -1):
#     #     actual_url = f"{url_template}{page}"
#     #     redis.publish(
#     #         "queue-channel",
#     #         QueueItem(
#     #             queue_item_type=QueueItemType.PAGE_WITH_NEW_ENTRIES, url=actual_url
#     #         ).model_dump_json(),
#     #     )
#     # logger.info("Finished adding main pages to queue.")
#     # while True:
#     #     p.get_message()
#     #     time.sleep(30)
