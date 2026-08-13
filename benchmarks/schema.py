"""Strict validation for performance scenario and result documents."""

from __future__ import annotations

import hashlib
import json
import math
import re
import string
from datetime import datetime
from pathlib import Path, PurePosixPath, PureWindowsPath
from typing import Any, Dict, List, Mapping, Optional, Sequence, Set


SCENARIO_SCHEMA = "fdu-performance-scenarios-v1"
RESULT_SCHEMA = "fdu-performance-result-v1"
MAX_JSON_BYTES = 8 * 1024 * 1024
MAX_RESULT_BYTES = 128 * 1024 * 1024

_SCENARIO_ID = re.compile(r"^[a-z0-9][a-z0-9._/-]*$")
_ENVIRONMENT_NAME = re.compile(r"^[A-Z_][A-Z0-9_]*$")
_FIELD_PATH = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*$")
_UTC_TIMESTAMP = re.compile(
    r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{6}Z$"
)
_PLACEHOLDERS = {
    "corpus_root",
    "executable",
    "manifest_path",
    "output_path",
    "run_root",
    "snapshot_path",
}
_JOBS = {
    "content-basic-cold",
    "content-basic-warm-fs",
    "content-binary-gate",
    "content-cache-hit",
    "content-churn",
    "content-disabled",
    "content-query",
    "selfhost-content",
    "watch-stream",
    "cli-files",
    "cli-human",
    "cli-json",
    "cli-summary",
    "concurrent-query",
    "delta-apply",
    "python-open-query",
    "query",
    "revalidate-churn",
    "revalidate-unchanged",
    "scan-index",
    "scan-producer",
    "snapshot-load",
    "snapshot-save",
}
_SNAPSHOT_STATES = {
    "absent",
    "compatible-changed",
    "compatible-unchanged",
    "corrupt",
    "incompatible-engine",
    "incompatible-scope",
}
_FILESYSTEM_CACHE_STATES = {
    "controlled-cold",
    "uncontrolled",
    "verified-warm",
}
_CORPUS_STATES = {"baseline", "mixed-1pct", "modify-1pct", "one-change"}
_RESOURCE_METRICS = {
    "input_blocks",
    "involuntary_context_switches",
    "major_faults",
    "minor_faults",
    "output_blocks",
    "peak_rss_bytes",
    "read_bytes",
    "retained_rss_bytes",
    "syscalls",
    "system_cpu_ns",
    "user_cpu_ns",
    "voluntary_context_switches",
    "write_bytes",
}


class SchemaError(ValueError):
    """Raised when performance evidence does not match its declared schema."""


def load_scenario_set(path: Path) -> Dict[str, Any]:
    """Read and strictly validate one scenario-set document."""
    document = _load_json(path)
    validate_scenario_set(document)
    return document


def validate_scenario_set(document: Any) -> None:
    """Validate a scenario-set document without applying defaults."""
    scenario_set = _object(
        document,
        required={"scenarios", "schema"},
        optional=set(),
        path="scenario set",
    )
    if scenario_set["schema"] != SCENARIO_SCHEMA:
        raise SchemaError(
            f"unsupported scenario schema: {scenario_set['schema']!r}"
        )
    scenarios = scenario_set["scenarios"]
    if not isinstance(scenarios, list) or not scenarios:
        raise SchemaError("scenario set.scenarios must be a non-empty array")
    seen: Set[str] = set()
    for index, value in enumerate(scenarios):
        scenario_id = _validate_scenario(value, f"scenarios[{index}]")
        if scenario_id in seen:
            raise SchemaError(f"duplicate scenario id: {scenario_id!r}")
        seen.add(scenario_id)


def load_result(path: Path) -> Dict[str, Any]:
    """Read and strictly validate one immutable result document."""
    document = _load_json(path, maximum_bytes=MAX_RESULT_BYTES)
    validate_result_document(document)
    return document


def validate_result_document(document: Any) -> None:
    """Validate a complete performance result and its internal cross-references."""
    result = _object(
        document,
        required={
            "executables",
            "host_class",
            "harness",
            "identity",
            "invocation_order",
            "method",
            "result_hash",
            "scenario_definitions",
            "schema",
            "source",
            "trials",
        },
        optional=set(),
        path="result",
    )
    if result["schema"] != RESULT_SCHEMA:
        raise SchemaError(f"unsupported result schema: {result['schema']!r}")
    _sha256(result["result_hash"], "result.result_hash")
    if result["result_hash"] != result_document_hash(result):
        raise SchemaError("result.result_hash does not match the document")
    _validate_result_identity(result["identity"])
    _validate_source(result["source"])
    _validate_host_class(result["host_class"])
    _validate_harness_identity(result["harness"])
    _validate_executable_identities(result["executables"])
    _validate_result_method(result["method"])
    scenario_definitions = result["scenario_definitions"]
    validate_scenario_set(
        {"scenarios": scenario_definitions, "schema": SCENARIO_SCHEMA}
    )
    scenarios_by_id = {
        scenario["id"]: scenario for scenario in scenario_definitions
    }
    scenario_ids = set(scenarios_by_id)
    invocation_order = _validate_invocation_order(
        result["invocation_order"], scenario_ids
    )
    expected_order = build_invocation_order(
        scenario_definitions, result["method"]["order_seed"]
    )
    if invocation_order != expected_order:
        raise SchemaError("result.invocation_order does not match its declared method")
    trials = result["trials"]
    if not isinstance(trials, list) or len(trials) != len(invocation_order):
        raise SchemaError("result.trials must align one-to-one with invocation_order")
    for index, trial in enumerate(trials):
        invocation = invocation_order[index]
        _validate_trial(
            trial,
            invocation,
            scenarios_by_id[invocation["scenario_id"]],
            f"trials[{index}]",
        )


def result_document_hash(document: Mapping[str, Any]) -> str:
    """Return the canonical hash used to make a result self-authenticating."""
    unhashed = {key: value for key, value in document.items() if key != "result_hash"}
    encoded = json.dumps(
        unhashed, ensure_ascii=True, separators=(",", ":"), sort_keys=True
    ).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def build_invocation_order(
    scenarios: Sequence[Mapping[str, Any]], order_seed: str
) -> List[Dict[str, Any]]:
    """Build the stable paired round order recorded by result documents."""
    schedule: List[Dict[str, Any]] = []
    groups: Dict[str, List[Mapping[str, Any]]] = {}
    for scenario in scenarios:
        order_group = scenario["method"]["order_group"]
        group = order_group or f"ungrouped:{scenario['id']}"
        groups.setdefault(group, []).append(scenario)
    ordinal = 0
    for sample_kind, count_field in (("warmup", "warmups"), ("timed", "trials")):
        for group in sorted(groups):
            members = groups[group]
            rounds = max(member["method"][count_field] for member in members)
            for sample_index in range(rounds):
                active = [
                    member
                    for member in members
                    if sample_index < member["method"][count_field]
                ]
                active.sort(
                    key=lambda member: (
                        _order_key(
                            order_seed,
                            group,
                            sample_kind,
                            sample_index,
                            member["id"],
                        ),
                        member["id"],
                    )
                )
                for member in active:
                    schedule.append(
                        {
                            "ordinal": ordinal,
                            "sample_index": sample_index,
                            "sample_kind": sample_kind,
                            "scenario_id": member["id"],
                        }
                    )
                    ordinal += 1
    return schedule


def _validate_scenario(value: Any, path: str) -> str:
    scenario = _object(
        value,
        required={
            "adapter",
            "command",
            "corpus",
            "filesystem_cache_state",
            "id",
            "job",
            "method",
            "output_sink",
            "preparation",
            "process_state",
            "snapshot_state",
            "validation",
        },
        optional=set(),
        path=path,
    )
    scenario_id = _identifier(scenario["id"], f"{path}.id")
    _identifier(scenario["adapter"], f"{path}.adapter")
    _choice(scenario["job"], _JOBS, f"{path}.job")
    snapshot_state = _choice(
        scenario["snapshot_state"], _SNAPSHOT_STATES, f"{path}.snapshot_state"
    )
    filesystem_cache_state = _choice(
        scenario["filesystem_cache_state"],
        _FILESYSTEM_CACHE_STATES,
        f"{path}.filesystem_cache_state",
    )
    _choice(
        scenario["process_state"],
        {"new-process", "same-process"},
        f"{path}.process_state",
    )
    _choice(
        scenario["output_sink"],
        {"output-digest", "output-file"},
        f"{path}.output_sink",
    )
    corpus_state = _validate_corpus(scenario["corpus"], f"{path}.corpus")
    _validate_command(scenario["command"], f"{path}.command")
    preparation = _validate_preparation(
        scenario["preparation"], f"{path}.preparation"
    )
    _validate_preparation_states(
        snapshot_state,
        filesystem_cache_state,
        corpus_state,
        preparation,
        path,
    )
    _validate_job_state(
        scenario["job"],
        scenario["process_state"],
        snapshot_state,
        corpus_state,
        path,
    )
    _validate_method(scenario["method"], f"{path}.method")
    _validate_validation(scenario["validation"], f"{path}.validation")
    return scenario_id


def _validate_corpus(value: Any, path: str) -> str:
    corpus = _object(
        value,
        required={"recipe_id", "seed", "state", "target_entries"},
        optional=set(),
        path=path,
    )
    _identifier(corpus["recipe_id"], f"{path}.recipe_id")
    target = _integer(corpus["target_entries"], f"{path}.target_entries", minimum=1)
    if target > 10_000_000:
        raise SchemaError(f"{path}.target_entries exceeds the safety limit")
    seed = corpus["seed"]
    if seed is not None and (
        not isinstance(seed, str) or not seed or len(seed) > 200
    ):
        raise SchemaError(f"{path}.seed must be null or a non-empty short string")
    return _choice(corpus["state"], _CORPUS_STATES, f"{path}.state")


def _validate_command(value: Any, path: str) -> None:
    command = _object(
        value,
        required={"argv", "environment"},
        optional=set(),
        path=path,
    )
    _validate_argv(command["argv"], f"{path}.argv", allow_null=False)
    environment = _object(
        command["environment"],
        required={"set", "unset"},
        optional=set(),
        path=f"{path}.environment",
    )
    set_values = environment["set"]
    if not isinstance(set_values, dict):
        raise SchemaError(f"{path}.environment.set must be an object")
    for name, setting in set_values.items():
        _environment_name(name, f"{path}.environment.set")
        _validate_template(setting, f"{path}.environment.set[{name!r}]")
    unset_values = environment["unset"]
    if not isinstance(unset_values, list):
        raise SchemaError(f"{path}.environment.unset must be an array")
    unset_names: Set[str] = set()
    for index, name in enumerate(unset_values):
        normalized = _environment_name(name, f"{path}.environment.unset[{index}]")
        if normalized in unset_names:
            raise SchemaError(f"{path}.environment.unset contains a duplicate")
        unset_names.add(normalized)
    overlap = set(set_values) & unset_names
    if overlap:
        raise SchemaError(
            f"{path}.environment names cannot be both set and unset: {sorted(overlap)}"
        )


def _validate_preparation(value: Any, path: str) -> Dict[str, Any]:
    preparation = _object(
        value,
        required={"filesystem_cache_argv", "snapshot_argv"},
        optional=set(),
        path=path,
    )
    _validate_argv(
        preparation["filesystem_cache_argv"],
        f"{path}.filesystem_cache_argv",
        allow_null=True,
    )
    _validate_argv(
        preparation["snapshot_argv"],
        f"{path}.snapshot_argv",
        allow_null=True,
    )
    return preparation


def _validate_preparation_states(
    snapshot_state: str,
    filesystem_cache_state: str,
    corpus_state: str,
    preparation: Mapping[str, Any],
    path: str,
) -> None:
    snapshot_argv = preparation["snapshot_argv"]
    if snapshot_state in {
        "compatible-changed",
        "compatible-unchanged",
        "incompatible-engine",
        "incompatible-scope",
    } and snapshot_argv is None:
        raise SchemaError(f"{path}.preparation.snapshot_argv is required for this state")
    if snapshot_state in {"absent", "corrupt"} and snapshot_argv is not None:
        raise SchemaError(f"{path}.preparation.snapshot_argv must be null for this state")
    if snapshot_state == "compatible-changed" and corpus_state == "baseline":
        raise SchemaError(f"{path}.corpus.state must declare churn for compatible-changed")
    cache_argv = preparation["filesystem_cache_argv"]
    if filesystem_cache_state == "uncontrolled" and cache_argv is not None:
        raise SchemaError(
            f"{path}.preparation.filesystem_cache_argv must be null when uncontrolled"
        )
    if filesystem_cache_state != "uncontrolled" and cache_argv is None:
        raise SchemaError(
            f"{path}.preparation.filesystem_cache_argv is required for "
            f"{filesystem_cache_state}"
        )


def _validate_job_state(
    job: str,
    process_state: str,
    snapshot_state: str,
    corpus_state: str,
    path: str,
) -> None:
    if (
        job in {"cli-human", "cli-json", "cli-summary", "cli-files", "watch-stream", "python-open-query"}
        and process_state != "new-process"
    ):
        raise SchemaError(f"{path}.process_state must be new-process for {job}")
    if job == "revalidate-unchanged" and (
        snapshot_state != "compatible-unchanged" or corpus_state != "baseline"
    ):
        raise SchemaError(
            f"{path} revalidate-unchanged requires a compatible unchanged snapshot"
        )
    if job == "revalidate-churn" and (
        snapshot_state != "compatible-changed" or corpus_state == "baseline"
    ):
        raise SchemaError(
            f"{path} revalidate-churn requires compatible-changed plus churn"
        )


def _validate_method(value: Any, path: str) -> None:
    method = _object(
        value,
        required={
            "order_group",
            "required_resources",
            "timeout_seconds",
            "trials",
            "warmups",
        },
        optional=set(),
        path=path,
    )
    _integer(method["warmups"], f"{path}.warmups", minimum=0, maximum=100)
    _integer(method["trials"], f"{path}.trials", minimum=1, maximum=1_000)
    timeout = method["timeout_seconds"]
    if (
        not isinstance(timeout, (int, float))
        or isinstance(timeout, bool)
        or not math.isfinite(timeout)
        or timeout < 0.01
        or timeout > 3_600
    ):
        raise SchemaError(f"{path}.timeout_seconds must be between 0.01 and 3600")
    order_group = method["order_group"]
    if order_group is not None:
        _identifier(order_group, f"{path}.order_group")
    required_resources = method["required_resources"]
    if (
        not isinstance(required_resources, list)
        or any(not isinstance(resource, str) for resource in required_resources)
        or len(required_resources) != len(set(required_resources))
    ):
        raise SchemaError(f"{path}.required_resources must be a unique array")
    for resource in required_resources:
        _choice(resource, _RESOURCE_METRICS, f"{path}.required_resources")


def _validate_validation(value: Any, path: str) -> None:
    validation = _object(
        value,
        required={
            "allowed_exit_codes",
            "corpus",
            "snapshot",
            "stderr",
            "stdout_json",
            "stdout_nonempty",
            "stdout_sha256",
        },
        optional=set(),
        path=path,
    )
    exit_codes = validation["allowed_exit_codes"]
    if not isinstance(exit_codes, list) or not exit_codes:
        raise SchemaError(f"{path}.allowed_exit_codes must be a non-empty array")
    normalized_codes = [
        _integer(code, f"{path}.allowed_exit_codes", minimum=0, maximum=255)
        for code in exit_codes
    ]
    if len(normalized_codes) != len(set(normalized_codes)):
        raise SchemaError(f"{path}.allowed_exit_codes contains duplicates")
    _choice(validation["corpus"], {"declared", "unchanged"}, f"{path}.corpus")
    _choice(validation["snapshot"], {"absent", "any", "exists"}, f"{path}.snapshot")
    _choice(validation["stderr"], {"empty", "ignore"}, f"{path}.stderr")
    if not isinstance(validation["stdout_nonempty"], bool):
        raise SchemaError(f"{path}.stdout_nonempty must be boolean")
    stdout_sha256 = validation["stdout_sha256"]
    if stdout_sha256 is not None:
        _sha256(stdout_sha256, f"{path}.stdout_sha256")
    if (
        not validation["stdout_nonempty"]
        and stdout_sha256 is None
        and validation["stdout_json"] is None
    ):
        raise SchemaError(f"{path} must declare at least one stdout validation")
    if validation["stdout_json"] is None:
        return
    stdout_json = _object(
        validation["stdout_json"],
        required={"equals", "matches_manifest", "record"},
        optional=set(),
        path=f"{path}.stdout_json",
    )
    equals = stdout_json["equals"]
    matches = stdout_json["matches_manifest"]
    if not isinstance(equals, dict) or not isinstance(matches, dict):
        raise SchemaError(f"{path}.stdout_json mappings must be objects")
    for output_path, expected in equals.items():
        _field_path(output_path, f"{path}.stdout_json.equals")
        _validate_json_value(expected, f"{path}.stdout_json.equals[{output_path!r}]")
    for output_path, manifest_path in matches.items():
        _field_path(output_path, f"{path}.stdout_json.matches_manifest")
        _field_path(manifest_path, f"{path}.stdout_json.matches_manifest")
    record = stdout_json["record"]
    if (
        not isinstance(record, list)
        or any(not isinstance(output_path, str) for output_path in record)
        or len(record) != len(set(record))
    ):
        raise SchemaError(f"{path}.stdout_json.record must be a unique array")
    if len(record) > 64:
        raise SchemaError(f"{path}.stdout_json.record exceeds 64 fields")
    for output_path in record:
        _field_path(output_path, f"{path}.stdout_json.record")
    if not equals and not matches:
        raise SchemaError(f"{path}.stdout_json must declare at least one assertion")


def _validate_argv(value: Any, path: str, *, allow_null: bool) -> None:
    if value is None and allow_null:
        return
    if not isinstance(value, list) or not value:
        raise SchemaError(f"{path} must be a non-empty argument array")
    if value[0] != "{executable}":
        raise SchemaError(f"{path}[0] must be '{{executable}}'")
    formatter = string.Formatter()
    for index, argument in enumerate(value):
        _validate_template(argument, f"{path}[{index}]", formatter=formatter)


def _validate_template(
    value: Any, path: str, *, formatter: Optional[string.Formatter] = None
) -> None:
    if not isinstance(value, str) or "\0" in value or len(value) > 4_096:
        raise SchemaError(f"{path} must be a bounded string")
    if PurePosixPath(value).is_absolute() or PureWindowsPath(value).is_absolute():
        raise SchemaError(
            f"{path} must use a run placeholder instead of an absolute path"
        )
    active_formatter = formatter or string.Formatter()
    try:
        parsed = list(active_formatter.parse(value))
    except ValueError as error:
        raise SchemaError(f"{path} has malformed braces") from error
    for _literal, placeholder, format_spec, conversion in parsed:
        if placeholder is None:
            continue
        if placeholder not in _PLACEHOLDERS:
            raise SchemaError(f"{path} has unknown placeholder {placeholder!r}")
        if format_spec or conversion:
            raise SchemaError(f"{path} may not format placeholders")


def _object(
    value: Any,
    *,
    required: Set[str],
    optional: Set[str],
    path: str,
) -> Dict[str, Any]:
    if not isinstance(value, dict):
        raise SchemaError(f"{path} must be an object")
    unknown = set(value) - required - optional
    missing = required - set(value)
    if unknown or missing:
        raise SchemaError(
            f"{path} has unknown fields {sorted(unknown)} or missing fields {sorted(missing)}"
        )
    return value


def _load_json(
    path: Path, *, maximum_bytes: int = MAX_JSON_BYTES
) -> Dict[str, Any]:
    try:
        if path.stat().st_size > maximum_bytes:
            raise SchemaError(
                f"JSON input {path.name} exceeds the {maximum_bytes}-byte safety limit"
            )
        with path.open("r", encoding="utf-8") as input_file:
            value = json.load(input_file, parse_constant=_reject_json_constant)
    except (OSError, json.JSONDecodeError) as error:
        raise SchemaError(f"cannot read valid JSON from {path.name}: {error}") from error
    if not isinstance(value, dict):
        raise SchemaError(f"{path.name} must contain a JSON object")
    return value


def _validate_result_identity(value: Any) -> None:
    identity = _object(
        value,
        required={"finished_at_utc", "run_id", "started_at_utc"},
        optional=set(),
        path="result.identity",
    )
    run_id = identity["run_id"]
    if (
        not isinstance(run_id, str)
        or len(run_id) != 32
        or any(character not in "0123456789abcdef" for character in run_id)
    ):
        raise SchemaError("result.identity.run_id must be 32 lowercase hex characters")
    timestamps: Dict[str, datetime] = {}
    for field in ("started_at_utc", "finished_at_utc"):
        timestamp = identity[field]
        if not isinstance(timestamp, str) or not _UTC_TIMESTAMP.fullmatch(timestamp):
            raise SchemaError(f"result.identity.{field} must be a UTC timestamp")
        try:
            timestamps[field] = datetime.fromisoformat(timestamp[:-1] + "+00:00")
        except ValueError as error:
            raise SchemaError(
                f"result.identity.{field} must be a valid UTC timestamp"
            ) from error
    if timestamps["finished_at_utc"] < timestamps["started_at_utc"]:
        raise SchemaError("result.identity finished before it started")


def _validate_source(value: Any) -> None:
    source = _object(
        value,
        required={"dirty", "reason", "revision"},
        optional=set(),
        path="result.source",
    )
    if source["dirty"] is not None and not isinstance(source["dirty"], bool):
        raise SchemaError("result.source.dirty must be boolean or null")
    if source["reason"] is not None and not isinstance(source["reason"], str):
        raise SchemaError("result.source.reason must be string or null")
    revision = source["revision"]
    if revision is not None:
        _sha256(revision, "result.source.revision", allow_sha1=True)
    complete = revision is not None and source["dirty"] is not None
    if complete != (source["reason"] is None):
        raise SchemaError("result.source.reason disagrees with source availability")


def _validate_host_class(value: Any) -> None:
    host = _object(
        value,
        required={
            "architecture",
            "cpu_logical",
            "filesystem",
            "filesystem_reason",
            "host_class",
            "kernel",
            "memory_bytes",
            "memory_reason",
            "operating_system",
        },
        optional=set(),
        path="result.host_class",
    )
    for field in ("architecture", "host_class", "kernel", "operating_system"):
        if not isinstance(host[field], str) or not host[field]:
            raise SchemaError(f"result.host_class.{field} must be a non-empty string")
    if host["cpu_logical"] is not None:
        _integer(host["cpu_logical"], "result.host_class.cpu_logical", minimum=1)
    if host["memory_bytes"] is not None:
        _integer(host["memory_bytes"], "result.host_class.memory_bytes", minimum=1)
    for field in ("filesystem", "filesystem_reason", "memory_reason"):
        if host[field] is not None and not isinstance(host[field], str):
            raise SchemaError(f"result.host_class.{field} must be string or null")
    if (host["filesystem"] is None) != (host["filesystem_reason"] is not None):
        raise SchemaError("result.host_class filesystem reason is inconsistent")
    if (host["memory_bytes"] is None) != (host["memory_reason"] is not None):
        raise SchemaError("result.host_class memory reason is inconsistent")


def _validate_executable_identities(value: Any) -> None:
    if not isinstance(value, dict) or not value:
        raise SchemaError("result.executables must be a non-empty object")
    for adapter, raw_identity in value.items():
        _identifier(adapter, "result.executables")
        identity = _object(
            raw_identity,
            required={"components", "identity_hash"},
            optional=set(),
            path=f"result.executables[{adapter!r}]",
        )
        _sha256(identity["identity_hash"], f"result.executables[{adapter!r}].identity_hash")
        components = identity["components"]
        if not isinstance(components, list) or not components:
            raise SchemaError(f"result.executables[{adapter!r}].components must be an array")
        names: Set[str] = set()
        for index, raw_component in enumerate(components):
            component = _object(
                raw_component,
                required={"name", "sha256"},
                optional=set(),
                path=f"result.executables[{adapter!r}].components[{index}]",
            )
            name = component["name"]
            if (
                not isinstance(name, str)
                or not name
                or Path(name).name != name
                or name in {".", ".."}
            ):
                raise SchemaError("executable component name must be a safe basename")
            if name in names:
                raise SchemaError("executable component names must be unique")
            names.add(name)
            _sha256(component["sha256"], "executable component sha256")
        expected_identity = _canonical_hash({"components": components})
        if identity["identity_hash"] != expected_identity:
            raise SchemaError(
                f"result.executables[{adapter!r}].identity_hash does not match"
            )


def _validate_harness_identity(value: Any) -> None:
    harness = _object(
        value,
        required={
            "components",
            "identity_hash",
            "python_implementation",
            "python_version",
        },
        optional=set(),
        path="result.harness",
    )
    for field in ("python_implementation", "python_version"):
        if not isinstance(harness[field], str) or not harness[field]:
            raise SchemaError(f"result.harness.{field} must be a non-empty string")
    components = harness["components"]
    if not isinstance(components, list) or not components:
        raise SchemaError("result.harness.components must be a non-empty array")
    names: Set[str] = set()
    for index, raw_component in enumerate(components):
        component = _object(
            raw_component,
            required={"name", "sha256"},
            optional=set(),
            path=f"result.harness.components[{index}]",
        )
        name = component["name"]
        if (
            not isinstance(name, str)
            or not name
            or Path(name).name != name
            or name in {".", ".."}
            or name in names
        ):
            raise SchemaError("harness component names must be unique safe basenames")
        names.add(name)
        _sha256(component["sha256"], "harness component sha256")
    _sha256(harness["identity_hash"], "result.harness.identity_hash")
    expected = _canonical_hash(
        {
            "components": components,
            "python_implementation": harness["python_implementation"],
            "python_version": harness["python_version"],
        }
    )
    if harness["identity_hash"] != expected:
        raise SchemaError("result.harness.identity_hash does not match")


def _validate_result_method(value: Any) -> None:
    method = _object(
        value,
        required={"order_algorithm", "order_seed"},
        optional=set(),
        path="result.method",
    )
    if method["order_algorithm"] != "sha256-round-robin-v1":
        raise SchemaError("result.method.order_algorithm is unsupported")
    if (
        not isinstance(method["order_seed"], str)
        or not method["order_seed"]
        or len(method["order_seed"]) > 200
    ):
        raise SchemaError("result.method.order_seed must be a non-empty short string")


def _validate_invocation_order(
    value: Any, scenario_ids: Set[str]
) -> List[Dict[str, Any]]:
    if not isinstance(value, list) or not value:
        raise SchemaError("result.invocation_order must be a non-empty array")
    for index, raw_invocation in enumerate(value):
        invocation = _object(
            raw_invocation,
            required={"ordinal", "sample_index", "sample_kind", "scenario_id"},
            optional=set(),
            path=f"result.invocation_order[{index}]",
        )
        if invocation["ordinal"] != index:
            raise SchemaError("result invocation ordinals must be contiguous")
        _integer(
            invocation["sample_index"],
            f"result.invocation_order[{index}].sample_index",
            minimum=0,
        )
        _choice(
            invocation["sample_kind"],
            {"timed", "warmup"},
            f"result.invocation_order[{index}].sample_kind",
        )
        if invocation["scenario_id"] not in scenario_ids:
            raise SchemaError("result invocation references an unknown scenario")
    return value


def _validate_trial(
    value: Any,
    invocation: Mapping[str, Any],
    scenario: Mapping[str, Any],
    path: str,
) -> None:
    trial = _object(
        value,
        required={
            "argv",
            "corpus",
            "environment",
            "filesystem_cache_state",
            "ordinal",
            "output",
            "preparation",
            "probe_metrics",
            "process",
            "resources",
            "sample_index",
            "sample_kind",
            "scenario_id",
            "snapshot_state",
            "timing",
            "validation",
        },
        optional=set(),
        path=path,
    )
    for field in ("ordinal", "sample_index", "sample_kind", "scenario_id"):
        if trial[field] != invocation[field]:
            raise SchemaError(f"{path}.{field} disagrees with invocation_order")
    _validate_argv(trial["argv"], f"{path}.argv", allow_null=False)
    if trial["argv"] != scenario["command"]["argv"]:
        raise SchemaError(f"{path}.argv disagrees with its scenario definition")
    _choice(
        trial["filesystem_cache_state"],
        _FILESYSTEM_CACHE_STATES,
        f"{path}.filesystem_cache_state",
    )
    _choice(trial["snapshot_state"], _SNAPSHOT_STATES, f"{path}.snapshot_state")
    if trial["filesystem_cache_state"] != scenario["filesystem_cache_state"]:
        raise SchemaError(f"{path}.filesystem_cache_state disagrees with its scenario")
    if trial["snapshot_state"] != scenario["snapshot_state"]:
        raise SchemaError(f"{path}.snapshot_state disagrees with its scenario")
    _validate_trial_corpus(trial["corpus"], scenario, f"{path}.corpus")
    _validate_trial_environment(trial["environment"], f"{path}.environment")
    _validate_trial_output(trial["output"], f"{path}.output")
    _validate_trial_preparation(trial["preparation"], f"{path}.preparation")
    _validate_trial_probe_metrics(trial["probe_metrics"], f"{path}.probe_metrics")
    _validate_trial_process(trial["process"], f"{path}.process")
    _validate_trial_resources(trial["resources"], f"{path}.resources")
    _validate_trial_timing(trial["timing"], f"{path}.timing")
    _validate_trial_validation(trial["validation"], f"{path}.validation")
    _validate_trial_contract(trial, scenario, path)


def _validate_trial_corpus(
    value: Any, scenario: Mapping[str, Any], path: str
) -> None:
    if value is None:
        return
    corpus = _object(
        value,
        required={
            "counts",
            "manifest_hash",
            "recipe_hash",
            "recipe_id",
            "semantic_digest",
            "state",
        },
        optional=set(),
        path=path,
    )
    counts = _object(
        corpus["counts"],
        required={
            "directories",
            "files",
            "hardlink_groups",
            "other",
            "symlinks",
            "total",
        },
        optional=set(),
        path=f"{path}.counts",
    )
    for field in ("directories", "files", "hardlink_groups", "other", "symlinks"):
        _integer(counts[field], f"{path}.counts.{field}", minimum=0)
    _integer(counts["total"], f"{path}.counts.total", minimum=1)
    kind_total = (
        counts["directories"]
        + counts["files"]
        + counts["other"]
        + counts["symlinks"]
    )
    if kind_total != counts["total"]:
        raise SchemaError(f"{path}.counts.total disagrees with kind counts")
    state = _object(
        corpus["state"],
        required={"id", "transition_index"},
        optional=set(),
        path=f"{path}.state",
    )
    _choice(state["id"], _CORPUS_STATES, f"{path}.state.id")
    _integer(state["transition_index"], f"{path}.state.transition_index", minimum=0)
    for field in ("manifest_hash", "recipe_hash", "semantic_digest"):
        _sha256(corpus[field], f"{path}.{field}")
    _identifier(corpus["recipe_id"], f"{path}.recipe_id")
    if corpus["recipe_id"] != scenario["corpus"]["recipe_id"]:
        raise SchemaError(f"{path}.recipe_id disagrees with its scenario")
    if state["id"] != scenario["corpus"]["state"]:
        raise SchemaError(f"{path}.state.id disagrees with its scenario")


def _validate_trial_preparation(value: Any, path: str) -> None:
    preparation = _object(
        value,
        required={
            "base_cache_key",
            "base_created",
            "base_generation_ns",
            "cloned_files",
            "copied_bytes",
            "copied_files",
            "copy_strategy",
            "directories",
            "hardlinks",
            "materialization_ns",
            "source_verification_ns",
            "strategy_probe_ns",
            "symlinks",
            "total_ns",
        },
        optional=set(),
        path=path,
    )
    cache_key = preparation["base_cache_key"]
    if cache_key is not None:
        _sha256(cache_key, f"{path}.base_cache_key")
    if not isinstance(preparation["base_created"], bool):
        raise SchemaError(f"{path}.base_created must be boolean")
    for field in (
        "base_generation_ns",
        "materialization_ns",
        "source_verification_ns",
        "strategy_probe_ns",
    ):
        _nullable_nonnegative_integer(preparation[field], f"{path}.{field}")
    for field in (
        "cloned_files",
        "copied_bytes",
        "copied_files",
        "directories",
        "hardlinks",
        "symlinks",
        "total_ns",
    ):
        _integer(preparation[field], f"{path}.{field}", minimum=0)
    strategy = preparation["copy_strategy"]
    if strategy is not None and (
        not isinstance(strategy, str) or not strategy or len(strategy) > 100
    ):
        raise SchemaError(f"{path}.copy_strategy must be a bounded string or null")
    if preparation["base_created"] != (
        preparation["base_generation_ns"] is not None
    ):
        raise SchemaError(f"{path}.base_created disagrees with base_generation_ns")
    materialized = cache_key is not None
    if preparation["base_created"] and not materialized:
        raise SchemaError(f"{path}.base_created requires a base_cache_key")
    for field in (
        "copy_strategy",
        "materialization_ns",
        "source_verification_ns",
        "strategy_probe_ns",
    ):
        if (preparation[field] is not None) != materialized:
            raise SchemaError(f"{path}.{field} disagrees with base_cache_key")
    if not materialized and any(
        preparation[field] != 0
        for field in (
            "cloned_files",
            "copied_bytes",
            "copied_files",
            "directories",
            "hardlinks",
            "symlinks",
        )
    ):
        raise SchemaError(f"{path} has copy counts without a materialized corpus")
    if strategy == "bounded-copy-v1" and preparation["cloned_files"] != 0:
        raise SchemaError(f"{path}.cloned_files disagrees with bounded-copy-v1")
    accounted_ns = sum(
        preparation[field] or 0
        for field in (
            "base_generation_ns",
            "materialization_ns",
            "source_verification_ns",
        )
    )
    if preparation["total_ns"] < accounted_ns:
        raise SchemaError(f"{path}.total_ns is smaller than its timed components")


def _validate_trial_environment(value: Any, path: str) -> None:
    environment = _object(
        value,
        required={"explicitly_unset", "set"},
        optional=set(),
        path=path,
    )
    if not isinstance(environment["set"], dict):
        raise SchemaError(f"{path}.set must be an object")
    for name, setting in environment["set"].items():
        _environment_name(name, f"{path}.set")
        if not isinstance(setting, str) or "\0" in setting:
            raise SchemaError(f"{path}.set values must be strings")
        if PurePosixPath(setting).is_absolute() or PureWindowsPath(setting).is_absolute():
            raise SchemaError(f"{path}.set values may not contain absolute paths")
    unset = environment["explicitly_unset"]
    if not isinstance(unset, list) or len(unset) != len(set(unset)):
        raise SchemaError(f"{path}.explicitly_unset must be a unique array")
    for name in unset:
        _environment_name(name, f"{path}.explicitly_unset")


def _validate_trial_output(value: Any, path: str) -> None:
    output = _object(
        value,
        required={
            "artifact",
            "stderr_bytes",
            "stderr_sha256",
            "stdout_bytes",
            "stdout_sha256",
        },
        optional=set(),
        path=path,
    )
    for field in ("stderr_bytes", "stdout_bytes"):
        _integer(output[field], f"{path}.{field}", minimum=0)
    for field in ("stderr_sha256", "stdout_sha256"):
        if output[field] is not None:
            _sha256(output[field], f"{path}.{field}")
    artifact = output["artifact"]
    if artifact is not None and (
        not isinstance(artifact, str)
        or Path(artifact).name != artifact
        or artifact in {".", ".."}
    ):
        raise SchemaError(f"{path}.artifact must be a safe basename or null")


def _validate_trial_process(value: Any, path: str) -> None:
    process = _object(
        value,
        required={"exit_code", "signal", "spawn_error", "timed_out", "wait_error"},
        optional=set(),
        path=path,
    )
    _nullable_nonnegative_integer(process["exit_code"], f"{path}.exit_code")
    _nullable_nonnegative_integer(process["signal"], f"{path}.signal")
    if process["spawn_error"] is not None and not isinstance(process["spawn_error"], str):
        raise SchemaError(f"{path}.spawn_error must be string or null")
    if process["wait_error"] is not None and not isinstance(process["wait_error"], str):
        raise SchemaError(f"{path}.wait_error must be string or null")
    if not isinstance(process["timed_out"], bool):
        raise SchemaError(f"{path}.timed_out must be boolean")


def _validate_trial_resources(value: Any, path: str) -> None:
    resources = _object(
        value,
        required={"collector", "unavailable_reasons", *_RESOURCE_METRICS},
        optional=set(),
        path=path,
    )
    collector = resources["collector"]
    if not isinstance(collector, str) or not collector or len(collector) > 100:
        raise SchemaError(f"{path}.collector must be a non-empty short string")
    for field in _RESOURCE_METRICS:
        _nullable_nonnegative_integer(resources[field], f"{path}.{field}")
    unavailable = resources["unavailable_reasons"]
    if not isinstance(unavailable, dict):
        raise SchemaError(f"{path}.unavailable_reasons must be an object")
    expected_unavailable = {
        field for field in _RESOURCE_METRICS if resources[field] is None
    }
    if set(unavailable) != expected_unavailable:
        raise SchemaError(
            f"{path}.unavailable_reasons must name exactly the null metrics"
        )
    if any(not isinstance(reason, str) or not reason for reason in unavailable.values()):
        raise SchemaError(
            f"{path}.unavailable_reasons values must be non-empty strings"
        )


def _validate_trial_probe_metrics(value: Any, path: str) -> None:
    if not isinstance(value, dict) or len(value) > 64:
        raise SchemaError(f"{path} must be an object with at most 64 fields")
    for field, metric in value.items():
        _field_path(field, path)
        if isinstance(metric, float) and not math.isfinite(metric):
            raise SchemaError(f"{path}[{field!r}] must be finite")
        if metric is not None and not isinstance(metric, (bool, int, float, str)):
            raise SchemaError(f"{path}[{field!r}] must be a JSON scalar")


def _validate_trial_timing(value: Any, path: str) -> None:
    timing = _object(
        value,
        required={"first_output_ns", "wall_ns"},
        optional=set(),
        path=path,
    )
    _nullable_nonnegative_integer(
        timing["first_output_ns"], f"{path}.first_output_ns"
    )
    _nullable_nonnegative_integer(timing["wall_ns"], f"{path}.wall_ns")
    if (
        timing["first_output_ns"] is not None
        and timing["wall_ns"] is not None
        and timing["first_output_ns"] > timing["wall_ns"]
    ):
        raise SchemaError(f"{path}.first_output_ns exceeds wall_ns")


def _validate_trial_validation(value: Any, path: str) -> None:
    validation = _object(
        value,
        required={"postcondition", "precondition", "reasons", "valid"},
        optional=set(),
        path=path,
    )
    for field in ("postcondition", "precondition", "valid"):
        if not isinstance(validation[field], bool):
            raise SchemaError(f"{path}.{field} must be boolean")
    reasons = validation["reasons"]
    if not isinstance(reasons, list) or any(
        not isinstance(reason, str) or not reason for reason in reasons
    ):
        raise SchemaError(f"{path}.reasons must be an array of non-empty strings")
    expected_valid = (
        validation["precondition"]
        and validation["postcondition"]
        and not validation["reasons"]
    )
    if validation["valid"] != expected_valid:
        raise SchemaError(f"{path}.valid disagrees with its validation evidence")


def _validate_trial_contract(
    trial: Mapping[str, Any], scenario: Mapping[str, Any], path: str
) -> None:
    validation = trial["validation"]
    if validation["precondition"] != (trial["corpus"] is not None):
        raise SchemaError(f"{path}.precondition disagrees with corpus evidence")
    if validation["postcondition"] and not validation["precondition"]:
        raise SchemaError(f"{path}.postcondition cannot pass without a precondition")
    if validation["precondition"] and trial["preparation"]["base_cache_key"] is None:
        raise SchemaError(f"{path}.precondition lacks corpus preparation evidence")

    expected_environment: Dict[str, str] = {
        "LANG": "C",
        "LC_ALL": "C",
        "TZ": "UTC",
    }
    declaration = scenario["command"]["environment"]
    expected_environment.update(declaration["set"])
    for name in declaration["unset"]:
        expected_environment.pop(name, None)
    actual_environment = dict(trial["environment"]["set"])
    actual_environment.pop("SYSTEMROOT", None)
    actual_environment.pop("WINDIR", None)
    if actual_environment != expected_environment:
        raise SchemaError(f"{path}.environment.set disagrees with its scenario")
    if trial["environment"]["explicitly_unset"] != sorted(declaration["unset"]):
        raise SchemaError(f"{path}.environment.explicitly_unset disagrees")

    output = trial["output"]
    timing = trial["timing"]
    if output["stdout_bytes"] > 0 and output["stdout_sha256"] is None:
        raise SchemaError(f"{path}.output.stdout_sha256 is missing")
    if output["stderr_bytes"] > 0 and output["stderr_sha256"] is None:
        raise SchemaError(f"{path}.output.stderr_sha256 is missing")
    if output["stdout_bytes"] == 0 and timing["first_output_ns"] is not None:
        raise SchemaError(f"{path}.timing.first_output_ns has no corresponding output")
    if output["stdout_bytes"] > 0 and timing["first_output_ns"] is None:
        raise SchemaError(f"{path}.timing.first_output_ns is missing")
    if scenario["output_sink"] == "output-digest" and output["artifact"] is not None:
        raise SchemaError(f"{path}.output.artifact is invalid for output-digest")

    if not validation["valid"]:
        return
    process = trial["process"]
    contract = scenario["validation"]
    if (
        process["spawn_error"] is not None
        or process["wait_error"] is not None
        or process["timed_out"]
    ):
        raise SchemaError(f"{path} cannot be valid after a process failure")
    if process["signal"] is not None:
        raise SchemaError(f"{path} cannot be valid after a signal")
    if process["exit_code"] not in contract["allowed_exit_codes"]:
        raise SchemaError(f"{path}.process.exit_code violates its scenario")
    if contract["stderr"] == "empty" and output["stderr_bytes"] != 0:
        raise SchemaError(f"{path}.output.stderr_bytes violates its scenario")
    if contract["stdout_nonempty"] and output["stdout_bytes"] == 0:
        raise SchemaError(f"{path}.output.stdout_bytes violates its scenario")
    expected_digest = contract["stdout_sha256"]
    if expected_digest is not None and output["stdout_sha256"] != expected_digest:
        raise SchemaError(f"{path}.output.stdout_sha256 violates its scenario")
    if scenario["output_sink"] == "output-file" and output["artifact"] is None:
        raise SchemaError(f"{path}.output.artifact is required for output-file")
    stdout_json = contract["stdout_json"]
    expected_probe_metrics = set(stdout_json["record"]) if stdout_json else set()
    if set(trial["probe_metrics"]) != expected_probe_metrics:
        raise SchemaError(f"{path}.probe_metrics disagrees with its scenario")
    for resource in scenario["method"]["required_resources"]:
        if trial["resources"][resource] is None:
            raise SchemaError(f"{path}.resources.{resource} is required")


def _nullable_nonnegative_integer(value: Any, path: str) -> None:
    if value is not None:
        _integer(value, path, minimum=0)


def _sha256(value: Any, path: str, *, allow_sha1: bool = False) -> str:
    allowed_lengths = {40, 64} if allow_sha1 else {64}
    if (
        not isinstance(value, str)
        or len(value) not in allowed_lengths
        or value != value.lower()
        or any(character not in "0123456789abcdef" for character in value)
    ):
        raise SchemaError(f"{path} must be a lowercase hexadecimal digest")
    return value


def _canonical_hash(value: Mapping[str, Any]) -> str:
    encoded = json.dumps(
        value, ensure_ascii=True, separators=(",", ":"), sort_keys=True
    ).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def _order_key(
    seed: str, group: str, kind: str, index: int, scenario_id: str
) -> bytes:
    value = f"{seed}\0{group}\0{kind}\0{index}\0{scenario_id}".encode("utf-8")
    return hashlib.sha256(value).digest()


def _identifier(value: Any, path: str) -> str:
    if not isinstance(value, str) or not _SCENARIO_ID.fullmatch(value):
        raise SchemaError(f"{path} must be a lowercase identifier")
    if any(component in {"", ".", ".."} for component in value.split("/")):
        raise SchemaError(f"{path} contains an unsafe path component")
    return value


def _environment_name(value: Any, path: str) -> str:
    if not isinstance(value, str) or not _ENVIRONMENT_NAME.fullmatch(value):
        raise SchemaError(f"{path} has an invalid environment variable name")
    return value


def _field_path(value: Any, path: str) -> str:
    if not isinstance(value, str) or not _FIELD_PATH.fullmatch(value):
        raise SchemaError(f"{path} has an invalid field path")
    return value


def _choice(value: Any, choices: Set[str], path: str) -> str:
    if not isinstance(value, str) or value not in choices:
        raise SchemaError(f"{path} must be one of: {', '.join(sorted(choices))}")
    return value


def _integer(
    value: Any,
    path: str,
    *,
    minimum: int,
    maximum: Optional[int] = None,
) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or value < minimum:
        raise SchemaError(f"{path} must be an integer at least {minimum}")
    if maximum is not None and value > maximum:
        raise SchemaError(f"{path} must be at most {maximum}")
    return value


def _validate_json_value(value: Any, path: str) -> None:
    if isinstance(value, float) and not math.isfinite(value):
        raise SchemaError(f"{path} must be a finite JSON number")
    if value is None or isinstance(value, (bool, int, float, str)):
        return
    if isinstance(value, list):
        for index, item in enumerate(value):
            _validate_json_value(item, f"{path}[{index}]")
        return
    if isinstance(value, dict):
        for key, item in value.items():
            if not isinstance(key, str):
                raise SchemaError(f"{path} contains a non-string object key")
            _validate_json_value(item, f"{path}.{key}")
        return
    raise SchemaError(f"{path} is not a JSON value")


def _reject_json_constant(value: str) -> None:
    raise json.JSONDecodeError(f"non-standard JSON constant {value!r}", value, 0)
