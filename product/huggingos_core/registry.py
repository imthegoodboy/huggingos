from __future__ import annotations

from dataclasses import dataclass, field

from .models import Capability


class RegistryError(Exception):
    """Raised when capability registration or lookup fails."""


@dataclass
class CapabilityRegistry:
    _capabilities: dict[str, Capability] = field(default_factory=dict)

    def register(self, capability: Capability) -> None:
        name = capability.metadata.name
        if name in self._capabilities:
            raise RegistryError(f"Duplicate capability: {name}")
        self._capabilities[name] = capability

    def get(self, name: str) -> Capability:
        try:
            return self._capabilities[name]
        except KeyError as exc:
            raise RegistryError(f"Unknown capability: {name}") from exc

    def list(self) -> list[Capability]:
        return [self._capabilities[name] for name in sorted(self._capabilities)]

    def to_dicts(self) -> list[dict[str, object]]:
        return [capability.metadata.to_dict() for capability in self.list()]
