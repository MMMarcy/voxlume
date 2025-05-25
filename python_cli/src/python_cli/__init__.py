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
