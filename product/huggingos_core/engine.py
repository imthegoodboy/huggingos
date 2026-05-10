from __future__ import annotations

from typing import Any

from .audit import AuditLogger
from .models import ActionRequest, ActionResult, ActionStatus, CapabilityError, Verification, utc_now
from .policy import PolicyDecision, PolicyEngine, PolicyOutcome
from .registry import CapabilityRegistry, RegistryError


class CapabilityEngine:
    def __init__(
        self,
        config: dict[str, Any],
        registry: CapabilityRegistry,
        policy: PolicyEngine | None = None,
        audit: AuditLogger | None = None,
    ):
        self.config = config
        self.registry = registry
        self.policy = policy or PolicyEngine(config)
        self.audit = audit or AuditLogger.from_config(config)

    def execute(self, request: ActionRequest) -> ActionResult:
        started_at = utc_now()
        try:
            capability = self.registry.get(request.capability)
            capability.validate_params(request.params)
            outcome = self.policy.decide(capability, request)
        except (RegistryError, CapabilityError) as exc:
            outcome = PolicyOutcome(PolicyDecision.DENY, str(exc))
            result = ActionResult(
                action_id=request.action_id,
                capability=request.capability,
                status=ActionStatus.DENIED,
                started_at=started_at,
                finished_at=utc_now(),
                summary="Capability request denied.",
                error=str(exc),
                verification=Verification(False, str(exc)),
            )
            return self.record_audit(request, outcome, result)

        audit_failure = self.ensure_audit_ready(request, outcome, started_at)
        if audit_failure is not None:
            return audit_failure

        if outcome.decision == PolicyDecision.DENY:
            result = ActionResult(
                action_id=request.action_id,
                capability=request.capability,
                status=ActionStatus.DENIED,
                started_at=started_at,
                finished_at=utc_now(),
                summary="Capability denied by policy.",
                error=outcome.reason,
                verification=Verification(False, outcome.reason),
            )
            return self.record_audit(request, outcome, result)

        if outcome.decision == PolicyDecision.CONFIRM:
            result = ActionResult(
                action_id=request.action_id,
                capability=request.capability,
                status=ActionStatus.CONFIRMATION_REQUIRED,
                started_at=started_at,
                finished_at=utc_now(),
                summary="Capability requires confirmation.",
                error=outcome.reason,
                verification=Verification(False, outcome.reason),
            )
            return self.record_audit(request, outcome, result)

        if request.dry_run or outcome.decision == PolicyDecision.DRY_RUN_ONLY:
            result = ActionResult(
                action_id=request.action_id,
                capability=request.capability,
                status=ActionStatus.DRY_RUN,
                started_at=started_at,
                finished_at=utc_now(),
                summary=f"Dry run: {request.capability} would execute.",
                data={"params": request.params},
                verification=Verification(True, "Dry run completed without mutation."),
            )
            return self.record_audit(request, outcome, result)

        try:
            data = capability.executor(request, self.config)
            verification = capability.verifier(request, self.config, data)
            status = ActionStatus.SUCCEEDED if verification.ok else ActionStatus.FAILED
            result = ActionResult(
                action_id=request.action_id,
                capability=request.capability,
                status=status,
                started_at=started_at,
                finished_at=utc_now(),
                summary=verification.message,
                data=data,
                verification=verification,
            )
        except CapabilityError as exc:
            result = ActionResult(
                action_id=request.action_id,
                capability=request.capability,
                status=ActionStatus.FAILED,
                started_at=started_at,
                finished_at=utc_now(),
                summary="Capability failed.",
                error=str(exc),
                verification=Verification(False, str(exc)),
            )
        except OSError as exc:
            result = ActionResult(
                action_id=request.action_id,
                capability=request.capability,
                status=ActionStatus.FAILED,
                started_at=started_at,
                finished_at=utc_now(),
                summary="Capability failed due to an OS error.",
                error=str(exc),
                verification=Verification(False, str(exc)),
            )

        return self.record_audit(request, outcome, result)

    def ensure_audit_ready(
        self,
        request: ActionRequest,
        outcome: PolicyOutcome,
        started_at: str,
    ) -> ActionResult | None:
        try:
            self.audit.ensure_ready()
            return None
        except OSError as exc:
            return ActionResult(
                action_id=request.action_id,
                capability=request.capability,
                status=ActionStatus.FAILED,
                started_at=started_at,
                finished_at=utc_now(),
                summary="Capability blocked because audit logging is unavailable.",
                error=f"Audit logging failed: {exc}",
                verification=Verification(False, outcome.reason),
            )

    def record_audit(
        self,
        request: ActionRequest,
        outcome: PolicyOutcome,
        result: ActionResult,
    ) -> ActionResult:
        try:
            result.audit_ref = self.audit.append(request, outcome, result)
        except OSError as exc:
            result.status = ActionStatus.FAILED
            result.summary = "Capability completed but audit logging failed."
            result.error = f"Audit logging failed: {exc}"
            result.verification = Verification(False, result.error)
        return result
