from __future__ import annotations

import re
import stat
from pathlib import Path
from typing import Any

from .audit import AuditLogger
from .config import product_status, workspace_dir
from .models import (
    ActionRequest,
    Capability,
    CapabilityError,
    CapabilityMetadata,
    RiskLevel,
    Verification,
)
from .registry import CapabilityRegistry


MAX_TEXT_BYTES = 64 * 1024


def build_registry() -> CapabilityRegistry:
    registry = CapabilityRegistry()
    for capability in (
        product_status_capability(),
        fs_list_capability(),
        fs_read_text_capability(),
        notes_create_capability(),
        audit_list_capability(),
    ):
        registry.register(capability)
    return registry


def product_status_capability() -> Capability:
    metadata = CapabilityMetadata(
        name="product.status",
        version="1.0.0",
        owner="huggingos",
        description="Report real product and host status.",
        risk=RiskLevel.READ,
        permissions=("product:read",),
        input_schema={"type": "object", "properties": {}, "required": []},
        result_schema={"type": "object"},
    )

    def execute(request: ActionRequest, config: dict[str, Any]) -> dict[str, Any]:
        return product_status(config)

    def verify(
        request: ActionRequest,
        config: dict[str, Any],
        data: dict[str, Any],
    ) -> Verification:
        ok = data.get("track") == "product" and "host" in data
        return Verification(ok, "Product status returned host state." if ok else "Invalid product status.")

    return Capability(metadata, execute, verify)


def fs_list_capability() -> Capability:
    metadata = CapabilityMetadata(
        name="fs.list",
        version="1.0.0",
        owner="huggingos",
        description="List a local directory without changing filesystem state.",
        risk=RiskLevel.READ,
        permissions=("fs:read",),
        input_schema={
            "type": "object",
            "properties": {"path": {"type": "string"}},
            "required": ["path"],
        },
        result_schema={"type": "object"},
    )

    def execute(request: ActionRequest, config: dict[str, Any]) -> dict[str, Any]:
        path = resolve_existing_path(str(request.params["path"]))
        if not path.is_dir():
            raise CapabilityError(f"Path is not a directory: {path}")

        entries = []
        for child in sorted(path.iterdir(), key=lambda item: item.name.lower()):
            try:
                stat = child.stat()
            except OSError:
                continue
            entries.append(
                {
                    "name": child.name,
                    "path": str(child),
                    "type": "directory" if child.is_dir() else "file",
                    "size": stat.st_size,
                }
            )
        return {"path": str(path), "entries": entries, "entry_count": len(entries)}

    def verify(
        request: ActionRequest,
        config: dict[str, Any],
        data: dict[str, Any],
    ) -> Verification:
        ok = Path(data["path"]).is_dir()
        return Verification(ok, "Directory listing verified." if ok else "Directory no longer exists.")

    return Capability(metadata, execute, verify)


def fs_read_text_capability() -> Capability:
    metadata = CapabilityMetadata(
        name="fs.read_text",
        version="1.0.0",
        owner="huggingos",
        description="Read a small UTF-8 text file with a size limit.",
        risk=RiskLevel.READ,
        permissions=("fs:read",),
        input_schema={
            "type": "object",
            "properties": {"path": {"type": "string"}},
            "required": ["path"],
        },
        result_schema={"type": "object"},
    )

    def execute(request: ActionRequest, config: dict[str, Any]) -> dict[str, Any]:
        path = resolve_existing_path(str(request.params["path"]))
        if not path.is_file():
            raise CapabilityError(f"Path is not a file: {path}")
        file_stat = path.stat()
        if not stat.S_ISREG(file_stat.st_mode):
            raise CapabilityError(f"Path is not a regular file: {path}")
        size = file_stat.st_size
        if size > MAX_TEXT_BYTES:
            raise CapabilityError(f"File is too large for Phase 2 text read: {size} bytes.")
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError as exc:
            raise CapabilityError(f"File is not valid UTF-8 text: {path}") from exc
        return {"path": str(path), "size": size, "text": text}

    def verify(
        request: ActionRequest,
        config: dict[str, Any],
        data: dict[str, Any],
    ) -> Verification:
        ok = Path(data["path"]).is_file()
        return Verification(ok, "Text file read verified." if ok else "File no longer exists.")

    return Capability(metadata, execute, verify)


def notes_create_capability() -> Capability:
    metadata = CapabilityMetadata(
        name="notes.create",
        version="1.0.0",
        owner="huggingos",
        description="Create a text note inside the configured safe workspace.",
        risk=RiskLevel.LOW,
        permissions=("notes:create",),
        input_schema={
            "type": "object",
            "properties": {
                "title": {"type": "string"},
                "content": {"type": "string"},
                "filename": {"type": "string"},
            },
            "required": ["title"],
        },
        result_schema={"type": "object"},
        reversible=True,
    )

    def execute(request: ActionRequest, config: dict[str, Any]) -> dict[str, Any]:
        workspace = workspace_dir(config).resolve()
        workspace.mkdir(parents=True, exist_ok=True)
        title = str(request.params["title"]).strip()
        if not title:
            raise CapabilityError("Note title cannot be empty.")

        filename = safe_note_filename(str(request.params.get("filename") or title))
        path = (workspace / filename).resolve()
        if not is_relative_to(path, workspace):
            raise CapabilityError("Refusing to create a note outside the configured workspace.")

        content = str(request.params.get("content", ""))
        note_text = f"# {title}\n\n{content.rstrip()}\n"
        try:
            with path.open("x", encoding="utf-8") as note_file:
                note_file.write(note_text)
        except FileExistsError as exc:
            raise CapabilityError(f"Refusing to overwrite existing note: {path}") from exc
        return {"path": str(path), "workspace": str(workspace), "bytes": len(note_text.encode("utf-8"))}

    def verify(
        request: ActionRequest,
        config: dict[str, Any],
        data: dict[str, Any],
    ) -> Verification:
        path = Path(data["path"])
        ok = path.exists() and path.is_file()
        return Verification(ok, "Note exists in safe workspace." if ok else "Note was not created.")

    return Capability(metadata, execute, verify)


def audit_list_capability() -> Capability:
    metadata = CapabilityMetadata(
        name="audit.list",
        version="1.0.0",
        owner="huggingos",
        description="Show recent capability audit records.",
        risk=RiskLevel.READ,
        permissions=("audit:read",),
        input_schema={
            "type": "object",
            "properties": {"limit": {"type": "integer"}},
            "required": [],
        },
        result_schema={"type": "object"},
    )

    def execute(request: ActionRequest, config: dict[str, Any]) -> dict[str, Any]:
        limit = int(request.params.get("limit", 20))
        if limit < 1 or limit > 200:
            raise CapabilityError("Audit list limit must be between 1 and 200.")
        logger = AuditLogger.from_config(config)
        entries = logger.list_entries(limit)
        return {"path": str(logger.path), "entries": entries, "entry_count": len(entries)}

    def verify(
        request: ActionRequest,
        config: dict[str, Any],
        data: dict[str, Any],
    ) -> Verification:
        return Verification("entries" in data, "Audit entries loaded.")

    return Capability(metadata, execute, verify)


def resolve_existing_path(raw_path: str) -> Path:
    path = Path(raw_path).expanduser()
    if not path.is_absolute():
        path = (Path.cwd() / path).resolve()
    else:
        path = path.resolve()
    if not path.exists():
        raise CapabilityError(f"Path does not exist: {path}")
    return path


def safe_note_filename(value: str) -> str:
    stem = re.sub(r"[^A-Za-z0-9._-]+", "-", value.strip()).strip(".-").lower()
    if not stem:
        stem = "note"
    if not stem.endswith(".md"):
        stem = f"{stem}.md"
    return Path(stem).name


def is_relative_to(path: Path, base: Path) -> bool:
    try:
        path.relative_to(base)
        return True
    except ValueError:
        return False
