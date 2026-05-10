#!/usr/bin/env python3
"""Phase 1 huggingOS product CLI."""

from __future__ import annotations

import argparse
import json
import os
import platform
import sys
from pathlib import Path
from typing import Any

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - Python < 3.11 fallback path
    tomllib = None


PRODUCT_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_CONFIG = PRODUCT_ROOT / "config" / "defaults.toml"
MIN_PYTHON = (3, 11)


class CliError(Exception):
    """User-visible CLI error."""


def load_config() -> dict[str, Any]:
    if tomllib is None:
        raise CliError("Python 3.11 or newer is required for TOML config support.")

    config_path = Path(os.environ.get("HUGGINGOS_CONFIG_FILE", DEFAULT_CONFIG))
    if not config_path.is_absolute():
        config_path = (Path.cwd() / config_path).resolve()

    if not config_path.exists():
        raise CliError(f"Config file not found: {config_path}")

    try:
        with config_path.open("rb") as config_file:
            config = tomllib.load(config_file)
    except tomllib.TOMLDecodeError as exc:
        raise CliError(f"Invalid TOML config in {config_path}: {exc}") from exc

    config["_meta"] = {"config_path": str(config_path)}
    return config


def xdg_state_home() -> Path:
    explicit = os.environ.get("HUGGINGOS_STATE_DIR")
    if explicit:
        return Path(explicit).expanduser()

    xdg_state = os.environ.get("XDG_STATE_HOME")
    if xdg_state:
        return Path(xdg_state).expanduser() / "huggingos"

    return Path.home() / ".local" / "state" / "huggingos"


def xdg_config_home() -> Path:
    xdg_config = os.environ.get("XDG_CONFIG_HOME")
    if xdg_config:
        return Path(xdg_config).expanduser() / "huggingos"

    return Path.home() / ".config" / "huggingos"


def redact(value: Any) -> Any:
    if isinstance(value, dict):
        redacted: dict[str, Any] = {}
        for key, item in value.items():
            lowered = key.lower()
            if any(marker in lowered for marker in ("secret", "token", "key", "password")):
                redacted[key] = "<redacted>"
            else:
                redacted[key] = redact(item)
        return redacted
    if isinstance(value, list):
        return [redact(item) for item in value]
    return value


def host_info() -> dict[str, Any]:
    return {
        "system": platform.system(),
        "release": platform.release(),
        "version": platform.version(),
        "machine": platform.machine(),
        "python": platform.python_version(),
        "is_linux": sys.platform.startswith("linux"),
        "is_wsl": "microsoft" in platform.release().lower()
        or "WSL_DISTRO_NAME" in os.environ,
    }


def product_status(config: dict[str, Any]) -> dict[str, Any]:
    product = config.get("product", {})
    return {
        "product": product.get("name", "huggingOS"),
        "version": product.get("version", "unknown"),
        "track": product.get("track", "product"),
        "phase": product.get("phase", "Product Phase 1"),
        "base_strategy": product.get("base_strategy", "Ubuntu LTS hosted prototype"),
        "host": host_info(),
        "paths": {
            "product_root": str(PRODUCT_ROOT),
            "config_file": config.get("_meta", {}).get("config_path", str(DEFAULT_CONFIG)),
            "config_dir": str(xdg_config_home()),
            "state_dir": str(xdg_state_home()),
        },
        "features": config.get("features", {}),
    }


def doctor_report(config: dict[str, Any]) -> dict[str, Any]:
    checks = []

    def add_check(name: str, ok: bool, message: str, severity: str = "error") -> None:
        checks.append(
            {
                "name": name,
                "ok": ok,
                "severity": "info" if ok else severity,
                "message": message,
            }
        )

    add_check(
        "python",
        sys.version_info >= MIN_PYTHON,
        f"Python {platform.python_version()} detected; requires 3.11+.",
    )
    add_check(
        "linux-host",
        sys.platform.startswith("linux"),
        (
            "Linux host detected."
            if sys.platform.startswith("linux")
            else "Run product commands on Linux, WSL, or CI."
        ),
        severity="warning",
    )
    add_check(
        "default-config",
        DEFAULT_CONFIG.exists(),
        f"Default config present at {DEFAULT_CONFIG}.",
    )
    add_check("cli", (PRODUCT_ROOT / "cli" / "huggingos.py").exists(), "CLI entrypoint is present.")
    add_check("tests", (PRODUCT_ROOT / "tests").exists(), "Product tests directory is present.")

    policy = config.get("policy", {})
    required_actions = set(policy.get("confirmation_required_for", []))
    add_check(
        "policy-confirmations",
        {"delete", "secret", "system"}.issubset(required_actions),
        "Policy lists high-risk actions that must require confirmation.",
    )

    errors = [check for check in checks if not check["ok"] and check["severity"] == "error"]
    warnings = [check for check in checks if not check["ok"] and check["severity"] == "warning"]
    return {
        "product": "huggingOS",
        "status": "pass" if not errors else "fail",
        "error_count": len(errors),
        "warning_count": len(warnings),
        "checks": checks,
    }


def emit(payload: dict[str, Any], as_json: bool) -> None:
    if as_json:
        print(json.dumps(payload, indent=2, sort_keys=True))
        return

    if "checks" in payload:
        print(f"huggingOS doctor: {payload['status']}")
        for check in payload["checks"]:
            label = "OK" if check["ok"] else check["severity"].upper()
            print(f"[{label}] {check['name']}: {check['message']}")
        return

    if "config" in payload:
        print(json.dumps(payload["config"], indent=2, sort_keys=True))
        return

    print(f"{payload['product']} {payload['version']}")
    print(f"track: {payload['track']}")
    print(f"phase: {payload['phase']}")
    print(f"base: {payload['base_strategy']}")
    print(f"host: {payload['host']['system']} {payload['host']['release']} ({payload['host']['machine']})")
    print(f"config: {payload['paths']['config_file']}")
    print(f"state: {payload['paths']['state_dir']}")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="huggingos", description="huggingOS product CLI")
    subparsers = parser.add_subparsers(dest="command", required=True)

    status = subparsers.add_parser("status", help="Show real product and host status.")
    status.add_argument("--json", action="store_true", help="Emit machine-readable JSON.")

    doctor = subparsers.add_parser("doctor", help="Run Product Phase 1 environment checks.")
    doctor.add_argument("--json", action="store_true", help="Emit machine-readable JSON.")

    config = subparsers.add_parser("config", help="Show non-secret product config.")
    config.add_argument("--json", action="store_true", help="Emit machine-readable JSON.")

    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)

    try:
        config = load_config()
        if args.command == "status":
            emit(product_status(config), args.json)
            return 0
        if args.command == "doctor":
            report = doctor_report(config)
            emit(report, args.json)
            return 0 if report["error_count"] == 0 else 1
        if args.command == "config":
            emit({"config": redact(config)}, args.json)
            return 0
    except CliError as exc:
        print(f"huggingos: {exc}", file=sys.stderr)
        return 2

    parser.error(f"unknown command: {args.command}")
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
