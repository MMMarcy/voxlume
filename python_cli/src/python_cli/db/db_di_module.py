"""Dependency injection module for DB."""

from injector import Module, multiprovider, provider, singleton
from neo4j import Driver, GraphDatabase
from sqlalchemy.ext.asyncio import create_async_engine
from tembo_pgmq_python.async_queue import PGMQueue

from python_cli.configuration import (
    Neo4JConfiguriation,
    ParadeDBConfiguration,
    PGMQConfiguration,
)
from python_cli.custom_types import (
    Neo4JMigrationQuery,
    ParadeEngine,
    ShouldRunNeo4JMigration,
)


class DBModule(Module):
    """Module for the DB."""

    @multiprovider
    @singleton
    def _provide_migration_queries(self) -> list[Neo4JMigrationQuery]:
        return []

    @provider
    @singleton
    def _provide_neo4j_driver(
        self,
        neo4j_config: Neo4JConfiguriation,
        migration_queries: list[Neo4JMigrationQuery],
    ) -> Driver:
        driver = GraphDatabase.driver(
            uri=f"neo4j://{neo4j_config.neo4j_host}:{neo4j_config.neo4j_port}",
            auth=(neo4j_config.neo4j_username, neo4j_config.neo4j_password),
        )
        driver.verify_connectivity()
        for migration_query in migration_queries:
            _ = driver.execute_query(query_=migration_query)
        return driver

    @provider
    @singleton
    def _provide_paradedb_driver(
        self, parade_config: ParadeDBConfiguration
    ) -> ParadeEngine:
        url = "postgresql+psycopg://{user}:{password}@{host}:{port}/{dbname}".format(  # noqa: UP032
            user=parade_config.parade_username,
            password=parade_config.parade_password,
            host=parade_config.parade_host,
            port=parade_config.parade_port,
            dbname=parade_config.parade_database,
        )
        engine = create_async_engine(url=url)
        return ParadeEngine(engine)

    @provider
    @singleton
    def _provide_pgmq_queue(self, pgmq_config: PGMQConfiguration) -> PGMQueue:
        return PGMQueue(
            host=pgmq_config.pgmq_host,
            port=str(pgmq_config.pgmq_port),
            username=pgmq_config.pgmq_username,
            password=pgmq_config.pgmq_password,
            database=pgmq_config.pgmq_database,
        )
