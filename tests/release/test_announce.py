"""Announcement failures must leave registries untouched and incomplete drafts private."""

from __future__ import annotations

import copy
import hashlib
import json
import tempfile
import unittest
import zipfile
from pathlib import Path
from unittest.mock import patch

from scripts.release import announce, maintainer
from scripts.release.maintainer import CommandError, Host, Release, StepError
from scripts.release.registry_state import RegistryState
from tests.release.test_maintainer import DEMO, OTHER_DEMO, declaration, demo_asset, partial_demo
from tests.release.test_publish_gate import VERSION, record_evidence, write_release_set


class GitHub(Host):
    """Simulate partial uploads and GitHub's digest-bearing release records."""

    def __init__(self, release: Release) -> None:
        self.release = release
        self.record = None
        self.writes = []
        self.interrupt = False
        # Files at the release commit, byte for byte, keyed by path: the demo declaration.
        self.tree = {}

    def run(self, argv, **kwargs):
        if argv[:2] == ["git", "ls-tree"]:
            assert argv[2:6] == ["-z", "--full-tree", self.release.commit, "--"], argv
            return "".join(
                f"100644 blob {hashlib.sha1(self.tree[path]).hexdigest()}\t{path}\0"
                for path in argv[6:]
                if path in self.tree
            )
        if argv[:2] == ["gh", "api"]:
            if "/releases/tags/" in argv[-1] and (self.record is None or self.record["draft"]):
                raise CommandError(argv, 1, "HTTP 404")
            if argv[2:4] != ["--paginate", "--slurp"]:
                raise AssertionError(argv)
            return json.dumps([[self.record] if self.record else []])
        self.writes.append(argv)
        operation = argv[2]
        if operation == "create":
            self.record = {
                "tag_name": argv[3],
                "name": argv[argv.index("--title") + 1],
                "body": Path(argv[argv.index("--notes-file") + 1]).read_text(),
                "prerelease": False,
                "draft": "--draft" in argv,
                "assets": [],
                "html_url": "https://example.test/release",
            }
        elif operation == "upload":
            # Releases here are immutable: GitHub refuses any asset once one is published.
            if not self.record["draft"]:
                raise CommandError(
                    argv, 1, "HTTP 422: Cannot upload assets to an immutable release"
                )
            for filename in argv[4 : argv.index("--repo")]:
                path = Path(filename)
                self.record["assets"].append(
                    {
                        "name": path.name,
                        "size": path.stat().st_size,
                        "digest": f"sha256:{maintainer.inspect_artifacts.digest(path)}",
                        "state": "uploaded",
                    }
                )
                if self.interrupt:
                    self.interrupt = False
                    raise CommandError(argv, 1, "upload interrupted")
        elif operation == "edit":
            self.record["draft"] = False
        else:
            raise AssertionError(argv)
        return ""

    def run_bytes(self, argv, **kwargs):
        if argv[:3] != ["git", "cat-file", "blob"]:
            raise AssertionError(argv)
        (content,) = (
            blob for blob in self.tree.values() if hashlib.sha1(blob).hexdigest() == argv[3]
        )
        return content


class AnnouncementTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        root = Path(self.temp.name)
        self.release = Release(VERSION, "a" * 40, root, root)
        files = root / "published/files"
        files.mkdir(parents=True)
        evidence = root / "published/evidence"
        evidence.mkdir()
        write_release_set(files)
        record_evidence(files, evidence)
        (root / "notes.md").write_text("Release notes.\n")
        self.host = GitHub(self.release)
        self.states = [
            RegistryState("crates.io", "fdu-core", VERSION, "identical", "match"),
            RegistryState("crates.io", "fdu", VERSION, "identical", "match"),
            RegistryState("pypi", "fdu", VERSION, "identical", "match"),
        ]
        for name, value in (
            ("require_pushed_tag", None),
            ("show", "Release notes.\n<!-- footer -->\n"),
            ("registry_states", self.states),
        ):
            mock = patch.object(maintainer, name, return_value=value)
            mock.start()
            self.addCleanup(mock.stop)

    def test_complete_release_and_identical_rerun(self):
        announce.announce(self.host, self.release)
        self.assertFalse(self.host.record["draft"])
        self.assertEqual(len(self.host.record["assets"]), 11)
        self.assertFalse((self.release.directory / "published/media").exists())
        writes = copy.deepcopy(self.host.writes)
        announce.announce(self.host, self.release)
        self.assertEqual(self.host.writes, writes)

    def removal(self, name):
        return f"gh release delete-asset {self.release.tag} {name} --repo jlevy/fdu"

    def prepare_draft(self, *assets):
        """The draft `make release-demo` leaves: the tagged notes as body, the video only."""
        self.host.record = {
            "tag_name": self.release.tag,
            "name": f"fdu {VERSION}",
            "body": "Release notes.\n",
            "prerelease": False,
            "draft": True,
            "assets": list(assets),
            "html_url": "https://example.test/release",
        }

    def test_a_draft_holding_the_declared_demo_is_completed_and_published(self):
        self.host.tree[maintainer.DEMO_DECLARATION] = declaration()
        self.prepare_draft(demo_asset())
        announce.announce(self.host, self.release)
        self.assertFalse(self.host.record["draft"])
        self.assertEqual(len(self.host.record["assets"]), 12)
        # The job creates nothing and uploads only the eleven files; it has no video.
        self.assertEqual([argv[2] for argv in self.host.writes], ["upload", "edit"])
        (upload,) = [argv for argv in self.host.writes if argv[2] == "upload"]
        self.assertEqual(len(upload[4 : upload.index("--repo")]), 11)
        self.assertFalse(any("fdu-demo" in arg for arg in upload))
        writes = copy.deepcopy(self.host.writes)
        announce.announce(self.host, self.release)
        self.assertEqual(self.host.writes, writes)

    def test_a_draft_without_the_declared_demo_is_completed_but_never_published(self):
        self.host.tree[maintainer.DEMO_DECLARATION] = declaration()
        with self.assertRaisesRegex(StepError, "run `make release-demo`.*then rerun this job"):
            announce.announce(self.host, self.release)
        self.assertTrue(self.host.record["draft"])
        self.assertEqual(len(self.host.record["assets"]), 11)
        self.assertNotIn("edit", [argv[2] for argv in self.host.writes])
        # Once `make release-demo` attaches it to the draft, the rerun publishes twelve.
        self.host.record["assets"].append(demo_asset())
        self.host.writes.clear()
        announce.announce(self.host, self.release)
        self.assertFalse(self.host.record["draft"])
        self.assertEqual(len(self.host.record["assets"]), 12)
        self.assertEqual([argv[2] for argv in self.host.writes], ["edit"])

    def test_a_published_release_without_its_declared_demo_is_refused_for_good(self):
        announce.announce(self.host, self.release)
        self.host.tree[maintainer.DEMO_DECLARATION] = declaration()
        self.host.writes.clear()
        with self.assertRaisesRegex(StepError, "published without the declared fdu-demo.mp4"):
            announce.announce(self.host, self.release)
        self.assertEqual(self.host.writes, [])

    def test_a_demo_video_the_commit_does_not_declare_is_refused(self):
        announce.announce(self.host, self.release)
        for draft in (False, True):
            self.host.record["draft"] = draft
            self.host.record["assets"] = [*self.host.record["assets"][:11], demo_asset()]
            self.host.writes.clear()
            with self.assertRaisesRegex(StepError, "unexpected assets"):
                announce.announce(self.host, self.release)
            self.assertEqual(self.host.writes, [])

    def test_a_demo_video_other_than_declared_is_refused_with_its_removal(self):
        self.host.tree[maintainer.DEMO_DECLARATION] = declaration()
        for asset in (
            demo_asset(DEMO + b"\0"),
            demo_asset(OTHER_DEMO),
            {**demo_asset(), "digest": None},
            partial_demo(),
        ):
            with self.subTest(asset=asset):
                self.prepare_draft(asset)
                self.host.writes.clear()
                with self.assertRaisesRegex(
                    StepError, f"conflicts with {maintainer.DEMO_DECLARATION}: fdu-demo.mp4"
                ) as raised:
                    announce.announce(self.host, self.release)
                self.assertIn(self.removal("fdu-demo.mp4"), str(raised.exception))
                self.assertEqual(self.host.writes, [])
                self.assertTrue(self.host.record["draft"])

    def test_an_unfinished_upload_of_one_of_the_eleven_is_refused_with_its_removal(self):
        self.host.interrupt = True
        with self.assertRaises(CommandError):
            announce.announce(self.host, self.release)
        (asset,) = self.host.record["assets"]
        asset["state"] = "starter"
        self.host.writes.clear()
        with self.assertRaisesRegex(StepError, "state is starter, not uploaded") as raised:
            announce.announce(self.host, self.release)
        self.assertIn(self.removal(asset["name"]), str(raised.exception))
        self.assertEqual(self.host.writes, [])

    def test_a_malformed_declaration_stops_before_any_write(self):
        for content in (b"{}\n", declaration().replace(b"fdu-demo.mp4", b"fdu-demo.gif")):
            self.host.tree[maintainer.DEMO_DECLARATION] = content
            with self.assertRaisesRegex(StepError, maintainer.DEMO_DECLARATION):
                announce.announce(self.host, self.release)
            self.assertEqual(self.host.writes, [])
            self.assertIsNone(self.host.record)

    def test_interrupted_upload_resumes_only_missing_assets(self):
        self.host.interrupt = True
        with self.assertRaises(CommandError):
            announce.announce(self.host, self.release)
        self.assertTrue(self.host.record["draft"])
        self.assertEqual(len(self.host.record["assets"]), 1)
        announce.announce(self.host, self.release)
        self.assertFalse(self.host.record["draft"])
        self.assertEqual(len(self.host.record["assets"]), 11)

    def test_registry_conflict_or_corrupt_file_prevents_any_write(self):
        with (
            patch.object(
                maintainer,
                "registry_states",
                return_value=[RegistryState("pypi", "fdu", VERSION, "conflict", "hash mismatch")],
            ),
            self.assertRaises(StepError),
        ):
            announce.announce(self.host, self.release)
        self.assertEqual(self.host.writes, [])
        wheel = next((self.release.directory / "published/files").glob("*.whl"))
        with zipfile.ZipFile(wheel, "a") as archive:
            archive.writestr("tampered.txt", "corrupt")
        with self.assertRaises(StepError):
            announce.announce(self.host, self.release)
        self.assertEqual(self.host.writes, [])

    def test_conflicting_existing_release_is_never_overwritten(self):
        announce.announce(self.host, self.release)
        record = copy.deepcopy(self.host.record)
        for field, value in (("body", "changed"), ("name", "wrong title"), ("prerelease", True)):
            self.host.record = copy.deepcopy(record)
            self.host.record[field] = value
            self.host.writes.clear()
            with self.assertRaises(StepError):
                announce.announce(self.host, self.release)
            self.assertEqual(self.host.writes, [])
        self.host.record = record
        self.host.record["assets"][0]["digest"] = None
        self.host.writes.clear()
        with self.assertRaises(StepError):
            announce.announce(self.host, self.release)
        self.assertEqual(self.host.writes, [])

    def test_lookup_includes_drafts_on_later_pages_and_fails_closed(self):
        record = {"tag_name": self.release.tag, "draft": True}
        with patch.object(
            self.host, "run", return_value=json.dumps([[{"tag_name": "v9.0.0"}], [record]])
        ):
            self.assertEqual(maintainer.release_record(self.host, self.release), record)
        with (
            patch.object(self.host, "run", return_value=json.dumps([[record], [record]])),
            self.assertRaises(StepError),
        ):
            maintainer.release_record(self.host, self.release)
        with (
            patch.object(self.host, "run", side_effect=CommandError(["gh"], 1, "HTTP 403")),
            self.assertRaises(CommandError),
        ):
            announce.announce(self.host, self.release)
        self.assertEqual(self.host.writes, [])

    def test_notes_must_match_the_tagged_text(self):
        (self.release.directory / "notes.md").write_text("Invented notes")
        with self.assertRaises(ValueError):
            announce.announce(self.host, self.release)
        self.assertEqual(self.host.writes, [])


if __name__ == "__main__":
    unittest.main()
