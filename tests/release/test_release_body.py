"""Release-body derivation: strip comments, unwrap, and refuse draft leftovers."""

from __future__ import annotations

import tempfile
import tomllib
import unittest
from pathlib import Path

from scripts.release.release_body import (
    EXPECTED_HTML_COMMENTS,
    check_release_body,
    html_comment_count,
    main,
    strip_html_comments,
    unwrap_with_flowmark,
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
        notes = f"# Notes\n\n```\n<!-- not the footer -->\n```\n\n{GUIDELINE_FOOTER}\n"
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
        # Every notes file, including the one for the version being prepared: an unfilled
        # `<!-- ... -->` placeholder fails here, before it can reach a tagged release body.
        paths = sorted((ROOT / "docs/project/release-notes").glob("*.md"))
        self.assertTrue(paths)
        for path in paths:
            with self.subTest(notes=path.name):
                notes = path.read_text(encoding="utf-8")
                self.assertEqual(html_comment_count(notes), EXPECTED_HTML_COMMENTS)
                stripped = strip_html_comments(notes)
                check_release_body(notes, stripped, stripped)

    def test_the_cargo_version_has_release_notes(self) -> None:
        # The runbook derives the release body from `release-notes/<version>.md`; a bump
        # without notes would leave that step nothing to derive.
        manifest = tomllib.loads((ROOT / "crates/fdu/Cargo.toml").read_text(encoding="utf-8"))
        version = manifest["package"]["version"]
        self.assertTrue((ROOT / f"docs/project/release-notes/{version}.md").is_file())

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
            self.assertEqual(
                body_path.read_text(encoding="utf-8"), source_path.read_text(encoding="utf-8")
            )

    def test_a_second_comment_does_not_write_body_files(self) -> None:
        notes = f"# Notes\n\n<!-- TODO: fill highlights -->\n\nBody.\n\n{GUIDELINE_FOOTER}\n"
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
            self.assertEqual(status, 1)
            self.assertFalse(source_path.exists())
            self.assertFalse(body_path.exists())

    def test_flowmark_unwrap_joins_a_wrapped_paragraph(self) -> None:
        notes = (
            "# Notes\n\n"
            "This paragraph is wrapped across two lines so the unwrap must join\n"
            "them into one line in the GitHub release body.\n\n"
            f"{GUIDELINE_FOOTER}\n"
        )
        stripped = strip_html_comments(notes)
        body = unwrap_with_flowmark(stripped, root=ROOT)
        check_release_body(notes, stripped, body)
        joined = next(
            line for line in body.splitlines() if line.startswith("This paragraph is wrapped")
        )
        self.assertIn("join them into one line", joined)
        self.assertNotIn("join\n", joined)

    def test_cli_flowmark_unwrap_writes_after_check(self) -> None:
        notes = (
            "# Notes\n\n"
            "This paragraph is wrapped across two lines so the unwrap must join\n"
            "them into one line in the GitHub release body.\n\n"
            f"{GUIDELINE_FOOTER}\n"
        )
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
                    "--root",
                    str(ROOT),
                ]
            )
            self.assertEqual(status, 0)
            body = body_path.read_text(encoding="utf-8")
            joined = next(
                line for line in body.splitlines() if line.startswith("This paragraph is wrapped")
            )
            self.assertIn("join them into one line", joined)


if __name__ == "__main__":
    unittest.main()
