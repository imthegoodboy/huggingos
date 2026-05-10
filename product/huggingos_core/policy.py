from __future__ import annotations

import re
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from .config import workspace_dir
from .models import ActionRequest, Capability, PolicyDecision, RiskLevel


SENSITIVE_PATH_PARTS = {
    ".aws",
    ".azure",
    ".docker",
    ".gnupg",
    ".kube",
    ".password-store",
    ".ssh",
}
SENSITIVE_FILENAMES = {
    ".env",
    ".npmrc",
    ".pypirc",
    "credentials",
    "credentials.json",
    "id_dsa",
    "id_ecdsa",
    "id_ed25519",
    "id_rsa",
}
SENSITIVE_NAME_PATTERN = re.compile(
    r"(^|[._-])(api[-_]?keys?|credentials?|password|private[-_]?keys?|secret|token)([._-]|$)"
)


@dataclass(frozen=True)
class PolicyOutcome:
    decision: PolicyDecision
    reason: str

    def to_dict(self) -> dict[str, str]:
        return {"decision": self.decision.value, "reason": self.reason}


class PolicyEngine:
    def __init__(self, config: dict[str, Any]):
        self.config = config

    def decide(self, capability: Capability, request: ActionRequest) -> PolicyOutcome:
        risk = capability.metadata.risk

        if request.dry_run:
            return PolicyOutcome(PolicyDecision.ALLOW, "Dry run allowed without mutation.")

        path_value = request.params.get("path")
        if capability.metadata.name in {"fs.list", "fs.read_text"} and is_sensitive_path(path_value):
            return PolicyOutcome(PolicyDecision.DENY, "Sensitive paths require a higher-risk capability.")

        if risk == RiskLevel.READ:
            return PolicyOutcome(PolicyDecision.ALLOW, "Read-only capability allowed.")

        if risk == RiskLevel.LOW:
            if capability.metadata.name == "notes.create" and not str(workspace_dir(self.config)):
                return PolicyOutcome(PolicyDecision.DENY, "No safe workspace is configured.")
            return PolicyOutcome(PolicyDecision.ALLOW, "Low-risk capability allowed.")

        if risk == RiskLevel.MEDIUM:
            if request.confirmed:
                return PolicyOutcome(PolicyDecision.ALLOW, "Medium-risk capability confirmed.")
            return PolicyOutcome(PolicyDecision.CONFIRM, "Medium-risk capability requires confirmation.")

        if risk == RiskLevel.HIGH:
            if request.confirmed:
                return PolicyOutcome(PolicyDecision.CONFIRM, "High-risk capability requires interactive approval.")
            return PolicyOutcome(PolicyDecision.DENY, "High-risk capability denied by default.")

        return PolicyOutcome(PolicyDecision.DENY, f"Unsupported risk level: {risk}")


def is_sensitive_path(value: Any) -> bool:
    if value is None:
        return False
    normalized = str(Path(str(value)).expanduser()).replace("\\", "/")
    parts = [part.lower() for part in normalized.split("/") if part]
    for part in parts:
        if part in SENSITIVE_PATH_PARTS or part in SENSITIVE_FILENAMES:
            return True
        if SENSITIVE_NAME_PATTERN.search(part):
            return True
    return False
