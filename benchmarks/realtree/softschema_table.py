"""Project a directory of soft-schema Markdown artifacts into a generic table.

The projection reads only authoritative frontmatter values. It uses the compiled JSON
Schema for column labels, types, descriptions, and requiredness; Markdown body prose is
deliberately outside the data path. Callers should validate artifacts with ``softschema``
or a semantic model before projecting them when the output is a trust boundary.
"""

from __future__ import annotations

from collections import Counter
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from frontmatter_format import fmf_read_frontmatter, read_yaml_file

REPOSITORY_ROOT = Path(__file__).resolve().parents[2]


@dataclass(frozen=True, slots=True)
class ColumnSpec:
    """One scalar or compact collection to expose from an artifact payload."""

    path: str
    label: str | None = None
    display: str = "auto"

    def __post_init__(self) -> None:
        if not self.path or any(not part for part in self.path.split(".")):
            raise ValueError(f"invalid column path {self.path!r}")


def _mapping(value: Any, *, context: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise TypeError(f"{context} must be a mapping")
    return value


def _resolve_reference(schema: dict[str, Any], node: dict[str, Any]) -> dict[str, Any]:
    reference = node.get("$ref")
    if reference is None:
        return node
    if not isinstance(reference, str) or not reference.startswith("#/"):
        raise ValueError(
            f"only local JSON Schema references are supported: {reference!r}"
        )
    target: Any = schema
    for encoded_part in reference.removeprefix("#/").split("/"):
        part = encoded_part.replace("~1", "/").replace("~0", "~")
        target = _mapping(target, context=f"schema reference {reference!r}").get(part)
        if target is None:
            raise ValueError(f"unresolved schema reference {reference!r}")
    return _mapping(target, context=f"schema reference {reference!r}")


def _schema_node(schema: dict[str, Any], path: str) -> tuple[dict[str, Any], bool]:
    node = schema
    required = True
    for part in path.split("."):
        node = _resolve_reference(schema, node)
        properties = _mapping(
            node.get("properties"), context=f"properties above {path!r}"
        )
        if part not in properties:
            raise ValueError(f"column path {path!r} is absent from the compiled schema")
        required_values = node.get("required") or []
        required = required and part in required_values
        node = _mapping(properties[part], context=f"schema node for {path!r}")
    return _resolve_reference(schema, node), required


def _schema_type(node: dict[str, Any]) -> str:
    node_type = node.get("type")
    if isinstance(node_type, str):
        return node_type
    if isinstance(node_type, list):
        return " | ".join(str(value) for value in node_type)
    alternatives = node.get("anyOf")
    if isinstance(alternatives, list):
        types = [
            alternative.get("type")
            for alternative in alternatives
            if isinstance(alternative, dict) and alternative.get("type") is not None
        ]
        if types:
            return " | ".join(str(value) for value in types)
    if "$ref" in node:
        return "object"
    return "unknown"


def _column_descriptor(
    schema: dict[str, Any], column: ColumnSpec
) -> dict[str, str | bool]:
    node, required = _schema_node(schema, column.path)
    inferred_label = str(node.get("title") or column.path.rsplit(".", 1)[-1])
    return {
        "path": column.path,
        "label": column.label or inferred_label.replace("_", " ").title(),
        "type": _schema_type(node),
        "required": required,
        "description": str(node.get("description") or ""),
        "display": column.display,
    }


def _payload_value(payload: dict[str, Any], path: str) -> Any:
    value: Any = payload
    for part in path.split("."):
        if not isinstance(value, dict) or part not in value:
            return None
        value = value[part]
    return value


def _source_href(prefix: str, filename: str) -> str:
    if not prefix:
        return filename
    return f"{prefix.rstrip('/')}/{filename}"


def project_directory(
    directory: Path,
    *,
    columns: tuple[ColumnSpec, ...],
    pattern: str = "*.md",
    source_href_prefix: str = "",
) -> dict[str, Any]:
    """Return a browser-ready table projection for one-contract artifact directory."""
    if not columns:
        raise ValueError("at least one table column is required")

    artifacts: list[tuple[Path, dict[str, Any], dict[str, Any]]] = []
    expected_contract: str | None = None
    expected_envelope: str | None = None
    expected_schema_name: str | None = None
    statuses: Counter[str] = Counter()

    for path in sorted(directory.glob(pattern)):
        frontmatter = _mapping(
            fmf_read_frontmatter(path), context=f"frontmatter in {path}"
        )
        metadata = _mapping(
            frontmatter.get("softschema"), context=f"softschema in {path}"
        )
        contract = metadata.get("contract")
        envelope = metadata.get("envelope")
        schema_name = metadata.get("schema")
        status = metadata.get("status")
        if not all(
            isinstance(value, str) and value
            for value in (contract, envelope, schema_name, status)
        ):
            raise ValueError(f"{path}: incomplete softschema contract metadata")
        if expected_contract is None:
            expected_contract = contract
            expected_envelope = envelope
            expected_schema_name = schema_name
        elif contract != expected_contract:
            raise ValueError(
                f"mixed soft-schema contracts: {expected_contract!r} and {contract!r}"
            )
        elif envelope != expected_envelope or schema_name != expected_schema_name:
            raise ValueError(
                f"{path}: schema or envelope differs from the directory contract"
            )

        payload = _mapping(frontmatter.get(envelope), context=f"{envelope} in {path}")
        statuses[str(status or "unspecified")] += 1
        artifacts.append((path, metadata, payload))

    if not artifacts:
        raise ValueError(f"no artifacts matching {pattern!r} in {directory}")
    assert expected_contract is not None
    assert expected_envelope is not None
    assert expected_schema_name is not None

    schema_path = artifacts[0][0].parent / expected_schema_name
    schema = _mapping(
        read_yaml_file(schema_path), context=f"compiled schema {schema_path}"
    )
    schema_contract = _mapping(
        schema.get("x-softschema"), context=f"x-softschema in {schema_path}"
    ).get("contract")
    if schema_contract != expected_contract:
        raise ValueError(
            f"schema contract {schema_contract!r} does not match artifacts {expected_contract!r}"
        )

    descriptors = [_column_descriptor(schema, column) for column in columns]
    rows = []
    for path, _metadata, payload in artifacts:
        identity = payload.get("id") or payload.get("title") or path.stem
        rows.append(
            {
                "key": path.name,
                "identity": str(identity),
                "source": path.name,
                "href": _source_href(source_href_prefix, path.name),
                "values": {
                    column.path: _payload_value(payload, column.path)
                    for column in columns
                },
                "payload": payload,
            }
        )

    return {
        "contract": expected_contract,
        "envelope": expected_envelope,
        "status_counts": dict(sorted(statuses.items())),
        "schema": {
            "title": str(schema.get("title") or expected_envelope),
            "description": str(schema.get("description") or ""),
            "source": expected_schema_name,
        },
        "columns": descriptors,
        "row_count": len(rows),
        "rows": rows,
    }
