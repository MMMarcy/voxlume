"""Module containing the custom type definition.

These definitions will be used for *typed* dependency injection.
"""

from collections.abc import Callable, Coroutine
from enum import Enum, StrEnum
from pathlib import Path
from typing import Any, LiteralString, NewType

from google.adk.agents import BaseAgent
from google.adk.sessions.session import Session
from sqlalchemy.ext.asyncio import AsyncEngine


class _YesNoEnum(Enum):
    NO = 0
    YES = 1


class IsBackfillJob(StrEnum):
    """Whether or not this is a backfill job."""

    YES = "yes"
    NO = "no"


ConfigurationPath = NewType("ConfigurationPath", Path)
AudiobookBayURL = NewType("AudiobookBayURL", str)

# Databases section.
Neo4JMigrationQuery = NewType("Neo4JMigrationQuery", LiteralString)
ShouldRunNeo4JMigration = NewType("ShouldRunNeo4JMigration", _YesNoEnum)
ParadeEngine = NewType("ParadeEngine", AsyncEngine)  # type: ignore
QueueName = NewType("QueueName", str)

# Agent section
AgentName = NewType("AgentName", str)
AppName = NewType("AppName", str)
GeminiModelVersion = NewType("GeminiModelVersion", str)
ParseAudiobookPageAgent = NewType("ParseAudiobookPageAgent", BaseAgent)  # type: ignore
ParseNewPublicationsPageAgent = NewType("ParseNewPublicationsPageAgent", BaseAgent)  # type: ignore
VeryShortDescriptionAgent = NewType("VeryShortDescriptionAgent", BaseAgent)  # type: ignore
DescriptionForEmbeddingsAgent = NewType("DescriptionForEmbeddingsAgent", BaseAgent)  # type: ignore
StrcturedResponseKey = NewType("StrcturedResponseKey", str)
CreateSessionFn = Callable[[str, str, str], Coroutine[None, None, Session]]
