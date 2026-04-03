from __future__ import annotations

from typing import Any, Literal

from pydantic import BaseModel, Field


class WorkerRequest(BaseModel):
    request_id: str
    worker_type: str
    payload: dict[str, Any] = Field(default_factory=dict)
    timeout_ms: int = 30000
    worker_version: str = "0.1.0"


class WorkerResponse(BaseModel):
    request_id: str
    status: Literal["ok", "error"]
    result: dict[str, Any] = Field(default_factory=dict)
    error_code: str | None = None
    error_detail: str | None = None
