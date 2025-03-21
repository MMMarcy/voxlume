"""Entities used by the CLIs."""

from textwrap import dedent

from pydantic import BaseModel, Field


class NewSubmission(BaseModel):
    """A single submission."""

    submission_date: str = Field(
        description="The date in which the submission was done.",
    )
    title: str = Field(
        description="The title of the book WITHOUT the author name included."
    )
    author: str = Field(description="The author of the book.")
    url: str = Field(description="The url or partial url of this book.")


class NewSubmissionList(BaseModel):
    """List of submissions."""

    submissions: list[NewSubmission] = Field(
        description=dedent("""
        The list of submissions that can be found in the HTML table of the
        latest books table.
        """).strip()
    )


class AudioBookMetadata(BaseModel):
    """The metadata about the audiobook."""

    categories: list[str] = Field(
        description="The categories this book has been categorized into"
    )

    language: str = Field(description="The language of the audiobook.")

    keywords: list[str] = Field(description="The list of keywords for this audiobook.")

    cover_url: str = Field(description="The url of the cover image.")

    author: str = Field(description="The name of author of this audiobook.")

    read_by: str = Field(
        description="The name of the voice actor that read this audiobook."
    )

    format: str = Field(
        description="The format used to saved the audio file associated with this book."
    )

    bitrate: str | None = Field(
        description="The bitrate used to save the audio file for this audiobook if present.",
        default=None,
    )

    unabriged: bool = Field(
        description="Whether or not this book is unabriged.", default=False
    )

    description: str = Field(description="The description of this audiobook.")

    runtime: str | None = Field(
        description="The runtime lenght of the audiobook if present.", default=None
    )
