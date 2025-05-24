"""Main module."""

import asyncio
from pathlib import Path
from textwrap import dedent

from absl import app, flags
from injector import Binder, Injector, SingletonScope
from loguru import logger
from sqlalchemy import text
from tembo_pgmq_python.async_queue import PGMQueue

from python_cli.agent.agent_di_module import AgentDIModule
from python_cli.configuration import ConfigurationModel, ConfigurationModule
from python_cli.custom_types import (
    AgentName,
    AppName,
    ConfigurationPath,
    GeminiModelVersion,
    ParadeEngine,
)
from python_cli.db.db_di_module import DBModule

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


async def _main_impl() -> None:
    injector = Injector(
        [_bind_flags, ConfigurationModule(), DBModule(), AgentDIModule()]
    )
    config: ConfigurationModel = injector.get(ConfigurationModel)
    logger.info("{config}", config=config.model_dump_json(indent=2))
    parade_engine = injector.get(ParadeEngine)
    async with parade_engine.begin() as conn:
        res = await conn.execute(text("SELECT 1"))
        logger.info(res.all())

    pgmq = injector.get(PGMQueue)
    await pgmq.init()

    await pgmq.create_queue("test_queue")
    logger.info("{}", await pgmq.list_queues())


def main(argv: list[str]) -> None:
    """Main entrypoint."""
    del argv
    asyncio.run(_main_impl())


def _main() -> None:
    app.run(main)


if __name__ == "__main__":
    _main()
