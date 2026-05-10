from __future__ import annotations

from dataclasses import dataclass

from .config import workspace_dir
from .models import ActionRequest, Capability, PolicyDecision, RiskLevel


@dataclass(frozen=True)
class PolicyOutcome:
    decision: PolicyDecision
    reason: str

    def to_dict(self) -> dict[str, str]:
        return {"decision": self.decision.value, "reason": self.reason}


class PolicyEngine:
    def __init__(self, config: dict[str, object]):
        self.config = config

    def decide(self, capability: Capability, request: ActionRequest) -> PolicyOutcome:
        risk = capability.metadata.risk

        if request.dry_run:
            return PolicyOutcome(PolicyDecision.ALLOW, "Dry run allowed without mutation.")

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
