"""Module containing the custom type definition.

These definitions will be used for *typed* dependency injection.
"""

from enum import Enum
from pathlib import Path
from typing import LiteralString, NewType


class _YesNoEnum(Enum):
    NO = 0
    YES = 1


ConfigurationPath = NewType("ConfigurationPath", Path)

# Databases section.
Neo4JMigrationQuery = NewType("Neo4JMigrationQuery", LiteralString)
ShouldRunNeo4JMigration = NewType("ShouldRunNeo4JMigration", _YesNoEnum)


# Agent section
GeminiModelVersion = NewType("GeminiModelVersion", Li

