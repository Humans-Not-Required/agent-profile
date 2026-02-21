"""Custom exceptions for the Agent Profile SDK."""


class AgentProfileError(Exception):
    """Base exception for all Agent Profile errors."""

    def __init__(self, message: str, status_code: int | None = None, body: dict | None = None):
        super().__init__(message)
        self.status_code = status_code
        self.body = body or {}

    def __str__(self) -> str:
        if self.status_code:
            return f"[{self.status_code}] {super().__str__()}"
        return super().__str__()


class NotFoundError(AgentProfileError):
    """Profile or resource not found (404)."""


class UnauthorizedError(AgentProfileError):
    """Invalid or missing API key (401)."""


class ConflictError(AgentProfileError):
    """Username already taken (409)."""


class ValidationError(AgentProfileError):
    """Invalid request data (422)."""


class RateLimitError(AgentProfileError):
    """Rate limit exceeded (429). Slow down and retry."""


class ServerError(AgentProfileError):
    """Unexpected server error (5xx)."""
