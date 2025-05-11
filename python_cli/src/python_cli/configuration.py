"""Module containing the configuration for the CLI."""

from pydantic import BaseModel, Field
from enum import Enum, StrEnum
from injector import Module, provider, singleton

from python_cli.custom_types import ConfigurationPath
import json5
import json


class ShouldOverrideExistingBook(StrEnum):
    """Whether the system should override the an audiobook already in the DB."""

    YES = "YES"
    NO = "NO"


class ShouldStopOnFindingAnExistingBOok(Enum):
    """Whether or not the binary should when a book exists already."""

    YES = "YES"
    NO = "NO"


class ParadeDBConfiguration(BaseModel):
    """Configuration for paradeDB."""


class Neo4JConfiguriation(BaseModel):
    """Configuration for Neo4J."""

    neo4j_username: str = Field(default="neo4j")
    neo4j_password: str = Field(default="password")
    neo4j_host: str = Field(default="neo4j://localhost")
    neo4j_port: int = Field(default=7687)


class PGMQConfiguration(BaseModel):
    """Confiugration for PGMQ."""

    pgmq_host: str = Field(default="localhost")
    pgmq_port: int = Field(default=5432)
    pgmq_username: str = Field(default="postgres")
    pgmq_password: str = Field(default="postgres")
    pgmq_database: str = Field(default="postgres")


class RuntimeConfiguration(BaseModel):
    """Configuration for runtime parameters."""

    page_start: int = Field(default=1)
    page_end: int = Field(default=2)
    queue_name: str = Field(default="work_queue")
    wait_time_between_scraping_requests: int = Field(default=30)
    should_override_existing_book: ShouldOverrideExistingBook = Field(
        default=ShouldOverrideExistingBook.NO
    )
    should_stop_on_existing_book: ShouldStopOnFindingAnExistingBOok = Field(
        default=ShouldStopOnFindingAnExistingBOok.YES
    )
    base_url: str = Field(default="http://audiobookbay.is/")


class ConfigurationModel(BaseModel):
    """Root configuration model."""

    parade_db_configuration: ParadeDBConfiguration = Field(
        default=ParadeDBConfiguration()
    )
    neo4j_configuration: Neo4JConfiguriation = Field()
    pgmq_configuration: PGMQConfiguration = Field()
    runtime_configuration: RuntimeConfiguration = Field()


class ConfigurationModule(Module):
    """Configuration module for DI."""

    @provider
    @singleton
    def _provide_configuration_module(
        self,
        config_path: ConfigurationPath,
    ) -> ConfigurationModel:
        with config_path.open("r") as f:
            if config_path.name.lower().endswith(".json5"):
                value = json5.loads(f.read())
                return ConfigurationModel.model_validate(value)
            return ConfigurationModel.model_validate_json(f.read())
