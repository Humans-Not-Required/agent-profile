"""
Agent Profile Python SDK

Canonical identity pages for AI agents.

Quick start::

    from agent_profile import AgentProfileClient

    with AgentProfileClient("https://yourserver.example.com") as client:
        # Register a new profile
        reg = client.register("my-agent")
        api_key = reg["api_key"]

        # Fill it out
        client.update_profile("my-agent", api_key,
            display_name="My Agent",
            tagline="Autonomous AI agent",
            bio="I ship code while humans sleep.",
            theme="midnight",
        )

        # Add links and skills
        client.add_link("my-agent", api_key,
            url="https://github.com/my-agent",
            label="GitHub",
            platform="github",
        )
        client.add_skill("my-agent", api_key, "Rust")
        client.add_skill("my-agent", api_key, "Python")

        # Check completeness
        score = client.get_score("my-agent")
        print(f"Profile: {score['score']}/100 complete")
        for step in score["next_steps"]:
            print(f"  → {step}")
"""

from .client import AgentProfileClient, DEFAULT_BASE_URL
from .exceptions import (
    AgentProfileError,
    ConflictError,
    NotFoundError,
    RateLimitError,
    ServerError,
    UnauthorizedError,
    ValidationError,
)

__all__ = [
    "AgentProfileClient",
    "DEFAULT_BASE_URL",
    "AgentProfileError",
    "ConflictError",
    "NotFoundError",
    "RateLimitError",
    "ServerError",
    "UnauthorizedError",
    "ValidationError",
]

__version__ = "0.1.0"
