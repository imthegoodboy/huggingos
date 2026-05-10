from __future__ import annotations

import uuid
from dataclasses import dataclass, field
from datetime import UTC, datetime
from enum import StrEnum
from typing import Any, Callable


def utc_now() -> str:
    return datetime.now(UTC).isoformat().replace("+00:00", "Z")


class RiskLevel(StrEnum):
    READ = "read"
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"


class PolicyDecision(StrEnum):
    ALLOW = "allow"
    DENY = "deny"
    CONFIRM = "confirm"
    DRY_RUN_ONLY = "dry_run_only"


class ActionStatus(StrEnum):
    SUCCEEDED = "succeeded"
    FAILED = "failed"
    DENIED = "denied"
    CONFIRMATION_REQUIRED = "confirmation_required"
    DRY_RUN = "dry_run"


class CapabilityError(Exception):
    """Raised when a capability cannot complete safely."""


@dataclass(frozen=True)
class Verification:
    ok: bool
    message: str
    data: dict[str, Any] = field(default_factory=dict)

    def to_dict(self) -> dict[str, Any]:
        return {"ok": self.ok, "message": self.message, "data": self.data}


@dataclass(frozen=True)
class CapabilityMetadata:
    name: str
    version: str
    owner: str
    description: str
    risk: RiskLevel
    permissions: tuple[str, ...] = ()
    input_schema: dict[str, Any] = field(default_factory=dict)
    result_schema: dict[str, Any] = field(default_factory=dict)
    reversible: bool = False

    def to_dict(self) -> dict[str, Any]:
        return {
            "name": self.name,
            "version": self.version,
            "owner": self.owner,
            "description": self.description,
            "risk": self.risk.value,
            "permissions": list(self.permissions),
            "input_schema": self.input_schema,
            "result_schema": self.result_schema,
            "reversible": self.reversible,
        }


@dataclass
class ActionRequest:
    capability: str
    params: dict[str, Any]
    actor: str = "user"
    reason: str = ""
    dry_run: bool = False
    confirmed: bool = False
    action_id: str = field(default_factory=lambda: str(uuid.uuid4()))
    requested_at: str = field(default_factory=utc_now)

    def to_dict(self) -> dict[str, Any]:
        return {
            "action_id": self.action_id,
            "capability": self.capability,
            "params": self.params,
            "actor": self.actor,
            "reason": self.reason,
            "dry_run": self.dry_run,
            "confirmed": self.confirmed,
            "requested_at": self.requested_at,
        }


@dataclass
class ActionResult:
    action_id: str
    capability: str
    status: ActionStatus
    summary: str
    data: dict[str, Any] = field(default_factory=dict)
    error: str | None = None
    verification: Verification | None = None
    audit_ref: str | None = None
    started_at: str = field(default_factory=utc_now)
    finished_at: str = field(default_factory=utc_now)

    def to_dict(self) -> dict[str, Any]:
        return {
            "action_id": self.action_id,
            "capability": self.capability,
            "status": self.status.value,
            "started_at": self.started_at,
            "finished_at": self.finished_at,
            "summary": self.summary,
            "data": self.data,
            "error": self.error,
            "verification": self.verification.to_dict() if self.verification else None,
            "audit_ref": self.audit_ref,
        }


Executor = Callable[[ActionRequest, dict[str, Any]], dict[str, Any]]
Verifier = Callable[[ActionRequest, dict[str, Any], dict[str, Any]], Verification]


@dataclass(frozen=True)
class Capability:
    metadata: CapabilityMetadata
    executor: Executor
    verifier: Verifier

    def validate_params(self, params: dict[str, Any]) -> None:
        schema = self.metadata.input_schema
        required = schema.get("required", [])
        properties = schema.get("properties", {})

        for key in required:
            if key not in params:
                raise CapabilityError(f"Missing required parameter: {key}")

        for key, value in params.items():
            expected = properties.get(key, {}).get("type")
            if expected is None:
                continue
            if expected == "string" and not isinstance(value, str):
                raise CapabilityError(f"Parameter {key} must be a string.")
            if expected == "boolean" and not isinstance(value, bool):
                raise CapabilityError(f"Parameter {key} must be a boolean.")
            if expected == "integer" and (not isinstance(value, int) or isinstance(value, bool)):
                raise CapabilityError(f"Parameter {key} must be an integer.")
