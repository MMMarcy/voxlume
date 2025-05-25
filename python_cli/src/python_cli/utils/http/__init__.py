"""Utilities for HTTP requests."""

from collections.abc import Callable
from typing import cast

import requests
from loguru import logger
from random_user_agent.user_agent import UserAgent

user_agent_rotator = UserAgent(limit=100)


def get_user_agent() -> str:
    """Returns the user agent."""
    user_agent: str = cast("str", user_agent_rotator.get_random_user_agent())
    logger.trace("Using user agent {}", user_agent)
    return user_agent


def retrieve_and_clean_page(url: str, cleaning_fn: Callable[[str], str] = str) -> str:
    """Sends a GET requests and then uses the runnable to get the structured data."""
    http_response = requests.get(  # noqa: S113
        url, allow_redirects=True, headers={"User-Agent": get_user_agent()}
    )

    logger.trace("Gotten page {}, with status_code {}", url, http_response.status_code)
    return cleaning_fn(http_response.text)
