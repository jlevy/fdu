"""The history driver: command shapes per era, start states, answer checks, and the cell.

A history cell is only evidence if every build ran the component as that build spells
it, from the start state the manifest names, and gave the same answer as the others.
Each test here pins one of those, because a wrong shape or a shared cache produces a
cell that looks complete and measures something else.
"""

from __future__ import annotations

import copy
import json
import stat
import tempfile
import unittest
from pathlib import Path
from typing import Any, Dict, List, Sequence, Tuple
from unittest import mock

from benchmarks.realtree import compare_tools, history

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

    def test_a_probe_is_asked_for_its_mode_and_its_flags(self) -> None:
        with tempfile.TemporaryDirectory() as scratch:
            modern = fake_probe(Path(scratch) / "modern", modes=("opened-second-report",))
            old = fake_probe(Path(scratch) / "old", modes=("scan-index",))
            strict = fake_probe(
                Path(scratch) / "strict", modes=("opened-second-report",), flags=False
            )
            self.assertIsNone(history.probe_support(modern, "opened-second-report"))
            self.assertIn(
                "no opened-second-report mode",
                history.probe_support(old, "opened-second-report") or "",
            )
            self.assertIn(
                "does not accept --no-oracle",
                history.probe_support(strict, "opened-second-report") or "",
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
            history.parse_command("perf_probe opened-second-report").probe_mode,
            "opened-second-report",
        )
        for bad in ("", "fdu", "fdu --depth 1 PATH", "du PATH", "perf_probe a b"):
            with self.subTest(command=bad), self.assertRaises(history.HistoryError):
                history.parse_command(bad)

    def test_every_timed_component_maps_onto_the_reference_build(self) -> None:
        # A manifest component this driver cannot map fails here, before any run.
        for item in MANIFEST["components"]:
            if item.get("from_components"):
                continue
            with self.subTest(component=item["id"]):
                mapped = history.shape_for(item, V030, probe_check=lambda mode: None)
                self.assertIsInstance(mapped, history.Shape)

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

    def test_warm_metadata_spells_the_persisting_policy_per_era(self) -> None:
        self.assertEqual(
            shape("warm-metadata", PREWORK).argv, ("{binary}", "--color", "never", "{root}")
        )
        self.assertEqual(shape("warm-metadata", EXP101).argv[1:3], ("--cache", "auto"))
        self.assertEqual(shape("warm-metadata", V030).argv[1:3], ("--cache", "on"))
        for caps in (PREWORK, EXP101, V030):
            mapped = shape("warm-metadata", caps)
            self.assertEqual(mapped.setup_argv, mapped.argv)
            self.assertEqual(mapped.cache_scope, "sample")
        neither = history.Capabilities(version="x", cache_policies=frozenset({"auto", "off"}))
        self.assertIsInstance(shape("warm-metadata", neither), history.Unsupported)

    def test_the_opened_root_needs_a_probe_with_the_mode(self) -> None:
        self.assertEqual(
            shape("opened-root", V030),
            history.Unsupported("no perf_probe build was supplied for this milestone"),
        )
        self.assertEqual(
            shape("opened-root", V030, probe_check=lambda mode: f"no {mode} mode"),
            history.Unsupported("no opened-second-report mode"),
        )
        mapped = shape("opened-root", PREWORK, probe_check=lambda mode: None)
        self.assertTrue(mapped.uses_probe)
        self.assertEqual(
            mapped.argv,
            ("{binary}", "opened-second-report", "--root", "{root}", "--no-oracle"),
        )
        self.assertEqual(mapped.cache_scope, "sample")

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

    def run_check(
        self, caps: history.Capabilities, component_id: str, **kwargs: Any
    ) -> Tuple[Dict[str, Any], List[Tuple[List[str], str]]]:
        build = history.Build(label="b", binary=Path("/bin/fdu"), probe=Path("/bin/probe"))
        build.caps = caps
        build.shape = shape(component_id, caps, **kwargs)
        calls: List[Tuple[List[str], str]] = []

        def runner(argv: Sequence[str], overrides: Dict[str, str]) -> Tuple[int, bytes, bytes]:
            home = overrides["XDG_CACHE_HOME"]
            calls.append((list(argv), home))
            Path(home, f"written-{len(calls)}").write_text("x")
            if "--json" in argv or "json" in argv:
                return 0, json.dumps({"tree": self.SUMMARY, "complete": True}).encode(), b""
            if "opened-second-report" in argv:
                return 0, json.dumps({"summary": {}}).encode(), b""
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

    def test_the_probe_answer_is_its_oracle_without_the_timed_flags(self) -> None:
        with mock.patch.object(history.tree, "probe_agrees", return_value=None) as agrees:
            record, calls = self.run_check(V030, "opened-root", probe_check=lambda mode: None)
        self.assertIn("--no-oracle", calls[0][0])
        self.assertNotIn("--no-oracle", calls[1][0])
        self.assertIsNone(record["oracle_error"])
        agrees.assert_called_once()

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

    def make_cell(self) -> Dict[str, Any]:
        walls = {self.FIRST: 3_000_000, "v0.3.0": 1_000_000}
        builds = []
        with tempfile.TemporaryDirectory() as scratch:
            for label in (self.FIRST, self.SKIPPED, "v0.3.0"):
                binary = fake_fdu(Path(scratch), V030_HELP, f"fdu {label}", name=f"fdu-{label}")
                build = history.Build(label=label, binary=binary)
                build.caps = PREWORK if label == self.SKIPPED else V030
                build.shape = shape("summary", build.caps)
                builds.append(build)
            tools = {
                build.label: compare_tools.Tool(
                    build.label,
                    history.contract_for("summary", entry("summary"), build.shape),
                    build.binary,
                )
                for build in builds
                if build.supported
            }
            counter = [0]

            def spawn(argv, *, timeout_seconds, environment_overrides=None):
                counter[0] += 1
                label = Path(argv[0]).name.removeprefix("fdu-")
                return {
                    "exit_code": 0,
                    "timed_out": False,
                    "wall_ns": walls[label] + counter[0] % 7,
                    "stdout": b"",
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
        return history.build_cell(
            document,
            builds=builds,
            entry=entry("summary"),
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


def fake_fdu(directory: Path, help_text: str, version: str, *, name: str = "fdu") -> Path:
    path = directory / name
    path.write_text(
        "#!/bin/sh\n"
        "if [ \"$1\" = --help ]; then cat <<'EOF'\n" + help_text + "EOF\n"
        f'elif [ "$1" = --version ]; then echo "{version}"; fi\n'
    )
    path.chmod(path.stat().st_mode | stat.S_IEXEC)
    return path


def fake_probe(path: Path, *, modes: Sequence[str], flags: bool = True) -> Path:
    known = " ".join(modes)
    flag_check = (
        ""
        if flags
        else (
            'for a in "$@"; do [ "$a" = --no-oracle ] && { echo \'unknown argument "--no-oracle"\' >&2; exit 2; }; done\n'
        )
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
