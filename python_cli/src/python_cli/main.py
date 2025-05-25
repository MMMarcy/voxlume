"""Main module."""

import asyncio
from pathlib import Path
import sys
from textwrap import dedent
from typing import Generic, cast

from absl import app, flags
from google.adk.runners import BaseSessionService, Runner, Session
from google.genai import types
from injector import Binder, Injector, SingletonScope
from loguru import logger
from sqlalchemy import text
from tembo_pgmq_python.async_queue import PGMQueue

from python_cli.agent.agent import ToplevelAgent
from python_cli.agent.agent_di_module import AgentDIModule
from python_cli.configuration import ConfigurationModel, ConfigurationModule
from python_cli.custom_types import (
    AgentName,
    AppName,
    ConfigurationPath,
    CreateSessionFn,
    GeminiModelVersion,
    IsBackfillJob,
    ParadeEngine,
    QueueName,
)
from python_cli.db.db_di_module import DBModule
from python_cli.entities import QueueItem, QueueItemType

logger.disable("google.adk.models")

_CONFIGURATION_PATH = flags.DEFINE_string(
    name="configuration_path",
    help="The path of the configuration.",
    default="configuration.json5",
)
_GEMINI_MODEL_VERSION = flags.DEFINE_string(
    name="gemini_model_version",
    help="The gemini model version to use",
    default="gemini-2.5-flash-preview-04-17",
)
_AGENT_NAME = flags.DEFINE_string(
    name="top_level_agent_name",
    help="The top level agent name to use",
    default="audiobook_parsing_agent",
)
_APP_NAME = flags.DEFINE_string(
    name="parse_audiobooks",
    help="The name of this particular app.",
    default="audiobooks_app",
)
_QUEUE_NAME = flags.DEFINE_string(
    name="pgmq_queue_name",
    help="The queue to use within pgmq.",
    default="default_queue",
)
_IS_BACKFILL_JOB = flags.DEFINE_enum(
    name="backfill_job",
    help="Whether or not this is a backfill job.",
    default=IsBackfillJob.NO,
    enum_values=list(IsBackfillJob),
)
_BASE_URL = flags.DEFINE_string(
    name="base_url",
    help="Audiobookbay base url",
    default="https://audiobookbay.is/",
)


def _bind_flags(binder: Binder) -> None:
    configuration_path = Path(_CONFIGURATION_PATH.value)
    if not configuration_path.exists():
        raise ValueError(
            dedent(f"""
            Configuration path provided (
            {configuration_path.absolute()!s})
            does not exists.""")
        )
    binder.bind(
        ConfigurationPath,
        to=ConfigurationPath(configuration_path),
        scope=SingletonScope,
    )
    binder.bind(
        GeminiModelVersion,
        to=GeminiModelVersion(_GEMINI_MODEL_VERSION.value),
        scope=SingletonScope,
    )
    binder.bind(AppName, to=AppName(_APP_NAME.value), scope=SingletonScope)
    binder.bind(AgentName, to=AgentName(_AGENT_NAME.value), scope=SingletonScope)
    binder.bind(QueueName, to=QueueName(_QUEUE_NAME.value), scope=SingletonScope)


async def _main_impl() -> None:
    injector = Injector(
        [
            # Flags passed as command line arguments
            _bind_flags,
            # Configuration bindings.
            ConfigurationModule(),
            # DB bindings
            DBModule(),
            # Agent(s) bindings
            AgentDIModule(),
        ]
    )
    queue_name = injector.get(QueueName)

    pgmq = injector.get(PGMQueue)
    await pgmq.init()
    await pgmq.drop_queue(queue_name)
    await pgmq.create_queue(queue_name)
    runner = injector.get(Runner)
    app_name = injector.get(AppName)
    create_session_fn: CreateSessionFn = cast(
        "CreateSessionFn", injector.get(CreateSessionFn)
    )
    url = f"{_BASE_URL.value}/member/index?pid=1"
    session: Session = await create_session_fn("loop", "test_session", app_name)
    item = QueueItem(queue_item_type=QueueItemType.PAGE_WITH_NEW_ENTRIES, url=url)
    _ = await pgmq.send(queue_name, message={"data": item.model_dump_json()})

    while msg := await pgmq.pop(queue_name):
        if msg is None:
            break
        content = types.Content(
            role="user", parts=[types.Part(text=msg.message["data"])]
        )
        async for event in runner.run_async(
            user_id=session.user_id, session_id=session.id, new_message=content
        ):
            logger.info("{} - {}", event.author, event.is_final_response())


def main(argv: list[str]) -> None:
    """Main entrypoint."""
    del argv
    asyncio.run(_main_impl())


def _main() -> None:
    app.run(main)


if __name__ == "__main__":
    _main()
