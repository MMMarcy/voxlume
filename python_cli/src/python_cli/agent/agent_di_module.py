"""Submodule for the Dependency Injection wiring of Agent."""

from textwrap import dedent
from typing import override

from google.adk.agents import BaseAgent, LlmAgent
from google.adk.artifacts.base_artifact_service import BaseArtifactService
from google.adk.runners import InMemoryArtifactService, InMemorySessionService, Runner
from google.adk.sessions.base_session_service import BaseSessionService
from google.genai.types import Content, Part
from injector import Binder, Module, SingletonScope, provider, singleton

from python_cli.agent.agent import ToplevelAgent
from python_cli.custom_types import (
    AgentName,
    AppName,
    GeminiModelVersion,
    ParseAudiobookPageAgent,
    ParseNewPublicationsPageAgent,
)
from python_cli.entities import AudioBookMetadata, NewSubmissionList


class AgentDIModule(Module):
    """Actual wiring."""

    @provider
    @singleton
    def _provide_llm_for_parse_new_publications_page(
        self, model_version: GeminiModelVersion
    ) -> ParseNewPublicationsPageAgent:
        return ParseNewPublicationsPageAgent(
            LlmAgent(
                name="parse-publication-page-agent",
                model=model_version,
                instruction=dedent("""
                You extract information about new audiobook releases
                from the provided HTML."""),
                output_schema=NewSubmissionList,
            )
        )

    @provider
    @singleton
    def _provide_llm_for_parse_audiobook_page(
        self, model_version: GeminiModelVersion
    ) -> ParseAudiobookPageAgent:
        return ParseAudiobookPageAgent(
            LlmAgent(
                name="parse-audiobook-page-agent",
                model=model_version,
                instruction=dedent("""
                You extract information about new audiobook releases
                from the provided HTML."""),
                output_schema=AudioBookMetadata,
            )
        )

    @provider
    @singleton
    def _provide_adk_runner(
        self,
        agent: BaseAgent,
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
    def _provide_agent(
        self,
        agent_name: AgentName,
        parse_audiobook_page_agent: ParseAudiobookPageAgent,
        parse_new_publications_page_agent: ParseNewPublicationsPageAgent,
    ) -> BaseAgent:
        return ToplevelAgent(
            name=agent_name,
        )

    @provider
    @singleton
    def _provde_artifact_service(self) -> BaseArtifactService:
        return InMemoryArtifactService()

    @override
    def configure(self, binder: Binder) -> None:
        """Define simple bindings."""
        binder.bind(
            GeminiModelVersion,
            to=GeminiModelVersion("gemini-2.5-flash-preview-04-17"),
            scope=SingletonScope,
        )
