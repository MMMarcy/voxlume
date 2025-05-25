"""Submodule for the Dependency Injection wiring of Agent."""

from textwrap import dedent
from typing import override

from google.adk.agents import LlmAgent
from google.adk.artifacts.base_artifact_service import BaseArtifactService
from google.adk.runners import InMemoryArtifactService, InMemorySessionService, Runner
from google.adk.sessions.base_session_service import BaseSessionService
from google.adk.sessions.session import Session
from injector import Binder, Module, SingletonScope, provider, singleton
from loguru import logger

from python_cli.agent.agent import ToplevelAgent
from python_cli.custom_types import (
    AppName,
    CreateSessionFn,
    GeminiModelVersion,
    ParseAudiobookPageAgent,
    ParseNewPublicationsPageAgent,
    StrcturedResponseKey,
)
from python_cli.entities import AudioBookMetadata, NewSubmissionList


class AgentDIModule(Module):
    """Actual wiring."""

    @provider
    @singleton
    def _provide_llm_for_parse_new_publications_page(
        self,
        model_version: GeminiModelVersion,
        structured_response_key: StrcturedResponseKey,
    ) -> ParseNewPublicationsPageAgent:
        return ParseNewPublicationsPageAgent(
            LlmAgent(
                name="parse_publication_page_agent",
                model=model_version,
                disallow_transfer_to_parent=True,
                disallow_transfer_to_peers=True,
                instruction=dedent("""
                You extract information about new audiobook releases
                from the provided HTML:

                {html}
                """),
                output_schema=NewSubmissionList,
                output_key=structured_response_key,
            )
        )

    @provider
    @singleton
    def _provide_llm_for_parse_audiobook_page(
        self,
        model_version: GeminiModelVersion,
        structured_response_key: StrcturedResponseKey,
    ) -> ParseAudiobookPageAgent:
        return ParseAudiobookPageAgent(
            LlmAgent(
                name="parse_audiobook_page_agent",
                model=model_version,
                disallow_transfer_to_parent=True,
                disallow_transfer_to_peers=True,
                instruction=dedent("""
                You extract information about new audiobook releases
                from the provided HTML:

                {html}
                """),
                output_schema=AudioBookMetadata,
                output_key=structured_response_key,
            )
        )

    @provider
    @singleton
    def _provide_adk_runner(
        self,
        agent: ToplevelAgent,
        app_name: AppName,
        session_service: BaseSessionService,
        artifact_service: BaseArtifactService,
    ) -> Runner:
        return Runner(
            agent=agent,
            app_name=app_name,
            session_service=session_service,
            artifact_service=artifact_service,
        )

    @provider
    @singleton
    def _provide_session_service(self) -> BaseSessionService:
        return InMemorySessionService()

    @provider
    @singleton
    def _provde_artifact_service(self) -> BaseArtifactService:
        return InMemoryArtifactService()

    @provider
    def _provide_create_session_fn(
        self,
        session_service: BaseSessionService,
    ) -> CreateSessionFn:
        """Get's or create a session."""

        async def _helper(
            user_id: str,
            session_id: str,
            app_name: str,
        ) -> Session:
            maybe_session = await session_service.get_session(
                app_name=app_name, user_id=user_id, session_id=session_id
            )
            if maybe_session:
                logger.info("returning existing session.")
                return maybe_session
            logger.info("Creating new session.")
            return await session_service.create_session(
                app_name=app_name,
                user_id=user_id,
                session_id=session_id,
                state={},
            )

        return _helper

    @override
    def configure(self, binder: Binder) -> None:
        """Define simple bindings."""
        binder.bind(
            StrcturedResponseKey,
            to=StrcturedResponseKey("structured_response"),
            scope=SingletonScope,
        )
