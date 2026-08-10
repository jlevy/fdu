"""Direct-argv state-machine runner for reproducible performance evidence."""

from __future__ import annotations

import hashlib
import json
import os
import platform
import shutil
import signal
import subprocess
import sys
import tempfile
import threading
import time
import uuid
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Mapping, Optional, Sequence, Tuple

from benchmarks.corpus import (
    CORPUS_NAME,
    MANIFEST_NAME,
    CorpusError,
    apply_transition,
    cleanup_run_directory,
    reserve_run_directory,
    verify_corpus,
)
from benchmarks.corpus_cache import CorpusBasePool
from benchmarks.schema import (
    RESULT_SCHEMA,
    SchemaError,
    build_invocation_order,
    validate_result_document,
    validate_scenario_set,
)


MAX_VALIDATION_OUTPUT_BYTES = 512 * 1024 * 1024
MAX_DIGEST_JSON_CAPTURE_BYTES = 16 * 1024 * 1024
_BASE_ENVIRONMENT = {"LANG": "C", "LC_ALL": "C", "TZ": "UTC"}
_CORPUS_TRANSITIONS = {
    "baseline": (),
    "one-change": ("one-change",),
    "modify-1pct": ("one-change", "modify-1pct"),
    "mixed-1pct": ("one-change", "modify-1pct", "mixed-1pct"),
}


class RunnerError(ValueError):
    """Raised when a run cannot honor the declared measurement contract."""


@dataclass(frozen=True)
class _CommandResult:
    wall_ns: Optional[int]
    exit_code: Optional[int]
    signal: Optional[int]
    timed_out: bool
    spawn_error: Optional[str]
    capture_error: Optional[str]
    stdout_path: Path
    stderr_path: Path
    stdout_bytes: int
    stderr_bytes: int
    stdout_sha256: str
    stderr_sha256: str
    first_output_ns: Optional[int]
    resources: Dict[str, Any]
    wait_error: Optional[str]


@dataclass
class _OutputDrain:
    """Bounded state shared with the stdout-draining thread."""

    capture: Optional[bytearray]
    capture_error: Optional[str] = None
    first_output_ns: Optional[int] = None
    stdout_bytes: int = 0
    stdout_sha256: Optional[str] = None


@dataclass
class _WaitState:
    """Result written by the one thread that owns child reaping."""

    error: Optional[str] = None
    return_code: Optional[int] = None
    usage: Optional[Any] = None


_RESOURCE_METRICS = (
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
)


def run_scenario_set(
    document: Dict[str, Any],
    *,
    executables: Mapping[str, Sequence[str]],
    work_directory: Path,
    output_directory: Path,
    order_seed: str,
    scenario_ids: Optional[Sequence[str]] = None,
) -> Dict[str, Any]:
    """Execute selected scenarios and atomically persist one immutable result set."""
    validate_scenario_set(document)
    if not isinstance(order_seed, str) or not order_seed or len(order_seed) > 200:
        raise RunnerError(
            "order_seed must be a non-empty string of at most 200 characters"
        )
    selected = _select_scenarios(document["scenarios"], scenario_ids)
    controlled_cold = [
        scenario["id"]
        for scenario in selected
        if scenario["filesystem_cache_state"] == "controlled-cold"
    ]
    if controlled_cold:
        raise RunnerError(
            "controlled-cold execution requires the dedicated-host protocol; "
            f"the portable runner cannot substantiate {controlled_cold}"
        )
    executable_identities = _validate_executables(selected, executables)
    work_root = _prepare_directory(work_directory, "work directory")
    result_root = _prepare_directory(output_directory, "output directory")
    run_id = uuid.uuid4().hex
    started_at = _utc_now()
    schedule = build_invocation_order(selected, order_seed)
    artifact_root = result_root / f"artifacts-{run_id}"
    artifact_root.mkdir()
    trials: List[Dict[str, Any]] = []
    scenarios_by_id = {scenario["id"]: scenario for scenario in selected}
    with CorpusBasePool(work_root) as corpus_bases:
        for invocation in schedule:
            scenario = scenarios_by_id[invocation["scenario_id"]]
            trial = _execute_invocation(
                scenario,
                invocation,
                executables[scenario["adapter"]],
                work_root,
                artifact_root,
                corpus_bases,
            )
            trials.append(trial)

    if not any(artifact_root.iterdir()):
        artifact_root.rmdir()
    result: Dict[str, Any] = {
        "executables": executable_identities,
        "host_class": _host_class(),
        "harness": _harness_identity(),
        "identity": {
            "finished_at_utc": _utc_now(),
            "run_id": run_id,
            "started_at_utc": started_at,
        },
        "invocation_order": schedule,
        "method": {
            "order_algorithm": "sha256-round-robin-v1",
            "order_seed": order_seed,
        },
        "scenario_definitions": selected,
        "schema": RESULT_SCHEMA,
        "source": _source_identity(),
        "trials": trials,
    }
    result["result_hash"] = _hash_json(result)
    validate_result_document(result)
    _write_immutable_result(result_root, result)
    return result


def _select_scenarios(
    scenarios: Sequence[Dict[str, Any]], scenario_ids: Optional[Sequence[str]]
) -> List[Dict[str, Any]]:
    if scenario_ids is None:
        return list(scenarios)
    requested = list(scenario_ids)
    if not requested or len(requested) != len(set(requested)):
        raise RunnerError("scenario_ids must be a non-empty unique allowlist")
    by_id = {scenario["id"]: scenario for scenario in scenarios}
    missing = sorted(set(requested) - set(by_id))
    if missing:
        raise RunnerError(f"unknown requested scenario ids: {missing}")
    return [by_id[scenario_id] for scenario_id in requested]


def _validate_executables(
    scenarios: Sequence[Mapping[str, Any]],
    executables: Mapping[str, Sequence[str]],
) -> Dict[str, Any]:
    required = {str(scenario["adapter"]) for scenario in scenarios}
    missing = sorted(required - set(executables))
    if missing:
        raise RunnerError(f"missing executable mappings for adapters: {missing}")
    identities: Dict[str, Any] = {}
    for adapter in sorted(required):
        components = executables[adapter]
        if not isinstance(components, Sequence) or isinstance(components, (str, bytes)):
            raise RunnerError(f"executable mapping for {adapter!r} must be an array")
        if not components:
            raise RunnerError(f"executable mapping for {adapter!r} is empty")
        recorded_components: List[Dict[str, str]] = []
        for index, component in enumerate(components):
            if not isinstance(component, str) or "\0" in component:
                raise RunnerError(f"executable component for {adapter!r} is invalid")
            path = Path(component)
            if not path.is_absolute() or not path.is_file():
                raise RunnerError(
                    f"executable components for {adapter!r} must be absolute files"
                )
            if index == 0 and not os.access(path, os.X_OK):
                raise RunnerError(
                    f"first executable component for {adapter!r} is not executable"
                )
            recorded_components.append(
                {"name": path.name, "sha256": _hash_file(path)}
            )
        identities[adapter] = {
            "components": recorded_components,
            "identity_hash": _hash_json({"components": recorded_components}),
        }
    return identities


def _execute_invocation(
    scenario: Mapping[str, Any],
    invocation: Mapping[str, Any],
    executable: Sequence[str],
    work_root: Path,
    artifact_root: Path,
    corpus_bases: CorpusBasePool,
) -> Dict[str, Any]:
    run_root = reserve_run_directory(work_root)
    placeholders = _invocation_placeholders(run_root)
    environment = _build_environment(
        scenario["command"]["environment"], placeholders
    )
    reasons: List[str] = []
    precondition = False
    postcondition = False
    manifest: Optional[Dict[str, Any]] = None
    command_result: Optional[_CommandResult] = None
    recorded_environment = _record_environment(
        environment,
        scenario["command"]["environment"]["unset"],
        placeholders,
    )
    artifact_name: Optional[str] = None
    probe_metrics: Dict[str, Any] = {}
    preparation = _empty_preparation_evidence()
    preparation_started = time.perf_counter_ns()
    try:
        manifest = _prepare_invocation(
            scenario,
            executable,
            run_root,
            placeholders,
            environment,
            corpus_bases,
            preparation,
        )
        preparation["total_ns"] = time.perf_counter_ns() - preparation_started
        precondition = True
        argv = _expand_argv(scenario["command"]["argv"], executable, placeholders)
        command_result = _run_command(
            argv,
            environment,
            timeout_seconds=scenario["method"]["timeout_seconds"],
            run_root=run_root,
            label="timed",
            output_sink=scenario["output_sink"],
            retain_stdout=(
                scenario["output_sink"] == "output-digest"
                and scenario["validation"]["stdout_json"] is not None
            ),
        )
        reasons.extend(_validate_process(scenario, command_result))
        stdout_reasons, probe_metrics = _validate_stdout(
            scenario, command_result, manifest
        )
        reasons.extend(stdout_reasons)
        reasons.extend(_validate_required_resources(scenario, command_result))
        try:
            verify_corpus(run_root)
            postcondition = True
        except CorpusError as error:
            reasons.append(f"corpus postcondition failed: {error}")
        reasons.extend(_validate_snapshot_postcondition(scenario, placeholders))
        if scenario["output_sink"] == "output-file":
            artifact_name = (
                f"trial-{invocation['ordinal']:05d}-"
                f"{scenario['id'].replace('/', '_')}.stdout"
            )
            _copy_exclusive(command_result.stdout_path, artifact_root / artifact_name)
    except (CorpusError, RunnerError, OSError, SchemaError) as error:
        preparation["total_ns"] = time.perf_counter_ns() - preparation_started
        reasons.append(f"setup failed: {error}")
    finally:
        try:
            cleanup_run_directory(run_root)
        except CorpusError as error:
            reasons.append(f"cleanup failed: {error}")

    return _trial_record(
        scenario,
        invocation,
        manifest,
        command_result,
        recorded_environment,
        precondition,
        postcondition,
        reasons,
        artifact_name,
        probe_metrics,
        preparation,
    )


def _prepare_invocation(
    scenario: Mapping[str, Any],
    executable: Sequence[str],
    run_root: Path,
    placeholders: Mapping[str, str],
    environment: Mapping[str, str],
    corpus_bases: CorpusBasePool,
    preparation: Dict[str, Any],
) -> Dict[str, Any]:
    corpus = scenario["corpus"]
    manifest, corpus_preparation = corpus_bases.materialize(
        run_root,
        corpus["recipe_id"],
        target_entries=corpus["target_entries"],
        seed=corpus["seed"],
    )
    preparation.update(corpus_preparation)
    snapshot_state = scenario["snapshot_state"]
    if snapshot_state == "compatible-changed":
        _run_preparation_command(
            scenario["preparation"]["snapshot_argv"],
            executable,
            placeholders,
            environment,
            scenario["method"]["timeout_seconds"],
            run_root,
            "snapshot",
        )
    for transition in _CORPUS_TRANSITIONS[corpus["state"]]:
        manifest = apply_transition(run_root, transition)
    if snapshot_state in {
        "compatible-unchanged",
        "incompatible-engine",
        "incompatible-scope",
    }:
        _run_preparation_command(
            scenario["preparation"]["snapshot_argv"],
            executable,
            placeholders,
            environment,
            scenario["method"]["timeout_seconds"],
            run_root,
            "snapshot",
        )
    snapshot_path = Path(placeholders["snapshot_path"])
    if snapshot_state == "corrupt":
        snapshot_path.write_bytes(b"fdu-invalid-snapshot\0v1")
    if snapshot_state == "absent" and snapshot_path.exists():
        raise RunnerError("snapshot preparation did not establish the absent state")
    if snapshot_state not in {"absent", "corrupt"} and not snapshot_path.is_file():
        raise RunnerError("snapshot preparation did not produce the declared snapshot")
    verified = verify_corpus(run_root)
    if verified["manifest_hash"] != manifest["manifest_hash"]:
        raise RunnerError("prepared corpus manifest changed before cache preparation")
    cache_argv = scenario["preparation"]["filesystem_cache_argv"]
    if cache_argv is not None:
        _run_preparation_command(
            cache_argv,
            executable,
            placeholders,
            environment,
            scenario["method"]["timeout_seconds"],
            run_root,
            "filesystem-cache",
        )
    return manifest


def _invocation_placeholders(run_root: Path) -> Dict[str, str]:
    return {
        "corpus_root": str(run_root / CORPUS_NAME),
        "manifest_path": str(run_root / MANIFEST_NAME),
        "output_path": str(run_root / "command-output"),
        "run_root": str(run_root),
        "snapshot_path": str(run_root / "snapshot.fdu"),
    }


def _run_preparation_command(
    template: Optional[Sequence[str]],
    executable: Sequence[str],
    placeholders: Mapping[str, str],
    environment: Mapping[str, str],
    timeout_seconds: float,
    run_root: Path,
    label: str,
) -> None:
    if template is None:
        raise RunnerError(f"{label} preparation command is missing")
    argv = _expand_argv(template, executable, placeholders)
    result = _run_command(
        argv,
        environment,
        timeout_seconds=timeout_seconds,
        run_root=run_root,
        label=f"prepare-{label}",
        output_sink="output-digest",
        retain_stdout=False,
    )
    if result.spawn_error is not None:
        raise RunnerError(f"{label} preparation failed to start: {result.spawn_error}")
    if result.timed_out:
        raise RunnerError(f"{label} preparation timed out")
    if result.capture_error is not None:
        raise RunnerError(f"{label} preparation output failed: {result.capture_error}")
    if result.wait_error is not None:
        raise RunnerError(f"{label} preparation wait failed: {result.wait_error}")
    if result.exit_code != 0:
        raise RunnerError(
            f"{label} preparation exited with {result.exit_code}"
        )


def _run_command(
    argv: Sequence[str],
    environment: Mapping[str, str],
    *,
    timeout_seconds: float,
    run_root: Path,
    label: str,
    output_sink: str,
    retain_stdout: bool,
) -> _CommandResult:
    stdout_path = run_root / f".{label}.stdout"
    stderr_path = run_root / f".{label}.stderr"
    start_ns: Optional[int] = None
    wall_ns: Optional[int] = None
    process: Optional[subprocess.Popen[bytes]] = None
    spawn_error: Optional[str] = None
    timed_out = False
    return_code: Optional[int] = None
    wait_error: Optional[str] = None
    resource_evidence = _unavailable_resources(
        "per-process resource collection did not start"
    )
    drain = _OutputDrain(capture=bytearray() if retain_stdout else None)
    with stderr_path.open("xb") as stderr:
        stdout_file = (
            stdout_path.open("xb") if output_sink == "output-file" else None
        )
        try:
            deadline = time.monotonic() + timeout_seconds
            start_ns = time.perf_counter_ns()
            process = subprocess.Popen(
                list(argv),
                stdin=subprocess.DEVNULL,
                stdout=subprocess.PIPE,
                stderr=stderr,
                env=dict(environment),
                start_new_session=os.name == "posix",
                creationflags=(
                    subprocess.CREATE_NEW_PROCESS_GROUP if os.name == "nt" else 0
                ),
            )
            if process.stdout is None:
                raise RunnerError("subprocess stdout pipe was not created")
            drain_thread = threading.Thread(
                target=_drain_stdout,
                args=(
                    process.stdout,
                    stdout_file,
                    drain,
                    start_ns,
                    output_sink == "output-digest",
                ),
                daemon=True,
                name=f"fdu-perf-{label}-stdout",
            )
            drain_thread.start()
            wait_state: Optional[_WaitState] = None
            wait_thread: Optional[threading.Thread] = None
            if os.name == "posix" and hasattr(os, "wait4"):
                wait_state = _WaitState()
                wait_thread = threading.Thread(
                    target=_wait4_child,
                    args=(process.pid, wait_state),
                    daemon=True,
                    name=f"fdu-perf-{label}-wait4",
                )
                wait_thread.start()
                wait_thread.join(timeout=max(0.0, deadline - time.monotonic()))
                if wait_thread.is_alive():
                    timed_out = True
                    _kill_process_group(process)
                    wait_thread.join(timeout=5)
                if wait_thread.is_alive():
                    wait_error = "child did not exit after process-group cleanup"
                else:
                    return_code = wait_state.return_code
                    wait_error = wait_state.error
                    process.returncode = return_code
                    resource_evidence = _resources_from_wait4(wait_state.usage)
            else:
                try:
                    return_code = process.wait(
                        timeout=max(0.0, deadline - time.monotonic())
                    )
                except subprocess.TimeoutExpired:
                    timed_out = True
                    _kill_process_group(process)
                    return_code = process.wait()
                resource_evidence = _unavailable_resources(
                    "os.wait4 per-process rusage is unavailable on this platform"
                )
            drain_thread.join(timeout=max(0.0, deadline - time.monotonic()))
            if drain_thread.is_alive():
                timed_out = True
                _kill_process_group(process)
                drain_thread.join(timeout=5)
            if drain_thread.is_alive():
                try:
                    process.stdout.close()
                except OSError:
                    pass
                drain_thread.join(timeout=1)
            if drain_thread.is_alive():
                drain.capture_error = "stdout pipe did not close after process cleanup"
            wall_ns = time.perf_counter_ns() - start_ns
        except (OSError, RunnerError) as error:
            if start_ns is not None:
                wall_ns = time.perf_counter_ns() - start_ns
            spawn_error = getattr(error, "strerror", None) or str(error)
        finally:
            if stdout_file is not None:
                stdout_file.close()
    if output_sink == "output-digest" and drain.capture is not None:
        with stdout_path.open("xb") as captured_output:
            captured_output.write(drain.capture)
    process_signal = (
        -return_code if return_code is not None and return_code < 0 else None
    )
    exit_code = return_code if return_code is not None and return_code >= 0 else None
    if output_sink == "output-file":
        stdout_digest = _hash_file(stdout_path)
    else:
        stdout_digest = drain.stdout_sha256 or hashlib.sha256(b"").hexdigest()
    return _CommandResult(
        wall_ns=wall_ns,
        exit_code=exit_code,
        signal=process_signal,
        timed_out=timed_out,
        spawn_error=spawn_error,
        capture_error=drain.capture_error,
        stdout_path=stdout_path,
        stderr_path=stderr_path,
        stdout_bytes=drain.stdout_bytes,
        stderr_bytes=stderr_path.stat().st_size,
        stdout_sha256=stdout_digest,
        stderr_sha256=_hash_file(stderr_path),
        first_output_ns=drain.first_output_ns,
        resources=resource_evidence,
        wait_error=wait_error,
    )


def _drain_stdout(
    stdout: Any,
    destination: Any,
    state: _OutputDrain,
    start_ns: int,
    hash_output: bool,
) -> None:
    """Drain stdout without unbounded buffering and optionally retain bounded JSON."""
    digest = hashlib.sha256()
    read = getattr(stdout, "read1", stdout.read)
    try:
        while True:
            chunk = read(64 * 1024)
            if not chunk:
                break
            if state.first_output_ns is None:
                state.first_output_ns = time.perf_counter_ns() - start_ns
            state.stdout_bytes += len(chunk)
            if hash_output:
                digest.update(chunk)
            if destination is not None:
                destination.write(chunk)
            if state.capture is not None:
                if (
                    len(state.capture) + len(chunk)
                    <= MAX_DIGEST_JSON_CAPTURE_BYTES
                ):
                    state.capture.extend(chunk)
                elif state.capture_error is None:
                    state.capture_error = (
                        "stdout exceeds the bounded JSON validation capture; "
                        "use output-file"
                    )
                    state.capture = None
        if destination is not None:
            destination.flush()
    except (OSError, ValueError) as error:
        state.capture_error = f"cannot drain stdout: {error}"
    finally:
        state.stdout_sha256 = digest.hexdigest() if hash_output else None
        stdout.close()


def _wait4_child(pid: int, state: _WaitState) -> None:
    """Reap exactly one child and retain its non-cumulative resource usage."""
    try:
        _waited_pid, status, usage = os.wait4(pid, 0)
        state.return_code = os.waitstatus_to_exitcode(status)
        state.usage = usage
    except (ChildProcessError, OSError, ValueError) as error:
        state.error = f"cannot wait for child process: {error}"


def _resources_from_wait4(usage: Any) -> Dict[str, Any]:
    if usage is None:
        return _unavailable_resources("wait4 did not return resource usage")
    peak_rss = int(usage.ru_maxrss)
    if sys.platform != "darwin":
        peak_rss *= 1024
    values: Dict[str, Optional[int]] = {
        "input_blocks": int(usage.ru_inblock),
        "involuntary_context_switches": int(usage.ru_nivcsw),
        "major_faults": int(usage.ru_majflt),
        "minor_faults": int(usage.ru_minflt),
        "output_blocks": int(usage.ru_oublock),
        "peak_rss_bytes": peak_rss,
        "read_bytes": None,
        "retained_rss_bytes": None,
        "syscalls": None,
        "system_cpu_ns": round(float(usage.ru_stime) * 1_000_000_000),
        "user_cpu_ns": round(float(usage.ru_utime) * 1_000_000_000),
        "voluntary_context_switches": int(usage.ru_nvcsw),
        "write_bytes": None,
    }
    reasons = {
        "read_bytes": "POSIX rusage reports input blocks, not read bytes",
        "retained_rss_bytes": "retained RSS is not observable after process exit",
        "syscalls": "POSIX rusage does not report syscall counts",
        "write_bytes": "POSIX rusage reports output blocks, not written bytes",
    }
    return {
        "collector": "posix-wait4-rusage-v1",
        **values,
        "unavailable_reasons": reasons,
    }


def _unavailable_resources(reason: str) -> Dict[str, Any]:
    return {
        "collector": "unavailable-v1",
        **{metric: None for metric in _RESOURCE_METRICS},
        "unavailable_reasons": {metric: reason for metric in _RESOURCE_METRICS},
    }


def _kill_process_group(process: subprocess.Popen[bytes]) -> None:
    if os.name == "posix":
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        return
    try:
        subprocess.run(
            ["taskkill", "/PID", str(process.pid), "/T", "/F"],
            stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
            timeout=5,
        )
    except (OSError, subprocess.TimeoutExpired):
        process.kill()


def _validate_process(
    scenario: Mapping[str, Any], result: _CommandResult
) -> List[str]:
    reasons: List[str] = []
    if result.spawn_error is not None:
        reasons.append(f"command failed to start: {result.spawn_error}")
    if result.capture_error is not None:
        reasons.append(f"command output failed: {result.capture_error}")
    if result.wait_error is not None:
        reasons.append(f"command wait failed: {result.wait_error}")
    if result.timed_out:
        reasons.append("command timed out")
    if result.exit_code not in scenario["validation"]["allowed_exit_codes"]:
        reasons.append(f"unexpected exit code {result.exit_code}")
    if scenario["validation"]["stderr"] == "empty" and result.stderr_bytes != 0:
        reasons.append("stderr was not empty")
    return reasons


def _validate_required_resources(
    scenario: Mapping[str, Any], result: _CommandResult
) -> List[str]:
    reasons: List[str] = []
    unavailable = result.resources["unavailable_reasons"]
    for metric in scenario["method"]["required_resources"]:
        if result.resources[metric] is None:
            reasons.append(
                f"required resource {metric} is unavailable: {unavailable[metric]}"
            )
    return reasons


def _validate_stdout(
    scenario: Mapping[str, Any],
    result: _CommandResult,
    manifest: Mapping[str, Any],
) -> Tuple[List[str], Dict[str, Any]]:
    reasons: List[str] = []
    metrics: Dict[str, Any] = {}
    validation = scenario["validation"]
    if validation["stdout_nonempty"] and result.stdout_bytes == 0:
        reasons.append("stdout was empty")
    expected_digest = validation["stdout_sha256"]
    if expected_digest is not None and result.stdout_sha256 != expected_digest:
        reasons.append(
            f"stdout digest mismatch: expected {expected_digest}, "
            f"received {result.stdout_sha256}"
        )
    contract = validation["stdout_json"]
    if contract is None:
        return reasons, metrics
    try:
        output = _read_json_output(result.stdout_path)
    except RunnerError as error:
        reasons.append(str(error))
        return reasons, metrics
    for output_path, expected in contract["equals"].items():
        try:
            actual = _lookup_field(output, output_path)
        except RunnerError as error:
            reasons.append(str(error))
            continue
        if actual != expected:
            reasons.append(
                f"stdout value mismatch for {output_path}: expected {expected!r}, "
                f"received {actual!r}"
            )
    for output_path, manifest_path in contract["matches_manifest"].items():
        try:
            actual = _lookup_field(output, output_path)
            expected = _lookup_field(manifest, manifest_path)
        except RunnerError as error:
            reasons.append(str(error))
            continue
        if actual != expected:
            reasons.append(
                f"stdout manifest mismatch for {output_path}: expected {expected!r}, "
                f"received {actual!r}"
            )
    for output_path in contract["record"]:
        try:
            actual = _lookup_field(output, output_path)
            _validate_recorded_metric(actual, output_path)
        except RunnerError as error:
            reasons.append(str(error))
            continue
        metrics[output_path] = actual
    return reasons, metrics


def _validate_recorded_metric(value: Any, path: str) -> None:
    if value is None or isinstance(value, (bool, int, str)):
        return
    if isinstance(value, float) and value == value and abs(value) != float("inf"):
        return
    raise RunnerError(f"recorded stdout metric {path!r} must be a finite JSON scalar")


def _validate_snapshot_postcondition(
    scenario: Mapping[str, Any], placeholders: Mapping[str, str]
) -> List[str]:
    postcondition = scenario["validation"]["snapshot"]
    exists = Path(placeholders["snapshot_path"]).is_file()
    if postcondition == "exists" and not exists:
        return ["snapshot postcondition required a file"]
    if postcondition == "absent" and exists:
        return ["snapshot postcondition required absence"]
    return []


def _trial_record(
    scenario: Mapping[str, Any],
    invocation: Mapping[str, Any],
    manifest: Optional[Mapping[str, Any]],
    command: Optional[_CommandResult],
    environment: Mapping[str, Any],
    precondition: bool,
    postcondition: bool,
    reasons: Sequence[str],
    artifact_name: Optional[str],
    probe_metrics: Mapping[str, Any],
    preparation: Mapping[str, Any],
) -> Dict[str, Any]:
    corpus = None
    if manifest is not None:
        corpus = {
            "counts": manifest["counts"],
            "manifest_hash": manifest["manifest_hash"],
            "recipe_hash": manifest["recipe_hash"],
            "recipe_id": manifest["recipe_id"],
            "semantic_digest": manifest["semantic_digest"],
            "state": manifest["state"],
        }
    return {
        "argv": list(scenario["command"]["argv"]),
        "corpus": corpus,
        "environment": dict(environment),
        "filesystem_cache_state": scenario["filesystem_cache_state"],
        "ordinal": invocation["ordinal"],
        "output": {
            "artifact": artifact_name,
            "stderr_bytes": command.stderr_bytes if command else 0,
            "stderr_sha256": command.stderr_sha256 if command else None,
            "stdout_bytes": command.stdout_bytes if command else 0,
            "stdout_sha256": command.stdout_sha256 if command else None,
        },
        "preparation": dict(preparation),
        "process": {
            "exit_code": command.exit_code if command else None,
            "signal": command.signal if command else None,
            "spawn_error": command.spawn_error if command else None,
            "timed_out": command.timed_out if command else False,
            "wait_error": command.wait_error if command else None,
        },
        "probe_metrics": dict(probe_metrics),
        "resources": (
            command.resources
            if command
            else _unavailable_resources("command did not start")
        ),
        "sample_index": invocation["sample_index"],
        "sample_kind": invocation["sample_kind"],
        "scenario_id": scenario["id"],
        "snapshot_state": scenario["snapshot_state"],
        "timing": {
            "first_output_ns": command.first_output_ns if command else None,
            "wall_ns": command.wall_ns if command else None,
        },
        "validation": {
            "postcondition": postcondition,
            "precondition": precondition,
            "reasons": list(reasons),
            "valid": precondition and postcondition and not reasons,
        },
    }


def _empty_preparation_evidence() -> Dict[str, Any]:
    return {
        "base_cache_key": None,
        "base_created": False,
        "base_generation_ns": None,
        "cloned_files": 0,
        "copied_bytes": 0,
        "copied_files": 0,
        "copy_strategy": None,
        "directories": 0,
        "hardlinks": 0,
        "materialization_ns": None,
        "source_verification_ns": None,
        "strategy_probe_ns": None,
        "symlinks": 0,
        "total_ns": 0,
    }


def _build_environment(
    declaration: Mapping[str, Any], placeholders: Mapping[str, str]
) -> Dict[str, str]:
    environment = dict(_BASE_ENVIRONMENT)
    if os.name == "nt":
        for name in ("SYSTEMROOT", "WINDIR"):
            if name in os.environ:
                environment[name] = os.environ[name]
    for name, value in declaration["set"].items():
        environment[name] = value.format_map(placeholders)
    for name in declaration["unset"]:
        environment.pop(name, None)
    return environment


def _record_environment(
    environment: Mapping[str, str],
    explicitly_unset: Sequence[str],
    placeholders: Mapping[str, str],
) -> Dict[str, Any]:
    tokenized: Dict[str, str] = {}
    replacements = sorted(
        ((value, f"{{{name}}}") for name, value in placeholders.items()),
        key=lambda item: len(item[0]),
        reverse=True,
    )
    for name, value in sorted(environment.items()):
        recorded = value
        for resolved, token in replacements:
            recorded = recorded.replace(resolved, token)
        if name in {"SYSTEMROOT", "WINDIR"}:
            recorded = "{windows_system_root}"
        tokenized[name] = recorded
    return {
        "explicitly_unset": sorted(explicitly_unset),
        "set": tokenized,
    }


def _expand_argv(
    template: Sequence[str],
    executable: Sequence[str],
    placeholders: Mapping[str, str],
) -> List[str]:
    return list(executable) + [
        argument.format_map(placeholders) for argument in template[1:]
    ]


def _read_json_output(path: Path) -> Mapping[str, Any]:
    try:
        size = path.stat().st_size
        if size > MAX_VALIDATION_OUTPUT_BYTES:
            raise RunnerError("stdout exceeds the validation size limit")
        with path.open("r", encoding="utf-8") as input_file:
            value = json.load(input_file, parse_constant=_reject_json_constant)
    except RunnerError:
        raise
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise RunnerError(f"stdout is not valid JSON: {error}") from error
    if not isinstance(value, dict):
        raise RunnerError("stdout JSON must be an object")
    return value


def _lookup_field(value: Mapping[str, Any], path: str) -> Any:
    current: Any = value
    for component in path.split("."):
        if not isinstance(current, Mapping) or component not in current:
            raise RunnerError(f"field path {path!r} is missing")
        current = current[component]
    return current


def _reject_json_constant(value: str) -> None:
    raise json.JSONDecodeError(f"non-standard JSON constant {value!r}", value, 0)


def _prepare_directory(path: Path, label: str) -> Path:
    try:
        path.mkdir(parents=True, exist_ok=True)
        resolved = path.resolve(strict=True)
    except OSError as error:
        raise RunnerError(f"cannot prepare {label}: {error}") from error
    if not resolved.is_dir():
        raise RunnerError(f"{label} is not a directory")
    return resolved


def _copy_exclusive(source: Path, destination: Path) -> None:
    with source.open("rb") as input_file, destination.open("xb") as output_file:
        shutil.copyfileobj(input_file, output_file)
        output_file.flush()
        os.fsync(output_file.fileno())


def _write_immutable_result(result_root: Path, result: Mapping[str, Any]) -> Path:
    identity = result["identity"]
    timestamp = str(identity["started_at_utc"]).replace(":", "").replace("-", "")
    path = result_root / f"run-{timestamp}-{identity['run_id']}.json"
    with path.open("x", encoding="utf-8", newline="\n") as output:
        json.dump(result, output, ensure_ascii=True, indent=2, sort_keys=True)
        output.write("\n")
        output.flush()
        os.fsync(output.fileno())
    return path


def _source_identity() -> Dict[str, Any]:
    git = shutil.which("git")
    if git is None:
        return {"dirty": None, "reason": "git unavailable", "revision": None}
    revision = _git_output(git, "rev-parse", "HEAD")
    status = _git_output(git, "status", "--porcelain")
    return {
        "dirty": bool(status) if status is not None else None,
        "reason": (
            None
            if revision is not None and status is not None
            else "git query failed"
        ),
        "revision": revision,
    }


def _git_output(git: str, *arguments: str) -> Optional[str]:
    try:
        completed = subprocess.run(
            [git, *arguments],
            cwd=Path(__file__).resolve().parent.parent,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            check=False,
            text=True,
            timeout=5,
        )
    except (OSError, subprocess.TimeoutExpired):
        return None
    if completed.returncode != 0:
        return None
    return completed.stdout.strip()


def _host_class() -> Dict[str, Any]:
    return {
        "architecture": platform.machine() or "unknown",
        "cpu_logical": os.cpu_count(),
        "filesystem": None,
        "filesystem_reason": "portable runner does not infer a claim-grade filesystem",
        "host_class": "local-uncontrolled",
        "kernel": platform.release() or "unknown",
        "memory_bytes": None,
        "memory_reason": "portable collector unavailable",
        "operating_system": platform.system() or "unknown",
    }


def _harness_identity() -> Dict[str, Any]:
    benchmark_root = Path(__file__).resolve().parent
    components = [
        {"name": name, "sha256": _hash_file(benchmark_root / name)}
        for name in (
            "corpus.py",
            "corpus_cache.py",
            "report.py",
            "runner.py",
            "schema.py",
        )
    ]
    identity = {
        "components": components,
        "python_implementation": sys.implementation.name,
        "python_version": platform.python_version(),
    }
    return {**identity, "identity_hash": _hash_json(identity)}


def _hash_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as input_file:
        for chunk in iter(lambda: input_file.read(64 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _hash_json(value: Mapping[str, Any]) -> str:
    unhashed = {key: item for key, item in value.items() if key != "result_hash"}
    encoded = json.dumps(
        unhashed, ensure_ascii=True, separators=(",", ":"), sort_keys=True
    ).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def _utc_now() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="microseconds").replace(
        "+00:00", "Z"
    )
