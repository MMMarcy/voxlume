"""Root agent for parsing."""

from typing import override

from google.adk.agents import BaseAgent
from google.adk.events.event import Event
from google.adk.runners import AsyncGenerator, InvocationContext


class ToplevelAgent(BaseAgent):
    """The top level parsing agent.

    It dispatches the work to sub agents based on whether we are analyzing the top level
    page (the one with the new entries) or the specific audiobook one.

    In the case the page is the new entries one, the subsequent pages are added to the
    pgmq queue and the agent exits.
    """

    def __init__(self) -> None:
        """Init."""
        super().__init__(name="top_level_agent")

    @override
    async def _run_async_impl(
        self, ctx: InvocationContext
    ) -> AsyncGenerator[Event, None]:
        async for event in super()._run_async_impl(ctx):
            yield event
