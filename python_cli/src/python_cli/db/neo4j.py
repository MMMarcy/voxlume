"""Contains the neo4j stuff."""

import neo4j
from loguru import logger

from python_cli.entities import AudioBookMetadata

_PATH_PROPERTY_NAME = "path"


def _create_audiobook_and_relations_tx(
    tx: neo4j.Transaction, metadata: AudioBookMetadata, path: str
) -> None:
    """Insert the audiobook metadata.

    Neo4j transaction function to create audiobook, author, relationships,
    and optionally series information.
    Designed to be used with session.execute_write().
    """
    log = logger

    audiobook_props = metadata.model_dump(
        exclude={"author", "is_part_of_series", "series", "series_volume", "read_by"}
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

    log.debug(
        "Audiobook properties for Cypher: {audiobook_props}",
        audiobook_props=audiobook_props,
    )

    # 1. MERGE Author node (creates if not exists, matches if exists)
    # 2. CREATE Audiobook node (always create a new one)
    # 3. Set Audiobook properties
    # 4. MERGE relationships between Author and Audiobook
    query_main = """
    MERGE (author:Author {name: $author_name})
    MERGE (reader:Reader {name: $reader_name})
    CREATE (ab:Audiobook)
    SET ab = $audiobook_props
    MERGE (ab)-[:WRITTEN_BY]->(author)
    MERGE (ab)-[:READ_BY]->(reader)
    RETURN elementId(ab) AS audiobook_id, elementId(author) AS author_id
    """
    result = tx.run(
        query_main,
        author_name=metadata.author,
        audiobook_props=audiobook_props,
        reader_name=metadata.read_by,
    )

    # Get the internal Neo4j ID of the created audiobook for linking the series
    record = result.single()
    if not record:
        raise RuntimeError("Failed to create Audiobook node or retrieve its ID.")  # noqa: TRY003
    audiobook_id = record["audiobook_id"]
    author_id = record["author_id"]  # Keep author_id in case needed elsewhere
    log.info(
        "Created/Merged Author ID: {author_id}, Created Audiobook ID: {audiobook_id}",
        author_id=author_id,
        audiobook_id=audiobook_id,
    )

    # 5. Check if series information is present
    if metadata.is_part_of_series and metadata.series:
        log.info("Handling series: {series}", series=metadata.series)
        # 6. MERGE Series node (unique by title *for this author*)
        # 7. MERGE relationships between Author and Series
        # 8. MERGE relationships between Series and the newly created Audiobook
        # We MATCH the author and audiobook using their known identifiers/properties
        # to ensure we link the correct nodes. Using the internal ID is safest.
        query_series = """
        MATCH (author:Author) WHERE id(author) = $author_id
        MATCH (ab:Audiobook) WHERE id(ab) = $audiobook_id

        // Merge the series node - only based on title for simplicity,
        // but linking it to the author ensures context if needed later.
        MERGE (series:S
        eries {title: $series_title})

        // Merge relationships Author <-> Series
        MERGE (author)-[:AUTHORED_SERIES]->(series)
        MERGE (series)-[:WRITTEN_BY_SERIES]->(author)

        // Merge relationships Series <-> Audiobook
        MERGE (series)-[:COMPOSED_BY]->(ab)
        MERGE (ab)-[:PART_OF_SERIES]->(series)

        // Note: series_volume was already set on the Audiobook node in the first query
        """
        tx.run(
            query_series,
            author_id=author_id,
            audiobook_id=audiobook_id,
            series_title=metadata.series,
        )
        log.info(
            "Linked Audiobook ID {audiobook_id} to Series '{series}'",
            audiobook_id=audiobook_id,
            series=metadata.series,
        )


def store_audiobook_in_neo4j(
    driver: neo4j.Driver, metadata: AudioBookMetadata, path: str
) -> None:
    """Stores audiobook metadata in Neo4j using a managed transaction.

    Args:
        driver: An initialized neo4j.Driver instance.
        metadata: An AudioBookMetadata object containing the data.
    """
    try:
        with driver.session() as session:
            session.execute_write(
                _create_audiobook_and_relations_tx, metadata=metadata, path=path
            )  # type: ignore
        logger.info(
            "Successfully stored audiobook '{desc}...' by {author}",
            desc=metadata.description[:30],
            author=metadata.author,
        )
    except neo4j.exceptions.ServiceUnavailable as e:
        logger.error(f"Neo4j connection error: {e}")
        # Handle connection issues (e.g., retry, raise specific exception)
        raise
    except Exception as e:
        logger.exception(
            f"Failed to store audiobook metadata for author {metadata.author}: {e}"
        )
        # Handle other potential errors during the transaction
        raise  # Re-raise the exception if calling code needs to know
