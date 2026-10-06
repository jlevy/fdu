"""The history driver: command shapes per era, start states, answer checks, and the cell.

A history cell is only evidence if every build ran the component as that build spells
it, from the start state the manifest names, and gave the same answer as the others.
Each test here pins one of those, because a wrong shape or a shared cache produces a
cell that looks complete and measures something else.
"""

from __future__ import annotations

import contextlib
import copy
import io
import json
import stat
import tempfile
import unittest
from pathlib import Path
from typing import Any, Dict, List, Sequence, Tuple
from unittest import mock

from benchmarks.realtree import compare_tools, history, perf_index

# Help excerpts as each era printed them, wrapping included: the parser reads the
# options the suite needs and nothing else.
PREWORK_HELP = """\
Usage: fdu-prework [OPTIONS] [PATH]

Options:
  -d, --depth <N>      Directory levels to show; does not limit scanning [default: 2]
      --by-type        Group totals by file extension instead of directory
      --json           Write schema-versioned JSON to stdout
      --no-cache       Do not read or write the snapshot cache
      --color <WHEN>   Colorize human output: auto, always, or never [default: auto]

Cache:
  Unless --no-cache is set, fdu reads and writes a snapshot in the user cache directory.
"""

EXP101_HELP = """\
VIEWS
      --view <LIST>         Views: tree, extensions, types, families, languages, documents, largest,
                            recent, files, summary, or full. Defaults to the view that displays what
                            --analyze asked for
CONTENT ANALYSIS
      --analyze <LIST>        Analyzers to run: none, lines, code, words, or all [default: none]
OUTPUT
      --format <FORMAT>  Output format: text, json, jsonl, or yaml [default: text]
      --color <WHEN>     Colorize human output: auto, always, or never [default: auto]
EXECUTION
      --cache <POLICY>  Cache policy: auto, refresh, read-only, only, or off [default: auto]
"""

V010_HELP = """\
SCOPE
      --no-gitignore                 Read no .gitignore files: rows lose their ignored share, and
                                     the snapshot scope differs
VIEWS
      --view <LIST>         Views: list, tree, files, extensions, types, families, languages,
                            documents, largest, recent, summary, or full. Defaults to list with no
                            analysis, otherwise to a view that displays the requested analysis
CONTENT ANALYSIS
      --analyze <LIST>        Analyzers to run: none, lines, code, words, or all [default: none]
OUTPUT
      --format <FORMAT>  Format: tree (list default), paths, long (size/age/path), json, jsonl,
                         yaml, or automatic text [default: text]
EXECUTION
      --cache <POLICY>  Cache policy: auto, refresh, read-only, only (unverified), or off [default:
                        auto]
"""

V030_HELP = """\
SCOPE
      --no-gitignore                 Read no .gitignore files: rows lose their gitignored share, and
                                     the snapshot scope differs
VIEWS
      --view <LIST>         Views: list, tree, files, extensions, types, families, languages, code,
                            documents, largest, recent, summary, or full. Defaults to list with no
                            analysis, otherwise to a view that displays the requested analysis
CONTENT ANALYSIS
      --analyze <LIST>  Analyzers to run: none, lines, code, words, or all [default: none]
OUTPUT
      --format <FORMAT>  Format: tree (list default), paths, long (size/age/path), json, jsonl,
                         yaml, or automatic text [default: text]
EXECUTION
      --cache <POLICY>  Cache policy: auto (where it pays for this run), on, or off [default: auto]
"""

PREWORK = history.capabilities(PREWORK_HELP, "fdu 0.0.1")
EXP101 = history.capabilities(EXP101_HELP, "fdu 0.1.0-dev")
V010 = history.capabilities(V010_HELP, "fdu 0.1.0")
V030 = history.capabilities(V030_HELP, "fdu 0.3.0")

MANIFEST = history.load_manifest()


def entry(component_id: str) -> Dict[str, Any]:
    return history.component(MANIFEST, component_id)


def shape(component_id: str, caps: history.Capabilities, **kwargs: Any) -> Any:
    return history.shape_for(entry(component_id), caps, **kwargs)


def command_entry(component_id: str, command: str, cache_state: str = "") -> Dict[str, Any]:
    """A component defined by a command, as an override or an earlier manifest had it."""
    item = {key: value for key, value in entry(component_id).items() if key != "jobs"}
    item["command"] = command
    if cache_state:
        item["cache_state"] = cache_state
    return item


class CapabilityTests(unittest.TestCase):
    def test_each_era_spells_the_cache_switches_its_own_way(self) -> None:
        self.assertEqual(PREWORK.cache_off(), ("--no-cache",))
        # Before 0.1.0 the default read and wrote the snapshot: no flag is the policy.
        self.assertEqual(PREWORK.cache_on(), ())
        self.assertEqual(EXP101.cache_policies, history.PRE_COST_MODEL_POLICIES)
        self.assertEqual(EXP101.cache_off(), ("--cache", "off"))
        self.assertEqual(EXP101.cache_on(), ("--cache", "auto"))
        self.assertEqual(V010.cache_on(), ("--cache", "auto"))
        self.assertEqual(V030.cache_policies, frozenset({"auto", "on", "off"}))
        self.assertEqual(V030.cache_on(), ("--cache", "on"))

    def test_a_policy_set_with_no_persisting_policy_has_no_cache_on(self) -> None:
        caps = history.Capabilities(version="x", cache_policies=frozenset({"auto", "off"}))
        self.assertIsNone(caps.cache_on())
        self.assertEqual(caps.cache_off(), ("--cache", "off"))
        self.assertIsNone(history.Capabilities(version="x").cache_off())

    def test_views_analyzers_and_ignore_reading_come_from_the_help(self) -> None:
        self.assertEqual(PREWORK.views, frozenset())
        self.assertFalse(PREWORK.reads_gitignore)
        self.assertTrue(PREWORK.json_flag)
        self.assertFalse(PREWORK.json_format)
        self.assertIn("documents", EXP101.views)
        self.assertIn("summary", EXP101.views)
        self.assertNotIn("code", EXP101.views)
        self.assertFalse(EXP101.reads_gitignore)
        self.assertTrue(EXP101.json_format)
        self.assertTrue(V010.reads_gitignore)
        self.assertNotIn("code", V010.views)
        self.assertIn("code", V030.views)
        self.assertTrue(V030.view_list)
        self.assertEqual(V030.analyzers, frozenset({"none", "lines", "code", "words", "all"}))

    def test_a_build_is_asked_with_help_and_version(self) -> None:
        with tempfile.TemporaryDirectory() as scratch:
            binary = fake_fdu(Path(scratch), V030_HELP, "fdu 0.3.0")
            caps = history.probe_build(binary)
        self.assertEqual(caps.version, "fdu 0.3.0")
        self.assertEqual(caps, V030)

    def test_a_probe_is_asked_for_its_modes_and_its_flags(self) -> None:
        with tempfile.TemporaryDirectory() as scratch:
            modern = fake_probe(
                Path(scratch) / "modern",
                modes=("opened-second-report", "revalidate", "snapshot-save"),
            )
            old = fake_probe(Path(scratch) / "old", modes=("scan-index", "revalidate"))
            strict = fake_probe(
                Path(scratch) / "strict", modes=("delta-apply-large",), reject="--operations"
            )
            self.assertIsNone(history.probe_support(modern, "opened-second-report"))
            self.assertIsNone(history.probe_support(modern, "warm-revalidate"))
            self.assertIn(
                "no opened-second-report mode",
                history.probe_support(old, "opened-second-report") or "",
            )
            # A job is only as available as the mode that prepares its snapshot.
            self.assertIn(
                "no snapshot-save mode", history.probe_support(old, "warm-revalidate") or ""
            )
            self.assertIn(
                "does not accept the delta-apply-large flags",
                history.probe_support(strict, "delta-apply-large") or "",
            )


class CommandTests(unittest.TestCase):
    def test_manifest_commands_parse_into_requests(self) -> None:
        self.assertEqual(history.parse_command("fdu PATH"), history.Request(program="fdu"))
        self.assertEqual(
            history.parse_command("fdu --view code,documents,languages PATH").views,
            ("code", "documents", "languages"),
        )
        self.assertEqual(history.parse_command("fdu --cache on PATH").cache_policy, "on")
        self.assertEqual(
            history.parse_command("fdu --analyze code,words PATH").analyzers, ("code", "words")
        )
        self.assertEqual(
            history.parse_command("perf_probe opened-second-report").probe_jobs,
            ("opened-second-report",),
        )
        self.assertEqual(
            history.parse_command("perf_probe opened-second-report delta-apply-large").probe_jobs,
            ("opened-second-report", "delta-apply-large"),
        )
        for bad in (
            "",
            "fdu",
            "fdu --depth 1 PATH",
            "du PATH",
            "perf_probe",
            "perf_probe no-such-job",
        ):
            with self.subTest(command=bad), self.assertRaises(history.HistoryError):
                history.parse_command(bad)

    def test_every_timed_component_maps_onto_the_reference_build(self) -> None:
        # A manifest component this driver cannot map fails here, before any run.
        for item in MANIFEST["components"]:
            if item.get("from_components"):
                continue
            for job in item.get("jobs") or [None]:
                with self.subTest(component=item["id"], job=job):
                    mapped = history.shape_for(item, V030, probe_check=lambda _: None, job=job)
                    self.assertIsInstance(mapped, history.Shape)
                    self.assertEqual(mapped.job, job)

    def test_a_cell_that_overrides_the_manifest_must_say_why(self) -> None:
        errors = io.StringIO()
        with contextlib.redirect_stderr(errors):
            code = history.main(
                [
                    *("--component", "warm-metadata", "--root", "missing", "--label", "x"),
                    *("--build", "v0.3.0=missing", "--name", "cell"),
                    *("--command", "perf_probe warm-revalidate"),
                ]
            )
        self.assertEqual(code, 1)
        self.assertIn("needs --override-note", errors.getvalue())

    def test_derived_unknown_and_unimplemented_components_fail_closed(self) -> None:
        with self.assertRaisesRegex(history.HistoryError, "derived"):
            history.component(MANIFEST, "memory")
        with self.assertRaisesRegex(history.HistoryError, "unknown component"):
            history.component(MANIFEST, "no-such-component")
        changed = copy.deepcopy(MANIFEST)
        changed["components"][0]["cache_state"] = "the cache, roughly warm"
        with self.assertRaisesRegex(history.HistoryError, "does not implement"):
            history.component(changed, changed["components"][0]["id"])


class ShapeTests(unittest.TestCase):
    def test_first_runs_switch_the_cache_off_in_each_eras_spelling(self) -> None:
        old = shape("cold-cache", PREWORK)
        new = shape("cold-cache", V030)
        self.assertEqual(old.argv, ("{binary}", "--no-cache", "--color", "never", "{root}"))
        self.assertEqual(new.argv, ("{binary}", "--cache", "off", "--color", "never", "{root}"))
        for mapped in (old, new):
            self.assertEqual(mapped.cache_scope, "sample")
            self.assertEqual(mapped.setup_argv, ())
        self.assertEqual(old.variant, "no-cache")
        self.assertEqual(new.variant, "cache-off")

    def test_content_views_name_the_analyzer_they_display(self) -> None:
        self.assertEqual(
            shape("code", V030).argv,
            (
                *("{binary}", "--cache", "off", "--analyze", "code", "--view", "code"),
                *("--color", "never", "{root}"),
            ),
        )
        self.assertEqual(
            shape("documents", EXP101).argv[3:7], ("--analyze", "words", "--view", "documents")
        )
        multi = shape("multi-view", V030)
        self.assertEqual(
            multi.argv[3:7], ("--analyze", "code,words", "--view", "code,documents,languages")
        )

    def test_a_build_without_the_capability_is_unsupported_not_forced(self) -> None:
        self.assertEqual(shape("code", V010), history.Unsupported("no code view"))
        self.assertEqual(shape("summary", PREWORK), history.Unsupported("no summary view"))
        self.assertEqual(shape("multi-view", EXP101), history.Unsupported("no code view"))
        no_list = history.Capabilities(
            version="x",
            cache_policies=frozenset({"off"}),
            views=frozenset({"code", "documents", "languages"}),
            analyzers=frozenset({"code", "words"}),
        )
        self.assertEqual(shape("multi-view", no_list), history.Unsupported("--view takes no list"))
        no_words = history.Capabilities(
            version="x",
            cache_policies=frozenset({"off"}),
            views=frozenset({"documents"}),
            analyzers=frozenset({"code"}),
        )
        self.assertEqual(shape("documents", no_words), history.Unsupported("no words analyzer"))

    def test_the_steady_state_keeps_one_directory_per_build_under_the_default_policy(self) -> None:
        for component_id in ("default-tree", "summary"):
            mapped = shape(component_id, V030)
            with self.subTest(component=component_id):
                self.assertEqual(mapped.cache_scope, "tool")
                self.assertEqual(mapped.setup_argv, ())
                self.assertFalse(any(item.startswith("--cache") for item in mapped.argv))
        self.assertEqual(
            shape("default-tree", PREWORK).argv, ("{binary}", "--color", "never", "{root}")
        )

    def test_a_filled_cache_is_set_up_by_the_same_command_in_a_fresh_directory(self) -> None:
        for component_id in ("warm-content-code", "warm-content-documents"):
            mapped = shape(component_id, V030)
            with self.subTest(component=component_id):
                self.assertEqual(mapped.cache_scope, "sample")
                self.assertEqual(mapped.setup_argv, mapped.argv)
                # The default policy: the content cache is what the setup fills.
                self.assertFalse(any(item.startswith("--cache") for item in mapped.argv))
        self.assertEqual(
            shape("warm-content-documents", EXP101).setup_argv[1:3], ("--analyze", "words")
        )
        self.assertEqual(shape("warm-content-code", V010), history.Unsupported("no code view"))

    def test_a_persisting_cache_policy_is_spelled_per_era(self) -> None:
        # The first definition of warm metadata, `fdu --cache on PATH`.
        persisting = command_entry(
            "warm-metadata",
            "fdu --cache on PATH",
            "snapshot written by an untimed run of the same build just before",
        )
        self.assertEqual(
            history.shape_for(persisting, PREWORK).argv, ("{binary}", "--color", "never", "{root}")
        )
        self.assertEqual(history.shape_for(persisting, EXP101).argv[1:3], ("--cache", "auto"))
        self.assertEqual(history.shape_for(persisting, V030).argv[1:3], ("--cache", "on"))
        for caps in (PREWORK, EXP101, V030):
            mapped = history.shape_for(persisting, caps)
            self.assertEqual(mapped.setup_argv, mapped.argv)
            self.assertEqual(mapped.cache_scope, "sample")
        neither = history.Capabilities(version="x", cache_policies=frozenset({"auto", "off"}))
        self.assertIsInstance(history.shape_for(persisting, neither), history.Unsupported)

    def test_the_opened_root_needs_a_probe_with_the_job(self) -> None:
        job = "opened-second-report"
        self.assertEqual(
            shape("opened-root", V030, job=job),
            history.Unsupported("no perf_probe build was supplied for this milestone"),
        )
        self.assertEqual(
            shape("opened-root", V030, probe_check=lambda job: f"no {job} mode", job=job),
            history.Unsupported("no opened-second-report mode"),
        )
        mapped = shape("opened-root", PREWORK, probe_check=lambda job: None, job=job)
        self.assertTrue(mapped.uses_probe)
        self.assertEqual(mapped.job, "opened-second-report")
        # The harness's own job command, oracle included: every sample is checked.
        self.assertEqual(mapped.argv, ("{binary}", "opened-second-report", "--root", "{root}"))
        self.assertEqual(mapped.cache_scope, "sample")
        self.assertEqual(mapped.setup_argv, ())

    def test_a_component_of_several_jobs_times_the_one_it_is_asked_for(self) -> None:
        # The manifest's own opened root: a `jobs` list beside a prose command.
        two = entry("opened-root")
        self.assertEqual(two["jobs"], ["opened-second-report", "delta-apply-large"])
        with self.assertRaisesRegex(history.HistoryError, "choose one with --job"):
            history.shape_for(two, V030, probe_check=lambda job: None)
        with self.assertRaisesRegex(history.HistoryError, "not one of"):
            history.shape_for(two, V030, probe_check=lambda job: None, job="warm-revalidate")
        mapped = history.shape_for(two, V030, probe_check=lambda job: None, job="delta-apply-large")
        self.assertEqual(
            mapped.argv,
            ("{binary}", "delta-apply-large", "--root", "{root}", "--operations", "100000"),
        )

    def test_a_snapshot_job_is_set_up_by_the_same_probe_into_the_sample_directory(self) -> None:
        mapped = shape(
            "warm-metadata", PREWORK, probe_check=lambda job: None, job="warm-revalidate"
        )
        self.assertEqual(
            mapped.setup_argv,
            ("{binary}", "snapshot-save", "--root", "{root}", "--snapshot", "{cache}/snapshot.fdu"),
        )
        self.assertEqual(
            mapped.argv,
            ("{binary}", "revalidate", "--root", "{root}", "--snapshot", "{cache}/snapshot.fdu"),
        )
        self.assertEqual(mapped.cache_scope, "sample")
        self.assertEqual(mapped.state, "filled")

    def test_each_job_of_a_probe_component_keeps_its_own_start_state(self) -> None:
        # Warm metadata as review B defines it: a timed first run that scans and writes the
        # snapshot, and a timed revalidation of one an untimed run wrote just before. The
        # command form (`perf_probe a; perf_probe b`, a mode name for the second) is how
        # the cells were overridden before the manifest gave the jobs as a list.
        warm = command_entry(
            "warm-metadata",
            "perf_probe cold-open-save; perf_probe revalidate",
            "a first run that scans and writes the snapshot; then a run that loads and "
            "revalidates a snapshot written by an untimed run of the same build just before",
        )
        first = history.shape_for(warm, V030, probe_check=lambda job: None, job="cold-open-save")
        self.assertEqual(first.setup_argv, ())
        self.assertEqual(first.state, "empty")
        self.assertEqual(
            first.argv,
            (
                "{binary}",
                "cold-open-save",
                "--root",
                "{root}",
                "--snapshot",
                "{cache}/snapshot.fdu",
            ),
        )
        # The mode name `revalidate` names the job that runs it.
        second = history.shape_for(warm, V030, probe_check=lambda job: None, job="warm-revalidate")
        self.assertEqual(second.job, "warm-revalidate")
        self.assertEqual(second.setup_argv[1], "snapshot-save")
        self.assertEqual(second.state, "filled")
        # A probe start state never maps onto a command-line report.
        with self.assertRaisesRegex(history.HistoryError, "needs a probe command"):
            history.shape_for({**warm, "command": "fdu PATH"}, V030)

    def test_probe_commands_name_jobs_or_the_modes_they_run(self) -> None:
        self.assertEqual(history.resolve_probe_job("warm-revalidate"), "warm-revalidate")
        self.assertEqual(history.resolve_probe_job("revalidate"), "warm-revalidate")
        self.assertEqual(
            history.parse_command(
                "perf_probe opened-second-report; perf_probe delta-apply-large"
            ).probe_jobs,
            ("opened-second-report", "delta-apply-large"),
        )
        with self.assertRaisesRegex(history.HistoryError, "neither"):
            history.resolve_probe_job("no-such-mode")
        # A mode several jobs run is ambiguous; the job id has to be named.
        shared = [
            mode
            for mode in {job.argv[1] for job in history.measure.PROBE_JOBS.values()}
            if sum(job.argv[1] == mode for job in history.measure.PROBE_JOBS.values()) > 1
            and mode not in history.measure.PROBE_JOBS
        ]
        for mode in shared[:1]:
            with self.assertRaisesRegex(history.HistoryError, "name one of them"):
                history.resolve_probe_job(mode)

    def test_an_analyze_command_needs_the_view_that_shows_it(self) -> None:
        code = {**entry("code"), "command": "fdu --analyze code PATH"}
        mapped = history.shape_for(code, V030)
        self.assertEqual(
            mapped.argv,
            ("{binary}", "--cache", "off", "--analyze", "code", "--color", "never", "{root}"),
        )
        self.assertEqual(history.shape_for(code, V010), history.Unsupported("no code view"))
        uncovered = {**entry("documents"), "command": "fdu --view documents --analyze code PATH"}
        with self.assertRaisesRegex(history.HistoryError, "does not analyze"):
            history.shape_for(uncovered, V030)

    def test_the_revised_manifest_commands_map_as_written(self) -> None:
        # The command forms review B found the round-1 driver refusing (B2).
        cold = {**entry("cold-cache"), "command": "fdu --cache off PATH"}
        self.assertEqual(history.shape_for(cold, PREWORK).argv[1], "--no-cache")
        self.assertEqual(history.shape_for(cold, V030).argv[1:3], ("--cache", "off"))
        on = {**entry("cold-cache"), "command": "fdu --cache on PATH"}
        with self.assertRaisesRegex(history.HistoryError, "cannot run --cache on"):
            history.shape_for(on, V030)
        code = {**entry("code"), "command": "fdu --view code --analyze code PATH"}
        self.assertEqual(
            history.shape_for(code, V030).argv[3:7], ("--analyze", "code", "--view", "code")
        )
        multi = {
            **entry("multi-view"),
            "command": "fdu --view code,documents,languages --analyze code,words PATH",
        }
        self.assertEqual(
            history.shape_for(multi, V030).argv[3:7],
            ("--analyze", "code,words", "--view", "code,documents,languages"),
        )
        everything = {**multi, "command": "fdu --view code,documents,languages --analyze all PATH"}
        self.assertEqual(history.shape_for(everything, V030).argv[3:5], ("--analyze", "all"))
        opened = {
            **entry("opened-root"),
            "command": "perf_probe opened-second-report; perf_probe delta-apply-large",
            "cache_state": "the library's open, then a second report; then an applied change",
        }
        for job_id in ("opened-second-report", "delta-apply-large"):
            mapped = history.shape_for(opened, V030, probe_check=lambda job: None, job=job_id)
            self.assertEqual(mapped.job, job_id)
            self.assertEqual(mapped.setup_argv, ())

    def test_the_answer_check_reads_totals_in_each_eras_json(self) -> None:
        self.assertEqual(
            shape("cold-cache", PREWORK).totals_argv,
            ("{binary}", "--no-cache", "--json", "--depth", "0", "{root}"),
        )
        self.assertEqual(
            shape("cold-cache", EXP101).totals_argv[3:7], ("--format", "json", "--view", "summary")
        )
        self.assertEqual(shape("cold-cache", V030).totals_argv[1:3], ("--cache", "off"))

    def test_a_contract_carries_the_shape_and_may_anchor(self) -> None:
        mapped = shape("warm-content-code", V030)
        contract = history.contract_for("warm-content-code", entry("warm-content-code"), mapped)
        self.assertTrue(contract.fdu_anchor)
        self.assertTrue(contract.writes_cache)
        self.assertEqual(contract.cache_scope, "sample")
        self.assertEqual(contract.setup_argv, mapped.argv)
        self.assertEqual(contract.measures, "fdu-index:warm-content-code")
        self.assertEqual(contract.name, "fdu-index/warm-content-code/default")
        self.assertEqual(contract.primary_metric, "wall_ns")
        self.assertIsNone(contract.stdout_metrics)
        with self.assertRaisesRegex(history.HistoryError, "only a probe job"):
            history.contract_for("code", entry("code"), shape("code", V030), timing="component")

    def test_a_probe_contract_reads_the_component_timer_and_the_oracle(self) -> None:
        mapped = shape(
            "opened-root", V030, probe_check=lambda job: None, job="opened-second-report"
        )
        contract = history.contract_for(
            "opened-root", entry("opened-root"), mapped, timing="component"
        )
        self.assertEqual(contract.primary_metric, "component_ns")
        self.assertIsNotNone(contract.stdout_metrics)
        self.assertEqual(contract.version_argv, ())


class AnswerCheckTests(unittest.TestCase):
    FINGERPRINT = {
        "counts": {"files": 10, "directories": 3, "symlinks": 0, "other": 0, "total": 13},
        "sizes": {"apparent_bytes": 100, "allocated_bytes": 200},
    }
    SUMMARY = {"files": 10, "dirs": 2, "bytes": 100, "allocated": 200}

    def test_root_totals_are_read_from_every_eras_layout(self) -> None:
        layouts = [
            {"tree": self.SUMMARY, "complete": True},
            {"reports": [{"tree": self.SUMMARY}], "status": {"complete": True}},
            {
                "reports": [{"summary": {**self.SUMMARY, "ignored": {"files": 1}}}],
                "status": {"complete": True},
            },
        ]
        for document in layouts:
            totals = history.root_totals(document)
            self.assertEqual(totals["files"], 10)
            self.assertEqual(totals["dirs"], 2)
            self.assertEqual(totals["apparent_bytes"], 100)
            self.assertEqual(totals["allocated_bytes"], 200)
            self.assertTrue(totals["complete"])
        self.assertEqual(history.root_totals(layouts[2])["ignored"], {"files": 1})
        with self.assertRaises(history.HistoryError):
            history.root_totals({"reports": []})

    PROBE_OUTPUT = {
        "schema": "fdu-perf-probe-v1",
        "mode": "fixture",
        "component_ns": 1234,
        "oracle_enabled": True,
        "summary": {"complete": True},
    }

    def run_check(
        self,
        caps: history.Capabilities,
        component_id: str,
        *,
        definition: Dict[str, Any] | None = None,
        **kwargs: Any,
    ) -> Tuple[Dict[str, Any], List[Tuple[List[str], str]]]:
        build = history.Build(label="b", binary=Path("/bin/fdu"), probe=Path("/bin/probe"))
        build.caps = caps
        build.shape = history.shape_for(definition or entry(component_id), caps, **kwargs)
        calls: List[Tuple[List[str], str]] = []

        def runner(argv: Sequence[str], overrides: Dict[str, str]) -> Tuple[int, bytes, bytes]:
            home = overrides["XDG_CACHE_HOME"]
            calls.append((list(argv), home))
            Path(home, f"written-{len(calls)}").write_text("x")
            if "--json" in argv or "json" in argv:
                return 0, json.dumps({"tree": self.SUMMARY, "complete": True}).encode(), b""
            if argv[0] == "/bin/probe":
                return 0, json.dumps(self.PROBE_OUTPUT).encode(), b""
            line = "perf: took 1 ms; analysis 0 fresh, 5 cached (1 B); warm revalidation"
            return 0, b"", line.encode()

        record = history.check_build(build, Path("/root"), self.FINGERPRINT, run=runner)
        return record, calls

    def test_the_steady_state_runs_the_command_again_in_the_same_directory(self) -> None:
        record, calls = self.run_check(V030, "default-tree")
        timed = [call for call in calls if "json" not in call[0]]
        self.assertEqual(len(timed), 2)
        self.assertEqual(timed[0][1], timed[1][1])
        self.assertEqual(record["exit_codes"], [0, 0])
        self.assertEqual(record["totals"]["files"], 10)

    def test_a_filled_state_sets_up_into_the_directory_the_checked_run_reuses(self) -> None:
        record, calls = self.run_check(V030, "warm-content-code")
        self.assertEqual(calls[0][1], calls[1][1])
        self.assertEqual(calls[0][0], calls[1][0])
        self.assertEqual(record["cache_evidence"], "warm")
        self.assertEqual(record["analysis"], {"fresh": 0, "cached": 5})
        # The totals come from a separate, cache-off run in a directory of its own.
        self.assertNotEqual(calls[2][1], calls[1][1])
        self.assertIn("off", calls[2][0])

    def test_the_probe_answer_is_the_jobs_own_oracle_on_the_checked_run(self) -> None:
        agrees = mock.Mock(return_value=None)
        job = "opened-second-report"
        with mock.patch.dict(history.measure._ORACLES, {"index-digest": agrees}):
            record, calls = self.run_check(V030, "opened-root", probe_check=lambda _: None, job=job)
        self.assertEqual(len(calls), 1)
        self.assertIsNone(record["oracle_error"])
        self.assertEqual(record["component_ns"], 1234)
        agrees.assert_called_once()
        disagrees = mock.Mock(return_value="probe engine_digest disagrees")
        with mock.patch.dict(history.measure._ORACLES, {"index-digest": disagrees}):
            record, _ = self.run_check(V030, "opened-root", probe_check=lambda _: None, job=job)
        self.assertIn("disagrees", record["oracle_error"])
        groups = history.answer_groups([record], self.FINGERPRINT)
        self.assertIn("disagrees", " ".join(history.answer_problems(groups)))

    def test_a_snapshot_job_checks_the_snapshot_its_own_setup_wrote(self) -> None:
        with mock.patch.dict(history.measure._ORACLES, {"index-digest": lambda f, s: None}):
            record, calls = self.run_check(
                V030, "warm-metadata", probe_check=lambda _: None, job="warm-revalidate"
            )
        (setup, setup_home), (timed, timed_home) = calls
        self.assertEqual(setup_home, timed_home)
        self.assertEqual(setup[1], "snapshot-save")
        self.assertEqual(timed[1], "revalidate")
        # Both name the same snapshot, inside that sample's own cache directory.
        self.assertEqual(setup[-1], f"{setup_home}/snapshot.fdu")
        self.assertEqual(timed[-1], setup[-1])
        self.assertEqual(record["exit_codes"], [0, 0])

    def test_a_probe_output_without_a_component_timer_is_not_evidence(self) -> None:
        read = history.probe_output_reader("opened-second-report")
        with mock.patch.dict(history.measure._ORACLES, {"index-digest": lambda f, s: None}):
            metrics, reasons = read(json.dumps(self.PROBE_OUTPUT).encode(), self.FINGERPRINT)
            self.assertEqual((metrics, reasons), ({"component_ns": 1234}, []))
            missing = {
                key: value for key, value in self.PROBE_OUTPUT.items() if key != "component_ns"
            }
            metrics, reasons = read(json.dumps(missing).encode(), self.FINGERPRINT)
        self.assertIsNone(metrics["component_ns"])
        self.assertIn("probe reported no component_ns", reasons)
        _metrics, reasons = read(b"", self.FINGERPRINT)
        self.assertTrue(reasons)

    def test_an_early_probe_without_the_newest_mtime_is_held_to_its_digest(self) -> None:
        fingerprint = {
            "counts": {"directories": 3, "total": 13, "files": 10, "other": 0, "symlinks": 0},
            "sizes": {"apparent_bytes": 100, "allocated_bytes": 200},
            "newest_file_mtime_ns": 7,
            "engine_digest": "d",
        }
        summary = {
            "complete": True,
            "dirs": 3,
            "entries": 13,
            "files": 10,
            "other": 0,
            "symlinks": 0,
            "apparent_bytes": 100,
            "allocated_bytes": 200,
            "engine_digest": "d",
        }
        read = history.probe_output_reader("warm-revalidate")

        def output(**fields: Any) -> bytes:
            return json.dumps({**self.PROBE_OUTPUT, "summary": {**summary, **fields}}).encode()

        # The field is absent from the early schema: the digest stands in for it.
        self.assertEqual(read(output(), fingerprint)[1], [])
        # A wrong digest still fails, and so does a reported but wrong newest mtime.
        self.assertTrue(read(output(engine_digest="x"), fingerprint)[1])
        self.assertTrue(read(output(newest_file_mtime_ns=None), fingerprint)[1])
        self.assertTrue(read(output(newest_file_mtime_ns=6), fingerprint)[1])

    def test_groups_split_on_ignore_reading_and_must_agree(self) -> None:
        def record(
            label: str, gitignore: bool, files: int = 10, exit_ok: bool = True
        ) -> Dict[str, Any]:
            totals = {
                "files": files,
                "dirs": 2,
                "apparent_bytes": 100,
                "allocated_bytes": 200,
                "complete": True,
                "ignored": None,
            }
            return {
                "label": label,
                "reads_gitignore": gitignore,
                "exit_ok": exit_ok,
                "totals_exit_code": 0,
                "totals": totals,
            }

        groups = history.answer_groups(
            [record("a", False), record("b", False), record("c", True)], self.FINGERPRINT
        )
        self.assertEqual(sorted(groups), ["with_gitignore", "without_gitignore"])
        self.assertEqual(groups["without_gitignore"]["labels"], ["a", "b"])
        self.assertTrue(groups["without_gitignore"]["matches_fingerprint"])
        self.assertEqual(history.answer_problems(groups), [])

        disagreeing = history.answer_groups(
            [record("a", False), record("b", False, files=11)], self.FINGERPRINT
        )
        self.assertIn("disagree", " ".join(history.answer_problems(disagreeing)))
        failed = history.answer_groups([record("a", False, exit_ok=False)], self.FINGERPRINT)
        self.assertIn("non-zero", " ".join(history.answer_problems(failed)))
        # A lone build whose totals did not parse has nothing to agree with, and fails.
        unparsed = history.answer_groups([{**record("a", False), "totals": None}], self.FINGERPRINT)
        self.assertIn("did not parse", " ".join(history.answer_problems(unparsed)))
        # Agreement with the independent walk is recorded; a group can agree and miss it.
        off = history.answer_groups(
            [record("a", False, files=9), record("b", False, files=9)], self.FINGERPRINT
        )
        self.assertFalse(off["without_gitignore"]["matches_fingerprint"])

    def test_performance_lines_say_what_state_a_run_started_from(self) -> None:
        warm = "perf: took 0.6 s; content read 0 B; analysis 0 fresh, 86,630 cached (1.6 GiB); warm revalidation"
        cold = "Performance: walked 86,630 files / 1.6 GiB; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total 99.9 ms"
        self.assertEqual(history.cache_evidence(warm), "warm")
        self.assertEqual(history.cache_evidence(cold), "cold")
        self.assertEqual(history.cache_evidence(None), "unknown")
        self.assertEqual(history.analysis_counts(warm), {"fresh": 0, "cached": 86630})
        self.assertEqual(
            history.analysis_counts("analysis 86,630 fresh at 18,149 files/s, 0 cached; cold scan"),
            {"fresh": 86630, "cached": 0},
        )
        self.assertEqual(history.footer(f"x\n{cold}\n", Path("/r")), cold)
        self.assertEqual(history.footer("perf: /r/x\n", Path("/r")), "perf: ROOT/x")


class StorageTests(unittest.TestCase):
    def test_every_timed_input_is_located_by_role_and_no_path_is_kept(self) -> None:
        supported = history.Build(label="v0.3.0", binary=Path("/opt/a/fdu"))
        supported.shape = shape("cold-cache", V030)
        skipped = history.Build(label="prework", binary=Path("/opt/b/fdu"))
        skipped.shape = history.Unsupported("no summary view")
        seen: List[Path] = []

        def locate(path: Path) -> str:
            seen.append(path)
            return "external" if path == Path("/opt/a/fdu") else "internal"

        locations = history.storage_check(Path("/tree"), [supported, skipped], locate=locate)
        self.assertEqual(locations["binary:v0.3.0"], "external")
        self.assertNotIn("binary:prework", locations)
        for role in ("subject", "harness", "python", "temporary"):
            self.assertEqual(locations[role], "internal")
        self.assertNotIn("/", json.dumps(locations).replace("binary:", ""))


class CellTests(unittest.TestCase):
    """The cell keeps the committed schema and adds the index fields."""

    COMMITTED = (
        Path(history.__file__).resolve().parents[3]
        / "docs/project/reports/performance-evidence/history/macos-linux-v6.12-internal.json"
    )

    # Labels the committed cell describes, so their descriptions are copied over.
    FIRST, SKIPPED = "prework", "exp001"

    def make_cell(
        self,
        component_id: str = "summary",
        timing: str = "wall",
        capture: Dict[str, Any] | None = None,
        job: str | None = None,
    ) -> Dict[str, Any]:
        """Time a fixture cell and summarize it; ``capture`` receives what it was made of."""
        walls = {self.FIRST: 3_000_000, "v0.3.0": 1_000_000}
        # A probe's own timer, deliberately a different ratio from the process's.
        components = {self.FIRST: 800_000, "v0.3.0": 400_000}
        builds = []
        with tempfile.TemporaryDirectory() as scratch:
            for label in (self.FIRST, self.SKIPPED, "v0.3.0"):
                binary = fake_fdu(Path(scratch), V030_HELP, f"fdu {label}", name=f"fdu-{label}")
                build = history.Build(label=label, binary=binary, probe=binary)
                build.caps = PREWORK if label == self.SKIPPED else V030
                build.shape = shape(
                    component_id,
                    build.caps,
                    probe_check=lambda _, label=label: "no mode" if label == self.SKIPPED else None,
                    job=job,
                )
                builds.append(build)
            tools = {
                build.label: compare_tools.Tool(
                    build.label,
                    history.contract_for(
                        component_id, entry(component_id), build.shape, timing=timing
                    ),
                    build.timed_binary,
                )
                for build in builds
                if build.supported
            }
            counter = [0]

            def spawn(argv, *, timeout_seconds, environment_overrides=None):
                counter[0] += 1
                label = Path(argv[0]).name.removeprefix("fdu-")
                output = {
                    "schema": "fdu-perf-probe-v1",
                    "component_ns": components[label] + counter[0] % 5,
                    "summary": {"complete": True},
                }
                return {
                    "exit_code": 0,
                    "timed_out": False,
                    "wall_ns": walls[label] + counter[0] % 7,
                    "stdout": json.dumps(output).encode(),
                    "stderr": "",
                    "resources": {field: 1 for field in compare_tools.measure._RESOURCE_FIELDS},
                }

            fingerprint = {
                "schema": "fdu-reference-tree-v3",
                # The committed cell's subject, so its descriptions are copied over.
                "label": "linux-v6.12-macos",
                "root_id": "r",
                "engine_digest": "d",
                "counts": {"files": 10, "directories": 3, "total": 13, "symlinks": 0, "other": 0},
                "sizes": {"allocated_bytes": 200, "apparent_bytes": 100},
            }
            root = Path(scratch) / "tree"
            root.mkdir()
            with (
                mock.patch.object(compare_tools.tree, "fingerprint", return_value=fingerprint),
                mock.patch.object(compare_tools.measure, "_spawn", side_effect=spawn),
                mock.patch.object(
                    compare_tools.measure, "_host_pressure_snapshot", return_value={}
                ),
                mock.patch.object(
                    compare_tools.measure, "host_facts", return_value={"system": "Darwin"}
                ),
                mock.patch.object(compare_tools, "_progress"),
                mock.patch.dict(history.measure._ORACLES, {"index-digest": lambda f, s: None}),
            ):
                document = compare_tools.run(
                    root=root,
                    label="linux-v6.12-macos",
                    anchor=tools["v0.3.0"],
                    competitors=[tools[self.FIRST]],
                    trials=4,
                    warmups=1,
                    baseline_fingerprint=None,
                    baseline_output=None,
                    storage="fixture storage",
                )
        records = [
            {
                "label": label,
                "reads_gitignore": True,
                "exit_ok": True,
                "totals_exit_code": 0,
                "totals": None,
                "cache_evidence": "cold",
            }
            for label in (self.FIRST, "v0.3.0")
        ]
        answer_check = {
            "builds": records,
            "groups": history.answer_groups(records, fingerprint),
            "problems": [],
        }
        source = json.loads(self.COMMITTED.read_text(encoding="utf-8"))
        if capture is not None:
            capture.update(
                document=document, builds=builds, answer_check=answer_check, source=source
            )
        return history.build_cell(
            document,
            builds=builds,
            entry=entry(component_id),
            manifest=MANIFEST,
            reference="v0.3.0",
            metadata_source=source,
            answer_check=answer_check,
            storage={
                "locations": {"subject": "internal", "binary:v0.3.0": "internal"},
                "allowed_external": False,
            },
            harness_revision="abc",
            run_artifact="cell.run.json.gz",
        )

    def test_a_stored_run_is_summarized_again_without_new_timing(self) -> None:
        parts: Dict[str, Any] = {}
        earlier = self.make_cell(capture=parts)
        with tempfile.TemporaryDirectory() as scratch:
            directory = Path(scratch)
            (directory / "run-cell.json").write_text(json.dumps(parts["document"]))
            (directory / "answer-check-cell.json").write_text(json.dumps(parts["answer_check"]))
            (directory / "cell.json").write_text(json.dumps(earlier))
            rebuilt_path = history.rebuild_cell(
                directory,
                "cell",
                builds=parts["builds"],
                entry=entry("summary"),
                manifest=MANIFEST,
                reference="v0.3.0",
                metadata_source=parts["source"],
                summary_revision="def",
            )
            rebuilt = json.loads(rebuilt_path.read_text())
            # Same numbers, the timing harness's revision kept, the summarizer's added.
            self.assertEqual(rebuilt["milestones"], json.loads(json.dumps(earlier["milestones"])))
            self.assertEqual(rebuilt["harness"]["harness_revision"], "abc")
            self.assertEqual(rebuilt["harness"]["summary_revision"], "def")
            self.assertEqual(rebuilt["run_artifact"], "cell.run.json.gz")
            # A build that no longer maps to the command the run timed is refused.
            moved = parts["builds"][0]
            original = moved.shape
            moved.shape = shape("default-tree", V030)
            try:
                with self.assertRaisesRegex(
                    history.HistoryError, "no longer map to what the run timed"
                ):
                    history.rebuild_cell(
                        directory,
                        "cell",
                        builds=parts["builds"],
                        entry=entry("summary"),
                        manifest=MANIFEST,
                        reference="v0.3.0",
                        metadata_source=parts["source"],
                        summary_revision="def",
                    )
            finally:
                moved.shape = original

    def test_a_cell_is_stamped_with_its_definition_only_if_it_ran_that_definition(self) -> None:
        parts: Dict[str, Any] = {}
        earlier = self.make_cell(capture=parts)
        with tempfile.TemporaryDirectory() as scratch:
            directory = Path(scratch)
            (directory / "run-cell.json").write_text(json.dumps(parts["document"]))
            (directory / "answer-check-cell.json").write_text(json.dumps(parts["answer_check"]))
            (directory / "cell.json").write_text(json.dumps(earlier))

            def rebuild(**kwargs: Any) -> Dict[str, Any]:
                path = history.rebuild_cell(
                    directory,
                    "cell",
                    builds=parts["builds"],
                    entry=entry("summary"),
                    manifest=MANIFEST,
                    reference="v0.3.0",
                    metadata_source=parts["source"],
                    summary_revision="def",
                    **kwargs,
                )
                return json.loads(path.read_text())

            stamped = rebuild()
            self.assertEqual(
                stamped["component_digest"], perf_index.component_digest(entry("summary"))
            )
            # A single command-line job is keyed by the component itself.
            self.assertEqual(stamped["job"], "summary")
            self.assertEqual(stamped["manifest_version"], 1)
            self.assertEqual(stamped["platform"], "macOS")
            # A definition that asks the builds for something else is refused a stamp,
            # here a summary with the cache off where the run used the default policy.
            other = command_entry(
                "summary",
                "fdu --cache off --view summary PATH",
                "fdu's caches empty for every sample",
            )
            expected = {
                build.label: history.shape_for(other, build.caps) for build in parts["builds"]
            }
            with self.assertRaisesRegex(history.HistoryError, "refusing to stamp summary"):
                rebuild(manifest_entry=other, expected=expected)
            # The same check runs before timing, on the commands a cell is about to run.
            with self.assertRaisesRegex(history.HistoryError, "refusing to stamp"):
                history.stamp_digest(other, history.planned_commands(parts["builds"]), expected)
            self.assertEqual(
                history.stamp_digest(
                    entry("summary"),
                    history.planned_commands(parts["builds"]),
                    {build.label: build.shape for build in parts["builds"]},
                ),
                perf_index.component_digest(entry("summary")),
            )

    def test_the_cell_names_its_component_manifest_and_platform(self) -> None:
        cell = self.make_cell()
        self.assertEqual(cell["schema"], "fdu-history-summary-v1")
        self.assertEqual(cell["component"], "summary")
        self.assertEqual(cell["manifest_version"], 1)
        self.assertEqual(cell["platform"], "macOS")
        self.assertEqual(cell["reference_build"], "v0.3.0")
        self.assertEqual(cell["subject"]["storage"], "internal SSD")

    def test_the_cell_keeps_every_field_of_the_committed_schema(self) -> None:
        cell = self.make_cell()
        committed = json.loads(self.COMMITTED.read_text(encoding="utf-8"))
        missing = set(committed) - set(cell) - {"display_order"}
        self.assertEqual(missing, set())
        for index, item in enumerate(cell["milestones"]):
            if not item["supported"]:
                continue
            with self.subTest(label=item["label"]):
                self.assertEqual(set(committed["milestones"][index]) - set(item), set())
        self.assertEqual(set(committed["headline"]) - set(cell["headline"]), set())
        self.assertEqual(set(committed["subject"]) - set(cell["subject"]), set())

    def test_an_unsupported_build_is_recorded_and_skipped_in_the_comparisons(self) -> None:
        cell = self.make_cell()
        labels = [item["label"] for item in cell["milestones"]]
        self.assertEqual(labels, [self.FIRST, self.SKIPPED, "v0.3.0"])
        skipped = cell["milestones"][1]
        self.assertFalse(skipped["supported"])
        self.assertEqual(skipped["unsupported_reason"], "no summary view")
        self.assertIsNone(skipped["wall_ms"])
        # Its description still comes from the earlier cell.
        self.assertEqual(skipped["short"], "H1 producer")
        first, anchor = cell["milestones"][0], cell["milestones"][2]
        self.assertTrue(first["supported"] and anchor["supported"])
        paired = first["vs_v0_3_0_paired_harness"]
        self.assertEqual(paired["pairs"], 4)
        self.assertGreater(paired["median_change_pct"], 150)
        self.assertIsNone(anchor["vs_v0_3_0_paired_harness"])
        # The previous milestone skips the build that did not run.
        self.assertEqual(anchor["vs_previous_milestone_derived"]["previous"], self.FIRST)
        self.assertEqual(cell["headline"]["first_label"], self.FIRST)
        self.assertAlmostEqual(cell["headline"]["paired_speedup_x"], 3.0, places=1)
        self.assertEqual(cell["timed_metric"], "wall_ns")
        self.assertIsNone(cell["override"])

    def test_a_probe_cell_times_the_whole_process_and_keeps_the_component_beside_it(
        self,
    ) -> None:
        cell = self.make_cell("opened-root", timing="wall", job="opened-second-report")
        self.assertEqual(cell["timed_metric"], "wall_ns")
        self.assertEqual(cell["job"], "opened-second-report")
        first = cell["milestones"][0]
        # Whole-process wall, discovery included: the 3x the process took.
        self.assertAlmostEqual(first["vs_v0_3_0_paired_harness"]["median_change_pct"], 200, delta=2)
        self.assertAlmostEqual(
            first["vs_v0_3_0_paired_harness_component"]["median_change_pct"], 100, delta=2
        )
        self.assertAlmostEqual(first["component_ms"]["median"], 0.8, places=2)
        self.assertAlmostEqual(cell["headline"]["paired_speedup_x"], 3.0, places=1)

    def test_a_component_timed_cell_compares_the_probes_own_timer(self) -> None:
        cell = self.make_cell("opened-root", timing="component", job="opened-second-report")
        self.assertEqual(cell["timed_metric"], "component_ns")
        self.assertEqual(cell["job"], "opened-second-report")
        first, anchor = cell["milestones"][0], cell["milestones"][2]
        self.assertEqual(first["job"], "opened-second-report")
        self.assertAlmostEqual(first["component_ms"]["median"], 0.8, places=2)
        self.assertAlmostEqual(first["wall_ms"]["median"], 3.0, places=2)
        # The comparison the index reads is on the component timer: 2x, not the 3x wall.
        self.assertAlmostEqual(first["vs_v0_3_0_paired_harness"]["median_change_pct"], 100, delta=2)
        self.assertAlmostEqual(
            first["vs_v0_3_0_paired_harness_wall"]["median_change_pct"], 200, delta=2
        )
        self.assertIsNone(anchor["vs_v0_3_0_paired_harness_wall"])
        self.assertAlmostEqual(cell["headline"]["paired_speedup_x"], 2.0, places=1)
        # Memory is recorded for every component, probe-timed or not.
        self.assertIsNotNone(first["peak_rss_mib"]["median"])
        self.assertEqual(cell["milestones"][1]["unsupported_reason"], "no mode")


def fake_fdu(directory: Path, help_text: str, version: str, *, name: str = "fdu") -> Path:
    path = directory / name
    path.write_text(
        "#!/bin/sh\n"
        "if [ \"$1\" = --help ]; then cat <<'EOF'\n" + help_text + "EOF\n"
        f'elif [ "$1" = --version ]; then echo "{version}"; fi\n'
    )
    path.chmod(path.stat().st_mode | stat.S_IEXEC)
    return path


def fake_probe(path: Path, *, modes: Sequence[str], reject: str = "") -> Path:
    """A probe that knows ``modes`` and, like the early probes, refuses ``reject`` first."""
    known = " ".join(modes)
    flag_check = (
        f'for a in "$@"; do [ "$a" = {reject} ] && '
        f"{{ echo 'unknown argument \"{reject}\"' >&2; exit 2; }}; done\n"
        if reject
        else ""
    )
    path.write_text(
        "#!/bin/sh\n"
        + flag_check
        + f'case " {known} " in *" $1 "*) echo "I/O error at $3" >&2; exit 1;; esac\n'
        + 'echo "unknown mode \\"$1\\"" >&2; exit 2\n'
    )
    path.chmod(path.stat().st_mode | stat.S_IEXEC)
    return path


if __name__ == "__main__":
    unittest.main()
