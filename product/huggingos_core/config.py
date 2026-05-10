from __future__ import annotations

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


class ConfigError(Exception):
    """User-visible config error."""


def load_config() -> dict[str, Any]:
    if tomllib is None:
        raise ConfigError("Python 3.11 or newer is required for TOML config support.")

    config_path = Path(os.environ.get("HUGGINGOS_CONFIG_FILE", DEFAULT_CONFIG))
    if not config_path.is_absolute():
        config_path = (Path.cwd() / config_path).resolve()

    if not config_path.exists():
        raise ConfigError(f"Config file not found: {config_path}")

    try:
        with config_path.open("rb") as config_file:
            config = tomllib.load(config_file)
    except tomllib.TOMLDecodeError as exc:
        raise ConfigError(f"Invalid TOML config in {config_path}: {exc}") from exc

    config["_meta"] = {"config_path": str(config_path)}
    return config


def xdg_state_home(config: dict[str, Any] | None = None) -> Path:
    runtime = config.get("runtime", {}) if config else {}
    env_name = runtime.get("state_dir_env") or "HUGGINGOS_STATE_DIR"
    explicit = os.environ.get(str(env_name))
    if explicit:
        return resolve_runtime_path(explicit)

    xdg_state = os.environ.get("XDG_STATE_HOME")
    if xdg_state:
        return resolve_runtime_path(xdg_state) / "huggingos"

    return resolve_runtime_path(Path.home() / ".local" / "state" / "huggingos")


def xdg_config_home() -> Path:
    xdg_config = os.environ.get("XDG_CONFIG_HOME")
    if xdg_config:
        return resolve_runtime_path(xdg_config) / "huggingos"

    return resolve_runtime_path(Path.home() / ".config" / "huggingos")


def workspace_dir(config: dict[str, Any]) -> Path:
    runtime = config.get("runtime", {})
    env_name = runtime.get("workspace_dir_env") or "HUGGINGOS_WORKSPACE_DIR"
    explicit = os.environ.get(str(env_name))
    if explicit:
        return resolve_runtime_path(explicit)

    configured = runtime.get("workspace_dir")
    if configured:
        configured_path = Path(str(configured)).expanduser()
        if configured_path.is_absolute():
            return resolve_runtime_path(configured_path)
        return resolve_runtime_path(xdg_state_home(config) / configured_path)

    return resolve_runtime_path(xdg_state_home(config) / "workspace")


def audit_log_path(config: dict[str, Any]) -> Path:
    policy = config.get("policy", {})
    log_name = Path(str(policy.get("audit_log_name", "audit.log"))).name
    if not log_name or log_name in {".", ".."}:
        log_name = "audit.log"
    return resolve_runtime_path(xdg_state_home(config) / log_name)


def resolve_runtime_path(value: str | Path) -> Path:
    return Path(value).expanduser().resolve()


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
        "phase": product.get("phase", "Product Phase 8"),
        "base_strategy": product.get("base_strategy", "Ubuntu LTS hosted prototype"),
        "host": host_info(),
        "paths": {
            "product_root": str(PRODUCT_ROOT),
            "config_file": config.get("_meta", {}).get("config_path", str(DEFAULT_CONFIG)),
            "config_dir": str(xdg_config_home()),
            "state_dir": str(xdg_state_home(config)),
            "workspace_dir": str(workspace_dir(config)),
            "audit_log": str(audit_log_path(config)),
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
    add_check(
        "workspace",
        bool(str(workspace_dir(config))),
        f"Workspace is configured as {workspace_dir(config)}.",
    )
    add_check(
        "audit-log",
        bool(str(audit_log_path(config))),
        f"Audit log path is {audit_log_path(config)}.",
    )

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
