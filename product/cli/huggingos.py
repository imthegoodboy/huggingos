#!/usr/bin/env python3
"""huggingOS product CLI."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any


PRODUCT_ROOT = Path(__file__).resolve().parents[1]
if str(PRODUCT_ROOT) not in sys.path:
    sys.path.insert(0, str(PRODUCT_ROOT))

from huggingos_core.capabilities import build_registry  # noqa: E402
from huggingos_core.config import (  # noqa: E402
    ConfigError,
    doctor_report,
    load_config,
    product_status,
    redact,
)
from huggingos_core.engine import CapabilityEngine  # noqa: E402
from huggingos_core.models import ActionRequest  # noqa: E402


class CliError(Exception):
    """User-visible CLI error."""


def emit_json(payload: dict[str, Any] | list[dict[str, Any]]) -> None:
    print(json.dumps(payload, indent=2, sort_keys=True))


def emit_status(payload: dict[str, Any], as_json: bool) -> None:
    if as_json:
        emit_json(payload)
        return

    print(f"{payload['product']} {payload['version']}")
    print(f"track: {payload['track']}")
    print(f"phase: {payload['phase']}")
    print(f"base: {payload['base_strategy']}")
    print(f"host: {payload['host']['system']} {payload['host']['release']} ({payload['host']['machine']})")
    print(f"config: {payload['paths']['config_file']}")
    print(f"state: {payload['paths']['state_dir']}")
    print(f"workspace: {payload['paths']['workspace_dir']}")
    print(f"audit: {payload['paths']['audit_log']}")


def emit_doctor(payload: dict[str, Any], as_json: bool) -> None:
    if as_json:
        emit_json(payload)
        return

    print(f"huggingOS doctor: {payload['status']}")
    for check in payload["checks"]:
        label = "OK" if check["ok"] else check["severity"].upper()
        print(f"[{label}] {check['name']}: {check['message']}")


def emit_capabilities(capabilities: list[dict[str, Any]], as_json: bool) -> None:
    if as_json:
        emit_json({"capabilities": capabilities})
        return

    for capability in capabilities:
        permissions = ", ".join(capability["permissions"]) or "none"
        print(
            f"{capability['name']} "
            f"v{capability['version']} "
            f"[{capability['risk']}] "
            f"{permissions} - {capability['description']}"
        )


def emit_result(result: dict[str, Any], as_json: bool) -> None:
    if as_json:
        emit_json(result)
        return

    print(f"{result['capability']}: {result['status']}")
    print(result["summary"])
    if result.get("error"):
        print(f"error: {result['error']}")
    if result.get("audit_ref"):
        print(f"audit: {result['audit_ref']}")
    data = result.get("data") or {}
    if data:
        print(json.dumps(data, indent=2, sort_keys=True))


def parse_param(raw: str) -> tuple[str, Any]:
    if "=" not in raw:
        raise CliError(f"Parameter must be key=value: {raw}")
    key, value = raw.split("=", 1)
    key = key.strip()
    if not key:
        raise CliError("Parameter key cannot be empty.")
    return key, parse_value(value)


def parse_value(value: str) -> Any:
    lowered = value.lower()
    if lowered == "true":
        return True
    if lowered == "false":
        return False
    try:
        return int(value)
    except ValueError:
        return value


def parse_params(param_items: list[str], params_json: str | None) -> dict[str, Any]:
    params: dict[str, Any] = {}
    if params_json:
        try:
            decoded = json.loads(params_json)
        except json.JSONDecodeError as exc:
            raise CliError(f"Invalid --params-json: {exc}") from exc
        if not isinstance(decoded, dict):
            raise CliError("--params-json must decode to an object.")
        params.update(decoded)

    for item in param_items:
        key, value = parse_param(item)
        params[key] = value
    return params


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="huggingos", description="huggingOS product CLI")
    subparsers = parser.add_subparsers(dest="command", required=True)

    status = subparsers.add_parser("status", help="Show real product and host status.")
    status.add_argument("--json", action="store_true", help="Emit machine-readable JSON.")

    doctor = subparsers.add_parser("doctor", help="Run product environment checks.")
    doctor.add_argument("--json", action="store_true", help="Emit machine-readable JSON.")

    config = subparsers.add_parser("config", help="Show non-secret product config.")
    config.add_argument("--json", action="store_true", help="Emit machine-readable JSON.")

    capabilities = subparsers.add_parser("capabilities", help="List registered capabilities.")
    capabilities.add_argument("--json", action="store_true", help="Emit machine-readable JSON.")

    run = subparsers.add_parser("run", help="Run a capability through policy and audit.")
    run.add_argument("capability", help="Capability name, for example product.status.")
    run.add_argument("--param", action="append", default=[], help="Capability parameter as key=value.")
    run.add_argument("--params-json", help="Capability parameters as a JSON object.")
    run.add_argument("--actor", default="user", help="Actor requesting the action.")
    run.add_argument("--reason", default="", help="Reason for the action.")
    run.add_argument("--dry-run", action="store_true", help="Evaluate without mutating state.")
    run.add_argument("--confirm", action="store_true", help="Confirm a capability that requires it.")
    run.add_argument("--json", action="store_true", help="Emit machine-readable JSON.")

    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)

    try:
        config = load_config()
        if args.command == "status":
            emit_status(product_status(config), args.json)
            return 0
        if args.command == "doctor":
            report = doctor_report(config)
            emit_doctor(report, args.json)
            return 0 if report["error_count"] == 0 else 1
        if args.command == "config":
            emit_json({"config": redact(config)})
            return 0
        if args.command == "capabilities":
            emit_capabilities(build_registry().to_dicts(), args.json)
            return 0
        if args.command == "run":
            request = ActionRequest(
                capability=args.capability,
                params=parse_params(args.param, args.params_json),
                actor=args.actor,
                reason=args.reason,
                dry_run=args.dry_run,
                confirmed=args.confirm,
            )
            result = CapabilityEngine(config, build_registry()).execute(request)
            emit_result(result.to_dict(), args.json)
            return 0 if result.error is None else 1
    except (CliError, ConfigError) as exc:
        print(f"huggingos: {exc}", file=sys.stderr)
        return 2

    parser.error(f"unknown command: {args.command}")
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
