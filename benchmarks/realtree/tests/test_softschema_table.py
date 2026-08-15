"""Tests for schema-directed tabular projections of soft-schema directories."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from benchmarks.realtree import softschema_table


class SoftSchemaTableTests(unittest.TestCase):
    def test_projects_a_real_artifact_directory_from_authoritative_frontmatter(
        self,
    ) -> None:
        columns = (
            softschema_table.ColumnSpec("id", "Experiment"),
            softschema_table.ColumnSpec("title", "Candidate"),
            softschema_table.ColumnSpec("subject.host_system", "Host"),
            softschema_table.ColumnSpec("method.trials", "Pairs"),
            softschema_table.ColumnSpec("verdict.decision", "Decision"),
            softschema_table.ColumnSpec("verdict.change_pct", "Change"),
        )

        projection = softschema_table.project_directory(
            softschema_table.REPOSITORY_ROOT / "docs/project/experiments",
            columns=columns,
            pattern="exp-*.md",
        )

        self.assertEqual(projection["contract"], "fdu.performance:Experiment/v1")
        self.assertEqual(projection["schema"]["title"], "Experiment")
        self.assertEqual(projection["row_count"], 56)
        self.assertEqual(
            [column["path"] for column in projection["columns"]],
            [column.path for column in columns],
        )
        self.assertEqual(projection["columns"][2]["type"], "string")
        self.assertEqual(projection["rows"][1]["values"]["id"], "exp-001")
        self.assertEqual(
            projection["rows"][1]["values"]["verdict.decision"], "accepted"
        )

    def test_does_not_treat_markdown_body_text_as_structured_data(self) -> None:
        with tempfile.TemporaryDirectory() as directory_name:
            directory = Path(directory_name)
            (directory / "record.schema.yaml").write_text(
                """\
$schema: https://json-schema.org/draft/2020-12/schema
title: Record
type: object
required: [id, score]
properties:
  id:
    type: string
    description: Stable record identifier.
  score:
    type: number
x-softschema:
  contract: example:Record/v1
""",
                encoding="utf-8",
            )
            (directory / "one.md").write_text(
                """\
---
softschema:
  contract: example:Record/v1
  schema: record.schema.yaml
  envelope: record
  status: enforced
record:
  id: real-id
  score: 7
---

# Notes

The score is 999 and the id is fake-id.
""",
                encoding="utf-8",
            )

            projection = softschema_table.project_directory(
                directory,
                columns=(
                    softschema_table.ColumnSpec("id"),
                    softschema_table.ColumnSpec("score"),
                ),
            )

        self.assertEqual(projection["rows"][0]["values"], {"id": "real-id", "score": 7})
        self.assertEqual(
            projection["columns"][0]["description"], "Stable record identifier."
        )

    def test_rejects_a_mixed_contract_directory(self) -> None:
        with tempfile.TemporaryDirectory() as directory_name:
            directory = Path(directory_name)
            (directory / "record.schema.yaml").write_text(
                """\
type: object
properties:
  id: {type: string}
x-softschema:
  contract: example:Record/v1
""",
                encoding="utf-8",
            )
            artifact = """\
---
softschema:
  contract: {contract}
  schema: record.schema.yaml
  envelope: record
  status: enforced
record:
  id: {identifier}
---
"""
            (directory / "one.md").write_text(
                artifact.format(contract="example:Record/v1", identifier="one"),
                encoding="utf-8",
            )
            (directory / "two.md").write_text(
                artifact.format(contract="example:Other/v1", identifier="two"),
                encoding="utf-8",
            )

            with self.assertRaisesRegex(ValueError, "mixed soft-schema contracts"):
                softschema_table.project_directory(
                    directory,
                    columns=(softschema_table.ColumnSpec("id"),),
                )

    def test_nested_column_is_not_required_when_its_parent_is_optional(self) -> None:
        schema = {
            "type": "object",
            "properties": {
                "optional": {
                    "type": "object",
                    "required": ["value"],
                    "properties": {"value": {"type": "string"}},
                }
            },
        }

        descriptor = softschema_table._column_descriptor(
            schema, softschema_table.ColumnSpec("optional.value")
        )

        self.assertFalse(descriptor["required"])


if __name__ == "__main__":
    unittest.main()
