"""Entrypoint."""

from loguru import logger

logger.disable("google_adk.google_llm")


# def handler(
#     message: dict[str, Any],
#     redis: Redis,
#     topic_name: str,
#     parse_new_entries_page_llm: Any,
#     parse_book_page_llm: Any,
#     neo4j: Driver,
# ):
#     logger.info("Getting item pubsub topic. Data: {data}", data=message["data"])
#     queue_item = QueueItem.model_validate_json(message["data"])
#     match queue_item.queue_item_type:
#         case QueueItemType.PAGE_WITH_NEW_ENTRIES:
#             logger.info(f"Parsing page with new entries. URL = {queue_item.url}")
#             submission_list: NewSubmissionList | None = retrieve_and_parse_page(
#                 url=queue_item.url,
#                 llm=parse_new_entries_page_llm,
#                 filtering_fn=extract_only_new_submissions_table,
#             )
#             if submission_list is None:
#                 raise RuntimeError()
#             for submission in submission_list.submissions:
#                 redis.publish(
#                     topic_name,
#                     QueueItem(
#                         queue_item_type=QueueItemType.PAGE_WITH_AUDIOBOOK_METADATA,
#                         url=submission.url,
#                     ).model_dump_json(),
#                 )
#         case QueueItemType.PAGE_WITH_AUDIOBOOK_METADATA:
#             complete_url = merge_url_parts(_BASE_URL.value, queue_item.url)
#             logger.info(f"Parsing page with audiobook metadata. PATH = {complete_url}")
#             audiobook_metadata = retrieve_and_parse_page(
#                 url=complete_url,
#                 llm=parse_book_page_llm,
#                 filtering_fn=extract_only_post_info,
#             )
#             if audiobook_metadata:
#                 store_audiobook_in_neo4j(
#                     driver=neo4j, metadata=audiobook_metadata, path=queue_item.url
#                 )
#             else:
#                 logger.warning("Parsing of audiobook data failed.")
#     time.sleep(30)


# def main(argv: list[str]) -> None:
#     """Main entrypoint."""
#     del argv
#     injector = Injector([_bind_flags, ConfigurationModule])
#     config: ConfigurationModel = injector.get(ConfigurationModel)
#     logger.info("{config}", config=config.model_dump_json(indent=2))
#
# neo4j = get_driver()
# redis = Redis(
#     host=_REDIS_HOST.value, port=_REDIS_PORT.value, password=_REDIS_PASSOWRD.value
# )
# p = redis.pubsub()
# llm = get_base_llm()
# parse_new_entries_page_llm = llm.with_structured_output(
#     NewSubmissionList, include_raw=True
# )
#
# parse_book_page_llm = llm.with_structured_output(
#     AudioBookMetadata, include_raw=True
# )
# _handler = partial(
#     handler,
#     redis=redis,
#     topic_name="queue-channel",
#     parse_new_entries_page_llm=parse_new_entries_page_llm,
#     parse_book_page_llm=parse_book_page_llm,
#     neo4j=neo4j,
# )
# p.subscribe(**{"queue-channel": _handler})
#
# url_template = f"{_BASE_URL.value}/member/index?pid="
#
# for page in range(_PAGE_END.value, _PAGE_START.value - 1, -1):
#     actual_url = f"{url_template}{page}"
#     redis.publish(
#         "queue-channel",
#         QueueItem(
#             queue_item_type=QueueItemType.PAGE_WITH_NEW_ENTRIES, url=actual_url
#         ).model_dump_json(),
#     )
# logger.info("Finished adding main pages to queue.")
# while True:
#     p.get_message()
#     time.sleep(30)
