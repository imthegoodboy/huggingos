import tempfile
import unittest
from pathlib import Path
import sys

PRODUCT_ROOT = Path(__file__).resolve().parents[1]
if str(PRODUCT_ROOT) not in sys.path:
    sys.path.insert(0, str(PRODUCT_ROOT))

from huggingos_core.audit import AuditLogger
from huggingos_core.capabilities import build_registry
from huggingos_core.engine import CapabilityEngine
from huggingos_core.models import (
    ActionRequest,
    ActionResult,
    ActionStatus,
    Capability,
    CapabilityError,
    CapabilityMetadata,
    RiskLevel,
    Verification,
)
from huggingos_core.policy import PolicyDecision, PolicyEngine


def test_config(tmp_dir):
    return {
        "product": {
            "name": "huggingOS",
            "version": "test",
            "track": "product",
            "phase": "Product Phase 2",
            "base_strategy": "test",
        },
        "runtime": {},
        "features": {},
        "policy": {"audit_log_name": "audit.log", "confirmation_required_for": []},
        "_meta": {"config_path": "test"},
        "_test_state_dir": tmp_dir,
    }


class CapabilityCoreTests(unittest.TestCase):
    def test_action_contract_serializes_required_fields(self):
        request = ActionRequest(
            capability="product.status",
            params={},
            actor="tester",
            reason="contract test",
            dry_run=True,
        )
        result = ActionResult(
            action_id=request.action_id,
            capability=request.capability,
            status=ActionStatus.SUCCEEDED,
            summary="ok",
            verification=Verification(True, "verified"),
        )

        request_payload = request.to_dict()
        result_payload = result.to_dict()

        self.assertEqual(request_payload["actor"], "tester")
        self.assertEqual(request_payload["reason"], "contract test")
        self.assertTrue(request_payload["dry_run"])
        self.assertIn("requested_at", request_payload)
        self.assertEqual(result_payload["status"], "succeeded")
        self.assertIn("started_at", result_payload)
        self.assertIn("finished_at", result_payload)
        self.assertIn("verification", result_payload)
        self.assertIn("audit_ref", result_payload)

    def test_registry_rejects_duplicate_capabilities(self):
        registry = build_registry()
        capability = registry.get("product.status")

        with self.assertRaises(Exception):
            registry.register(capability)

    def test_schema_validation_requires_params(self):
        capability = build_registry().get("fs.list")

        with self.assertRaises(CapabilityError):
            capability.validate_params({})

    def test_schema_validation_rejects_boolean_as_integer(self):
        capability = build_registry().get("audit.list")

        with self.assertRaises(CapabilityError):
            capability.validate_params({"limit": True})

    def test_policy_allows_read_and_dry_run(self):
        registry = build_registry()
        config = {"runtime": {}, "policy": {}}
        policy = PolicyEngine(config)

        read = policy.decide(
            registry.get("product.status"),
            ActionRequest(capability="product.status", params={}),
        )
        dry_run = policy.decide(
            registry.get("notes.create"),
            ActionRequest(
                capability="notes.create",
                params={"title": "x"},
                dry_run=True,
            ),
        )

        self.assertEqual(read.decision, PolicyDecision.ALLOW)
        self.assertEqual(dry_run.decision, PolicyDecision.ALLOW)

    def test_policy_requires_confirmation_for_medium_risk(self):
        policy = PolicyEngine({"runtime": {}, "policy": {}})
        capability = Capability(
            CapabilityMetadata(
                name="test.medium",
                version="1.0.0",
                owner="tests",
                description="Medium risk test capability.",
                risk=RiskLevel.MEDIUM,
            ),
            lambda request, config: {},
            lambda request, config, data: Verification(True, "ok"),
        )

        needs_confirmation = policy.decide(
            capability,
            ActionRequest(capability="test.medium", params={}),
        )
        confirmed = policy.decide(
            capability,
            ActionRequest(capability="test.medium", params={}, confirmed=True),
        )

        self.assertEqual(needs_confirmation.decision, PolicyDecision.CONFIRM)
        self.assertEqual(confirmed.decision, PolicyDecision.ALLOW)

    def test_engine_records_denied_actions(self):
        with tempfile.TemporaryDirectory() as tmp_dir:
            config = test_config(tmp_dir)
            audit = AuditLogger(Path(tmp_dir) / "audit.log")
            engine = CapabilityEngine(config, build_registry(), audit=audit)
            result = engine.execute(ActionRequest(capability="missing", params={}))

            self.assertEqual(result.status.value, "denied")
            self.assertTrue(audit.path.exists())
            self.assertIn("missing", audit.path.read_text(encoding="utf-8"))

    def test_engine_records_failed_and_dry_run_actions(self):
        with tempfile.TemporaryDirectory() as tmp_dir:
            config = test_config(tmp_dir)
            audit = AuditLogger(Path(tmp_dir) / "audit.log")
            engine = CapabilityEngine(config, build_registry(), audit=audit)

            failed = engine.execute(
                ActionRequest(
                    capability="fs.read_text",
                    params={"path": str(Path(tmp_dir) / "missing.txt")},
                )
            )
            dry_run = engine.execute(
                ActionRequest(
                    capability="notes.create",
                    params={"title": "dry run"},
                    dry_run=True,
                )
            )

            audit_text = audit.path.read_text(encoding="utf-8")
            self.assertEqual(failed.status.value, "failed")
            self.assertEqual(dry_run.status.value, "dry_run")
            self.assertIn('"status": "failed"', audit_text)
            self.assertIn('"status": "dry_run"', audit_text)


if __name__ == "__main__":
    unittest.main()
