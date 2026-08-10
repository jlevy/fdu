from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from benchmarks.schema import (
    SchemaError,
    load_result,
    load_scenario_set,
    validate_scenario_set,
)


FIXTURES = Path(__file__).resolve().parent / "fixtures" / "schema"


class ScenarioSchemaTests(unittest.TestCase):
    def test_committed_json_schemas_are_versioned_and_strict(self) -> None:
        schema_directory = Path(__file__).resolve().parents[1] / "schema"
        expected = {
            "observed-corpus-v1.schema.json": "fdu-observed-corpus-v1",
            "result-v1.schema.json": "fdu-performance-result-v1",
            "scenario-v1.schema.json": "fdu-performance-scenarios-v1",
        }
        for filename, version in expected.items():
            with self.subTest(filename=filename):
                document = json.loads(
                    (schema_directory / filename).read_text(encoding="utf-8")
                )
                self.assertEqual(
                    document["$schema"],
                    "https://json-schema.org/draft/2020-12/schema",
                )
                self.assertEqual(document["properties"]["schema"]["const"], version)
                self.assertFalse(document["additionalProperties"])
                for definition in document.get("$defs", {}).values():
                    if definition.get("type") == "object":
                        self.assertFalse(definition["additionalProperties"])

    def test_valid_fixture_loads_without_normalizing_away_inputs(self) -> None:
        loaded = load_scenario_set(FIXTURES / "scenario-valid.json")

        self.assertEqual(loaded["schema"], "fdu-performance-scenarios-v1")
        self.assertEqual(
            loaded["scenarios"][0]["id"], "fixture/contract/baseline"
        )
        self.assertEqual(loaded["scenarios"][0]["method"]["trials"], 2)

    def test_unknown_fields_incompatible_versions_and_truncation_fail_closed(self) -> None:
        cases = (
            ("scenario-unknown-field.json", "unknown fields"),
            ("scenario-incompatible-version.json", "unsupported scenario schema"),
            ("scenario-truncated.json", "valid JSON"),
        )
        for fixture, message in cases:
            with self.subTest(fixture=fixture):
                with self.assertRaisesRegex(SchemaError, message):
                    load_scenario_set(FIXTURES / fixture)

    def test_duplicate_ids_and_unknown_placeholders_fail_closed(self) -> None:
        scenario_set = load_scenario_set(FIXTURES / "scenario-valid.json")
        duplicate = json.loads(json.dumps(scenario_set["scenarios"][0]))
        scenario_set["scenarios"].append(duplicate)
        with self.assertRaisesRegex(SchemaError, "duplicate scenario id"):
            validate_scenario_set(scenario_set)

        scenario_set["scenarios"].pop()
        scenario_set["scenarios"][0]["command"]["argv"].append("{secret_path}")
        with self.assertRaisesRegex(SchemaError, "unknown placeholder"):
            validate_scenario_set(scenario_set)

    def test_result_unknown_fields_versions_and_truncation_fail_closed(self) -> None:
        cases = (
            ("result-unknown-field.json", "unknown fields"),
            ("result-incompatible-version.json", "unsupported result schema"),
            ("result-truncated.json", "valid JSON"),
        )
        for fixture, message in cases:
            with self.subTest(fixture=fixture):
                with self.assertRaisesRegex(SchemaError, message):
                    load_result(FIXTURES / fixture)

    def test_state_preparation_and_environment_rules_are_explicit(self) -> None:
        scenario_set = load_scenario_set(FIXTURES / "scenario-valid.json")
        scenario = scenario_set["scenarios"][0]
        scenario["filesystem_cache_state"] = "verified-warm"
        with self.assertRaisesRegex(SchemaError, "filesystem_cache_argv"):
            validate_scenario_set(scenario_set)

        scenario["filesystem_cache_state"] = "uncontrolled"
        scenario["command"]["environment"]["set"]["FDU_SENTINEL"] = "leak"
        with self.assertRaisesRegex(SchemaError, "both set and unset"):
            validate_scenario_set(scenario_set)

        scenario["command"]["environment"]["unset"] = []
        scenario["command"]["environment"]["set"]["FDU_CACHE"] = "{home}/cache"
        with self.assertRaisesRegex(SchemaError, "unknown placeholder"):
            validate_scenario_set(scenario_set)

    def test_oversized_document_is_rejected_before_json_allocation(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            path = Path(temporary_directory) / "oversized.json"
            with path.open("wb") as output:
                output.truncate(8 * 1024 * 1024 + 1)

            with self.assertRaisesRegex(SchemaError, "exceeds"):
                load_scenario_set(path)

    def test_nonstandard_nonfinite_json_numbers_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            path = Path(temporary_directory) / "nonfinite.json"
            path.write_text(
                '{"scenarios": [], "schema": NaN}', encoding="utf-8"
            )

            with self.assertRaisesRegex(SchemaError, "non-standard JSON constant"):
                load_scenario_set(path)

        scenario_set = load_scenario_set(FIXTURES / "scenario-valid.json")
        scenario_set["scenarios"][0]["method"]["timeout_seconds"] = float("inf")
        with self.assertRaisesRegex(SchemaError, "between 0.01 and 3600"):
            validate_scenario_set(scenario_set)


if __name__ == "__main__":
    unittest.main()
