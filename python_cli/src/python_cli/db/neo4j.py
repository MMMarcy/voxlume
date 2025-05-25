"""Contains the neo4j stuff."""

from textwrap import dedent

import neo4j
from loguru import logger

from python_cli.entities import AudioBookMetadataWithAugmentations

_PATH_PROPERTY_NAME = "path"


def _save_author_and_get_id(
    tx: neo4j.Transaction, author_name: str, audiobook_id: str
) -> str:
    query_main = """
        MATCH (ab:Audiobook) WHERE elementId(ab) = $audiobook_id
        MERGE (author:Author {name: $author_name})
        MERGE (ab)-[:WRITTEN_BY]->(author)
        RETURN elementId(author) AS author_id
        """
    result = tx.run(query_main, author_name=author_name, audiobook_id=audiobook_id)

    # Get the internal Neo4j ID of the created audiobook for linking the series
    record = result.single()
    if not record:
        raise RuntimeError("Failed to create Audiobook node or retrieve its ID.")  # noqa: TRY003
    author_id = record["author_id"]  # Keep author_id in case needed elsewhere
    logger.info(
        "Created/Merged Author '{author_name}' with audiobook '{audiobook_id}'",
        author_name=author_name,
        audiobook_id=audiobook_id,
    )
    return str(author_id)


def _create_audiobook_and_relations_tx(
    tx: neo4j.Transaction, metadata: AudioBookMetadataWithAugmentations, path: str
) -> None:
    """Insert the audiobook metadata.

    Neo4j transaction function to create audiobook, author, relationships,
    and optionally series information.
    Designed to be used with session.execute_write().
    """
    audiobook_props = metadata.model_dump(
        exclude={
            "authors",
            "is_part_of_series",
            "series",
            "series_volume",
            "read_by",
        }
    )
    # Adds the path to the properties of the audiobook
    audiobook_props[_PATH_PROPERTY_NAME] = path

    # Add series_volume to props only if it's relevant (part of a series)
    # This ensures it's set on the Audiobook node itself
    if metadata.is_part_of_series and metadata.series and metadata.series_volume:
        audiobook_props["series_volume"] = metadata.series_volume
    else:
        # Ensure it's explicitly null if not part of series or volume is None
        audiobook_props["series_volume"] = None

    logger.debug(
        "Audiobook properties for Cypher: {audiobook_props}",
        audiobook_props=audiobook_props,
    )
    query_create_audiobook = """
    MERGE (ab:Audiobook {path: $path})
    ON CREATE SET ab = $audiobook_props, ab.last_upload = timestamp()
    ON MATCH SET ab.last_upload = timestamp()
    RETURN elementId(ab) AS audiobook_id
    """
    result = tx.run(
        query_create_audiobook,
        audiobook_props=audiobook_props,
        path=path,
    )
    record = result.single()
    if not record:
        raise RuntimeError()
    audiobook_id = record["audiobook_id"]

    query_attach_reader = """
    MATCH (ab:Audiobook) WHERE elementId(ab) = $audiobook_id
    MERGE (reader:Reader {name: $reader_name})
    MERGE (ab)-[:READ_BY]->(reader)
    """
    for reader in metadata.read_by:
        _ = tx.run(query_attach_reader, audiobook_id=audiobook_id, reader_name=reader)

    query_attach_category = """
    MATCH (ab:Audiobook) WHERE elementId(ab) = $audiobook_id
    MERGE (category:Category {value: $category_name})
    MERGE (ab)-[:CATEGORIZED_AS]->(category)
    """
    for cat in metadata.categories:
        _ = tx.run(query_attach_category, audiobook_id=audiobook_id, category_name=cat)

    query_attach_keyword = """
    MATCH (ab:Audiobook) WHERE elementId(ab) = $audiobook_id
    MERGE (kw:Keyword {value: $keyword})
    MERGE (ab)-[:HAS_KEYWORD]->(kw)
    """
    for kw in metadata.keywords:
        _ = tx.run(query_attach_keyword, audiobook_id=audiobook_id, keyword=kw)

    author_ids = [
        _save_author_and_get_id(tx, name, audiobook_id) for name in metadata.authors
    ]

    # 5. Check if series information is present
    if metadata.is_part_of_series and metadata.series:
        logger.info("Handling series: {series}", series=metadata.series)

        merge_series = """
        // Get the audiobook
        MATCH (ab:Audiobook) WHERE elementId(ab) = $audiobook_id

        // Save the series if it's not there.
        MERGE (series:Series {title: $series_title})

        // Merge relationships Series <-> Audiobook
        MERGE (ab)-[:PART_OF_SERIES]->(series)

        RETURN elementId(series) AS series_id
        """
        result = tx.run(
            merge_series, audiobook_id=audiobook_id, series_title=metadata.series
        )
        record = result.single()
        if not record:
            raise RuntimeError()
        series_id = record["series_id"]
        query_series = """
        MATCH (author:Author) WHERE elementId(author) = $author_id
        MATCH (series:Series) WHERE elementId(series) = $series_id

        // Link author with series
        MERGE (series)-[:WRITTEN_BY_SERIES]->(author)
        """
        for author_id in author_ids:
            _ = tx.run(
                query_series,
                author_id=author_id,
                series_id=series_id,
            )
        logger.info(
            "Linked Audiobook ID {audiobook_id} to Series '{series}'",
            audiobook_id=audiobook_id,
            series=metadata.series,
        )


def store_audiobook_in_neo4j(
    driver: neo4j.Driver, metadata: AudioBookMetadataWithAugmentations, path: str
) -> None:
    """Stores audiobook metadata in Neo4j using a managed transaction.

    Args:
        driver: An initialized neo4j.Driver instance.
        metadata: An AudioBookMetadata object containing the data.
        path: The path used by audiobookbay.
    """
    try:
        with driver.session() as session:
            session.execute_write(
                _create_audiobook_and_relations_tx,
                metadata=metadata,
                path=path,
            )
        logger.info(
            "Successfully stored audiobook '{desc}...' by {author}",
            desc=metadata.description[:30],
            author=metadata.authors,
        )
    except neo4j.exceptions.ServiceUnavailable as e:
        logger.error(f"Neo4j connection error: {e}")
        # Handle connection issues (e.g., retry, raise specific exception)
        raise
    except Exception as e:
        logger.exception(
            f"Failed to store audiobook metadata for author {metadata.authors}: {e}"
        )
        # Handle other potential errors during the transaction
        raise  # Re-raise the exception if calling code needs to know


def does_audiobook_already_exists(driver: neo4j.Neo4jDriver, path: str) -> bool:
    """Check if an audiobook has already been ingested."""
    res, _, _ = driver.execute_query(
        query_=dedent("""
        RETURN EXISTS {
            MATCH (:Audiobook {path: $path})
        } AS pathExists"""),
        path=path,
    )
    return res[0]["pathExists"]
