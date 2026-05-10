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
        self.assertEqual(payload["phase"], "Product Phase 1")
        self.assertIn("python", payload["host"])
        self.assertTrue(Path(payload["paths"]["config_file"]).exists())

    def test_doctor_passes_phase1_foundation(self):
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


if __name__ == "__main__":
    unittest.main()
