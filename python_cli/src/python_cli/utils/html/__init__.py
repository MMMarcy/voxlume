"""Utilities for working with the HTML."""

from bs4 import BeautifulSoup


def extract_only_post_info(body: str) -> str:
    """Returns only audiobook info."""
    soup = BeautifulSoup(body, features="html.parser")
    return str(soup.select_one(".post"))


def extract_only_new_submissions_table(body: str) -> str:
    """Returns only the HTML of the table containing the new audiobooks."""
    soup = BeautifulSoup(body, features="html.parser")
    return str(soup.select_one(".main_table"))
