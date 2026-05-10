from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from .config import audit_log_path
from .models import ActionRequest, ActionResult, utc_now
from .policy import PolicyOutcome


@dataclass
class AuditLogger:
    path: Path

    @classmethod
    def from_config(cls, config: dict[str, Any]) -> "AuditLogger":
        return cls(audit_log_path(config))

    def append(
        self,
        request: ActionRequest,
        outcome: PolicyOutcome,
        result: ActionResult,
    ) -> str:
        self.path.parent.mkdir(parents=True, exist_ok=True)
        record = {
            "recorded_at": utc_now(),
            "action_id": request.action_id,
            "actor": request.actor,
            "capability": request.capability,
            "input_summary": summarize_params(request.params),
            "policy": outcome.to_dict(),
            "status": result.status.value,
            "summary": result.summary,
            "error": result.error,
            "started_at": result.started_at,
            "finished_at": result.finished_at,
            "verification": result.verification.to_dict() if result.verification else None,
        }
        with self.path.open("a", encoding="utf-8") as audit_file:
            audit_file.write(json.dumps(record, sort_keys=True) + "\n")
        return f"{self.path}:{request.action_id}"

    def list_entries(self, limit: int = 20) -> list[dict[str, Any]]:
        if not self.path.exists():
            return []
        entries: list[dict[str, Any]] = []
        with self.path.open("r", encoding="utf-8") as audit_file:
            for line in audit_file:
                line = line.strip()
                if not line:
                    continue
                entries.append(json.loads(line))
        return entries[-limit:]


def summarize_params(params: dict[str, Any]) -> dict[str, Any]:
    summary: dict[str, Any] = {}
    for key, value in params.items():
        lowered = key.lower()
        if any(marker in lowered for marker in ("secret", "token", "key", "password")):
            summary[key] = "<redacted>"
        elif isinstance(value, str) and len(value) > 120:
            summary[key] = value[:117] + "..."
        else:
            summary[key] = value
    return summary
