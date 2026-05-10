import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CLI = REPO_ROOT / "product" / "cli" / "huggingos.py"


def run_cli(*args, env=None):
    merged_env = os.environ.copy()
    if env:
        merged_env.update(env)
    return subprocess.run(
        [sys.executable, str(CLI), *args],
        cwd=REPO_ROOT,
        env=merged_env,
        text=True,
        capture_output=True,
        check=False,
    )


class HuggingOsCliTests(unittest.TestCase):
    def test_status_reports_real_product_state(self):
        result = run_cli("status", "--json")
        self.assertEqual(result.returncode, 0, result.stderr)

        payload = json.loads(result.stdout)
        self.assertEqual(payload["product"], "huggingOS")
        self.assertEqual(payload["track"], "product")
        self.assertEqual(payload["phase"], "Product Phase 5")
        self.assertIn("python", payload["host"])
        self.assertTrue(Path(payload["paths"]["config_file"]).exists())
        self.assertIn("audit_log", payload["paths"])

    def test_doctor_passes_product_foundation(self):
        result = run_cli("doctor", "--json")
        self.assertEqual(result.returncode, 0, result.stderr)

        payload = json.loads(result.stdout)
        self.assertEqual(payload["status"], "pass")
        self.assertEqual(payload["error_count"], 0)
        self.assertTrue(payload["checks"])

    def test_config_output_redacts_secret_like_keys(self):
        with tempfile.TemporaryDirectory() as tmp_dir:
            config_path = Path(tmp_dir) / "local.toml"
            config_path.write_text(
                """
[product]
name = "huggingOS"
version = "test"
track = "product"
phase = "Product Phase 1"
base_strategy = "test"

[features]
cloud_ai_enabled = false

[policy]
api_key = "should-not-print"
confirmation_required_for = ["delete", "secret", "system"]
""".strip(),
                encoding="utf-8",
            )

            result = run_cli(
                "config",
                "--json",
                env={"HUGGINGOS_CONFIG_FILE": str(config_path)},
            )

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertNotIn("should-not-print", result.stdout)
        payload = json.loads(result.stdout)
        self.assertEqual(payload["config"]["policy"]["api_key"], "<redacted>")

    def test_capabilities_list_includes_product_status(self):
        result = run_cli("capabilities", "--json")
        self.assertEqual(result.returncode, 0, result.stderr)

        payload = json.loads(result.stdout)
        names = {capability["name"] for capability in payload["capabilities"]}
        self.assertIn("product.status", names)
        self.assertIn("notes.create", names)

    def test_run_product_status_writes_audit(self):
        with tempfile.TemporaryDirectory() as tmp_dir:
            result = run_cli(
                "run",
                "product.status",
                "--json",
                env={"HUGGINGOS_STATE_DIR": tmp_dir},
            )

            self.assertEqual(result.returncode, 0, result.stderr)
            payload = json.loads(result.stdout)
            self.assertEqual(payload["status"], "succeeded")
            self.assertEqual(payload["capability"], "product.status")
            audit_log = Path(tmp_dir) / "audit.log"
            self.assertTrue(audit_log.exists())
            self.assertIn("product.status", audit_log.read_text(encoding="utf-8"))

    def test_note_create_dry_run_does_not_write_file(self):
        with tempfile.TemporaryDirectory() as tmp_dir:
            workspace = Path(tmp_dir) / "workspace"
            result = run_cli(
                "run",
                "notes.create",
                "--param",
                "title=Phase Two",
                "--param",
                "content=hello",
                "--dry-run",
                "--json",
                env={"HUGGINGOS_STATE_DIR": tmp_dir, "HUGGINGOS_WORKSPACE_DIR": str(workspace)},
            )

            self.assertEqual(result.returncode, 0, result.stderr)
            payload = json.loads(result.stdout)
            self.assertEqual(payload["status"], "dry_run")
            self.assertFalse((workspace / "phase-two.md").exists())
            self.assertIn("dry_run", (Path(tmp_dir) / "audit.log").read_text(encoding="utf-8"))

    def test_note_create_writes_only_inside_workspace(self):
        with tempfile.TemporaryDirectory() as tmp_dir:
            workspace = Path(tmp_dir) / "workspace"
            result = run_cli(
                "run",
                "notes.create",
                "--param",
                "title=Phase Two",
                "--param",
                "content=hello",
                "--json",
                env={"HUGGINGOS_STATE_DIR": tmp_dir, "HUGGINGOS_WORKSPACE_DIR": str(workspace)},
            )

            self.assertEqual(result.returncode, 0, result.stderr)
            payload = json.loads(result.stdout)
            self.assertEqual(payload["status"], "succeeded")
            note_path = Path(payload["data"]["path"])
            self.assertEqual(note_path, workspace / "phase-two.md")
            self.assertTrue(note_path.exists())

    def test_note_create_rejects_empty_title(self):
        with tempfile.TemporaryDirectory() as tmp_dir:
            result = run_cli(
                "run",
                "notes.create",
                "--param",
                "title=",
                "--json",
                env={"HUGGINGOS_STATE_DIR": tmp_dir},
            )

            self.assertEqual(result.returncode, 1)
            payload = json.loads(result.stdout)
            self.assertEqual(payload["status"], "failed")
            self.assertIn("title", payload["error"])

    def test_read_text_denies_sensitive_path_before_reading(self):
        with tempfile.TemporaryDirectory() as tmp_dir:
            secret_path = Path(tmp_dir) / ".env"
            secret_path.write_text("API_KEY=should-not-print", encoding="utf-8")

            result = run_cli(
                "run",
                "fs.read_text",
                "--param",
                f"path={secret_path}",
                "--json",
                env={"HUGGINGOS_STATE_DIR": tmp_dir},
            )

            self.assertEqual(result.returncode, 1)
            self.assertNotIn("should-not-print", result.stdout)
            payload = json.loads(result.stdout)
            self.assertEqual(payload["status"], "denied")

    def test_unknown_capability_is_denied_and_audited(self):
        with tempfile.TemporaryDirectory() as tmp_dir:
            result = run_cli(
                "run",
                "system.delete_everything",
                "--json",
                env={"HUGGINGOS_STATE_DIR": tmp_dir},
            )

            self.assertEqual(result.returncode, 1)
            payload = json.loads(result.stdout)
            self.assertEqual(payload["status"], "denied")
            audit_log = Path(tmp_dir) / "audit.log"
            self.assertTrue(audit_log.exists())

    def test_audit_list_returns_recent_entries(self):
        with tempfile.TemporaryDirectory() as tmp_dir:
            first = run_cli(
                "run",
                "product.status",
                "--json",
                env={"HUGGINGOS_STATE_DIR": tmp_dir},
            )
            self.assertEqual(first.returncode, 0, first.stderr)

            result = run_cli(
                "run",
                "audit.list",
                "--param",
                "limit=10",
                "--json",
                env={"HUGGINGOS_STATE_DIR": tmp_dir},
            )

            self.assertEqual(result.returncode, 0, result.stderr)
            payload = json.loads(result.stdout)
            self.assertEqual(payload["status"], "succeeded")
            self.assertGreaterEqual(payload["data"]["entry_count"], 1)


if __name__ == "__main__":
    unittest.main()
