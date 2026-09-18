"""Release-body derivation: strip comments, unwrap, and refuse draft leftovers."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from scripts.release.release_body import (
    EXPECTED_HTML_COMMENTS,
    check_release_body,
    html_comment_count,
    main,
    strip_html_comments,
)

ROOT = Path(__file__).resolve().parents[2]

GUIDELINE_FOOTER = (
    "<!-- This document follows common-doc-guidelines.md.\n"
    "See github.com/jlevy/practical-prose and review guidelines before editing.\n"
    "-->"
)


class ReleaseBodyTests(unittest.TestCase):
    """The body script must not strip code, and must refuse a second comment."""

    def test_an_unfilled_placeholder_comment_is_a_check_failure(self) -> None:
        notes = f"# Notes\n\n<!-- TODO: fill highlights -->\n\nBody.\n\n{GUIDELINE_FOOTER}\n"
        stripped = strip_html_comments(notes)
        self.assertEqual(html_comment_count(notes), 2)
        self.assertNotIn("TODO: fill highlights", stripped)
        with self.assertRaisesRegex(ValueError, "found 2"):
            check_release_body(notes, stripped, stripped)

    def test_a_comment_inside_a_code_span_is_kept_and_not_counted(self) -> None:
        notes = f"Show `<!-- keep -->` in examples.\n\n{GUIDELINE_FOOTER}\n"
        stripped = strip_html_comments(notes)
        self.assertIn("`<!-- keep -->`", stripped)
        self.assertNotIn("common-doc-guidelines", stripped)
        self.assertEqual(html_comment_count(notes), EXPECTED_HTML_COMMENTS)
        check_release_body(notes, stripped, stripped)

    def test_a_comment_inside_a_fenced_block_is_kept_and_not_counted(self) -> None:
        notes = (
            "# Notes\n\n"
            "```\n"
            "<!-- not the footer -->\n"
            "```\n\n"
            f"{GUIDELINE_FOOTER}\n"
        )
        stripped = strip_html_comments(notes)
        self.assertIn("<!-- not the footer -->", stripped)
        self.assertEqual(html_comment_count(notes), EXPECTED_HTML_COMMENTS)
        check_release_body(notes, stripped, stripped)

    def test_a_table_survives_stripping_unchanged(self) -> None:
        notes = (
            "# Notes\n\n"
            "| Channel | Setup |\n"
            "| --- | --- |\n"
            "| crates.io | token |\n\n"
            f"{GUIDELINE_FOOTER}\n"
        )
        stripped = strip_html_comments(notes)
        self.assertIn("| Channel | Setup |", stripped)
        self.assertIn("| crates.io | token |", stripped)
        check_release_body(notes, stripped, stripped)

    def test_shipped_release_notes_have_only_the_guideline_footer(self) -> None:
        notes = (ROOT / "docs/project/release-notes/0.1.0.md").read_text(encoding="utf-8")
        self.assertEqual(html_comment_count(notes), EXPECTED_HTML_COMMENTS)
        stripped = strip_html_comments(notes)
        check_release_body(notes, stripped, stripped)

    def test_cli_identity_unwrap_writes_and_checks(self) -> None:
        notes = f"# Notes\n\nA paragraph.\n\n{GUIDELINE_FOOTER}\n"
        with tempfile.TemporaryDirectory() as directory:
            notes_path = Path(directory) / "notes.md"
            source_path = Path(directory) / "notes-source.md"
            body_path = Path(directory) / "body.md"
            notes_path.write_text(notes, encoding="utf-8")
            status = main(
                [
                    "--notes",
                    str(notes_path),
                    "--source",
                    str(source_path),
                    "--body",
                    str(body_path),
                    "--unwrap",
                    "identity",
                ]
            )
            self.assertEqual(status, 0)
            self.assertEqual(source_path.read_text(encoding="utf-8"), strip_html_comments(notes))
            self.assertEqual(body_path.read_text(encoding="utf-8"), source_path.read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
