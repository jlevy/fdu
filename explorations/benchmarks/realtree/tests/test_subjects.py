"""The nominated subject set: what it refuses, and what it tells a reader.

Campaign 2 will not accept a change on generated evidence alone, and this module is
where that rule stops being prose. Every case here is a way the rule could be satisfied
on paper and not in fact.
"""

from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from benchmarks.realtree import subjects


def _subject(label: str, character: str, **overrides):
    document = {
        "label": label,
        "character": character,
        "provenance": "a real tree on this host",
        "reconstructible": False,
        "root_id": "a" * 64,
        "engine_digest": "b" * 64,
        "entries": 1000,
        "directories": 100,
        "files": 900,
        "symlinks": 0,
        "apparent_bytes": 4096,
        "allocated_bytes": 4096,
        "max_depth": 8,
        "sparse_ratio": 1.0,
    }
    document.update(overrides)
    return document


def _document(*subject_documents):
    return {
        "schema": subjects.DOCUMENT_SCHEMA,
        "characters_required": subjects.MINIMUM_CHARACTERS,
        "host": {"system": "Darwin", "release": "25.5.0", "arch": "arm64"},
        "subjects": list(subject_documents),
    }


class PolicyTests(unittest.TestCase):
    def test_a_spread_set_can_decide(self) -> None:
        document = _document(
            _subject("src", "source-checkout"),
            _subject("cache", "package-cache"),
            _subject("prefix", "system-prefix"),
        )
        self.assertEqual(subjects.policy_gaps(document), [])

    def test_three_trees_of_one_character_cannot(self) -> None:
        """Count is not spread.

        Three package caches agree with each other about everything that made the
        recorded transfer failures transfer failures, so they are one subject measured
        three times wearing three labels.
        """
        document = _document(
            _subject("a", "package-cache"),
            _subject("b", "package-cache"),
            _subject("c", "package-cache"),
        )
        gaps = subjects.policy_gaps(document)
        self.assertTrue(gaps)
        self.assertIn("1 of 3 required characters", gaps[0])

    def test_a_sparse_tree_screens_but_cannot_decide(self) -> None:
        """The exp-064 failure, caught before it decides anything.

        A tree of holes reads at no cost per file, so per-file work looks larger there
        than anywhere real -- which is how a -13.40% cold figure became -2.38% on dense
        source. Provenance saying "real" does not make a sparse tree a real subject.
        """
        document = _document(
            _subject("src", "source-checkout"),
            _subject("cache", "package-cache"),
            _subject("holes", "system-prefix", sparse_ratio=22.6),
        )
        gaps = subjects.policy_gaps(document)
        self.assertTrue(any("22.6x sparse" in gap for gap in gaps))
        self.assertTrue(any("screens but cannot decide" in gap for gap in gaps))

    def test_a_subject_without_provenance_is_reported(self) -> None:
        document = _document(
            _subject("src", "source-checkout"),
            _subject("cache", "package-cache"),
            _subject("prefix", "system-prefix", provenance="   "),
        )
        self.assertTrue(any("records no provenance" in gap for gap in subjects.policy_gaps(document)))

    def test_an_empty_set_says_so_plainly(self) -> None:
        self.assertEqual(subjects.policy_gaps(_document()), ["the set is empty"])


class DriftTests(unittest.TestCase):
    """A nominated tree is a live working directory, so it will move."""

    def test_an_unchanged_set_reports_nothing(self) -> None:
        document = _document(_subject("src", "source-checkout"))
        self.assertEqual(subjects.drift(document, _document(_subject("src", "source-checkout"))), [])

    def test_a_changed_tree_names_what_moved(self) -> None:
        before = _document(_subject("src", "source-checkout"))
        after = _document(
            _subject("src", "source-checkout", engine_digest="c" * 64, entries=1200, files=1100)
        )
        reasons = subjects.drift(before, after)
        self.assertEqual(len(reasons), 1)
        self.assertIn("entries 1000 -> 1200", reasons[0])

    def test_a_tree_that_moved_house_is_not_the_same_subject(self) -> None:
        # root_id hashes the path, so a different one is a different tree wearing a
        # familiar label -- worth saying differently from "its contents changed".
        before = _document(_subject("src", "source-checkout"))
        after = _document(_subject("src", "source-checkout", root_id="f" * 64))
        self.assertEqual(subjects.drift(before, after), ["src now lives at a different path"])

    def test_content_can_change_without_the_shape_changing(self) -> None:
        before = _document(_subject("src", "source-checkout"))
        after = _document(_subject("src", "source-checkout", engine_digest="c" * 64))
        self.assertIn("same shape, different content", subjects.drift(before, after)[0])

    def test_an_absent_or_extra_subject_is_named(self) -> None:
        before = _document(_subject("src", "source-checkout"))
        after = _document(_subject("other", "package-cache"))
        reasons = subjects.drift(before, after)
        self.assertIn("src is nominated but was not observed", reasons)
        self.assertIn("other was observed but is not in the committed set", reasons)


class NominationLoadingTests(unittest.TestCase):
    def _write(self, payload) -> Path:
        scratch = Path(tempfile.mkdtemp())
        self.addCleanup(lambda: None)
        destination = scratch / "subjects.local.json"
        destination.write_text(json.dumps(payload), encoding="utf-8")
        return destination

    def test_an_unknown_character_is_refused(self) -> None:
        path = self._write(
            [{"label": "x", "character": "vibes", "path": "/tmp", "provenance": "p"}]
        )
        with self.assertRaises(subjects.SubjectError) as raised:
            subjects.load_nominations(path)
        self.assertIn("unknown character", str(raised.exception))

    def test_a_missing_provenance_is_refused_at_nomination_time(self) -> None:
        # The same rule `perf-record` enforces, applied one step earlier: a subject
        # whose origin nobody wrote down cannot be obtained by anybody else.
        path = self._write([{"label": "x", "character": "media-tree", "path": "/tmp"}])
        with self.assertRaises(subjects.SubjectError) as raised:
            subjects.load_nominations(path)
        self.assertIn("provenance", str(raised.exception))

    def test_two_nominations_may_not_share_a_label(self) -> None:
        path = self._write(
            [
                {"label": "x", "character": "media-tree", "path": "/tmp", "provenance": "p"},
                {"label": "x", "character": "package-cache", "path": "/var", "provenance": "p"},
            ]
        )
        with self.assertRaises(subjects.SubjectError):
            subjects.load_nominations(path)

    def test_a_missing_file_says_how_to_make_one(self) -> None:
        with self.assertRaises(subjects.SubjectError) as raised:
            subjects.load_nominations(Path("/nonexistent/subjects.local.json"))
        self.assertIn("gitignored", str(raised.exception))


class RedactionTests(unittest.TestCase):
    def test_no_path_survives_into_the_document(self) -> None:
        """The one property that makes the document committable.

        Nominations hold absolute paths; the loop records ``root_id`` and never a path.
        A leak here would put somebody's directory layout into git.
        """
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch) / "a-very-distinctive-directory-name"
            (root / "nested").mkdir(parents=True)
            (root / "nested" / "file.txt").write_text("x", encoding="utf-8")
            document = subjects.observe(
                [
                    {
                        "label": "tmp",
                        "character": "media-tree",
                        "path": root,
                        "provenance": "made by this test",
                        "reconstructible": True,
                    }
                ]
            )
        encoded = json.dumps(document)
        self.assertNotIn("a-very-distinctive-directory-name", encoded)
        self.assertNotIn(scratch, encoded)
        self.assertEqual(document["subjects"][0]["label"], "tmp")
        self.assertEqual(document["subjects"][0]["files"], 1)


if __name__ == "__main__":
    unittest.main()
