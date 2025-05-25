"""Utilities for working with the ADK agents' state."""

import time
from enum import Enum

from google.adk.events.event import EventActions
from google.adk.runners import BaseSessionService, Event, Session
from result import Ok, Result


class AppendOrReplace(Enum):
    """Whether to append or replace the content for the given key."""

    append = 0
    replace = 1


async def add_content_to_agent_state(
    session_service: BaseSessionService,
    session: Session,
    target_key: str,
    content: str,
    append_or_replace: AppendOrReplace = AppendOrReplace.replace,
) -> Result[Event, str]:
    """Function to add to the state's given key some content."""
    if append_or_replace == AppendOrReplace.append:
        raise NotImplementedError()

    state_changes = session.state | {target_key: content}

    # --- Create Event with Actions ---
    actions_with_update = EventActions(state_delta=state_changes)
    # This event might represent an internal system action, not just an agent response
    system_event = Event(
        invocation_id="add_content_to_context",
        author="system",  # Or 'agent', 'tool' etc.
        actions=actions_with_update,
        timestamp=time.time(),
    )

    # --- Append the Event (This updates the state) ---
    return Ok(await session_service.append_event(session, system_event))
