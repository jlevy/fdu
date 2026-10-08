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
from tests.release.test_maintainer import DEMO
from tests.release.test_publish_gate import VERSION, record_evidence, write_release_set


class GitHub(Host):
    """Simulate partial uploads and GitHub's digest-bearing release records."""

    def __init__(self, release: Release) -> None:
        self.release = release
        self.record = None
        self.writes = []
        self.interrupt = False
        # Binary files at the release commit, keyed by path.
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
            for filename in argv[4 : argv.index("--repo")]:
                path = Path(filename)
                self.record["assets"].append(
                    {
                        "name": path.name,
                        "size": path.stat().st_size,
                        "digest": f"sha256:{maintainer.inspect_artifacts.digest(path)}",
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

    def test_a_commit_s_demo_video_is_uploaded_and_verified_as_a_twelfth_asset(self):
        self.host.tree[maintainer.DEMO_PATH] = DEMO
        announce.announce(self.host, self.release)
        self.assertFalse(self.host.record["draft"])
        assets = {asset["name"]: asset for asset in self.host.record["assets"]}
        self.assertEqual(len(assets), 12)
        digest = f"sha256:{hashlib.sha256(DEMO).hexdigest()}"
        self.assertEqual(
            assets["fdu-demo.mp4"], {"name": "fdu-demo.mp4", "size": len(DEMO), "digest": digest}
        )
        (upload,) = [argv for argv in self.host.writes if argv[2] == "upload"]
        self.assertIn(str(self.release.directory / "published/media/fdu-demo.mp4"), upload)
        writes = copy.deepcopy(self.host.writes)
        announce.announce(self.host, self.release)
        self.assertEqual(self.host.writes, writes)

    def test_a_demo_video_the_commit_lacks_is_refused(self):
        announce.announce(self.host, self.release)
        demo = {"name": "fdu-demo.mp4", "size": len(DEMO), "digest": "sha256:" + "0" * 64}
        for draft in (False, True):
            self.host.record["draft"] = draft
            self.host.record["assets"] = [*self.host.record["assets"][:11], demo]
            self.host.writes.clear()
            with self.assertRaisesRegex(StepError, "unexpected assets"):
                announce.announce(self.host, self.release)
            self.assertEqual(self.host.writes, [])

    def test_a_demo_video_asset_of_other_bytes_is_refused(self):
        self.host.tree[maintainer.DEMO_PATH] = DEMO
        self.host.interrupt = True
        with self.assertRaises(CommandError):
            announce.announce(self.host, self.release)
        good = hashlib.sha256(DEMO).hexdigest()
        for size, digest in ((len(DEMO) + 1, good), (len(DEMO), "0" * 64)):
            self.host.record["assets"] = [
                {"name": "fdu-demo.mp4", "size": size, "digest": f"sha256:{digest}"}
            ]
            self.host.writes.clear()
            with self.assertRaisesRegex(StepError, "conflicts with verified files: fdu-demo.mp4"):
                announce.announce(self.host, self.release)
            self.assertEqual(self.host.writes, [])
            self.assertTrue(self.host.record["draft"])

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
            self.assertEqual(announce.release_record(self.host, self.release), record)
        with (
            patch.object(self.host, "run", return_value=json.dumps([[record], [record]])),
            self.assertRaises(StepError),
        ):
            announce.release_record(self.host, self.release)
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
