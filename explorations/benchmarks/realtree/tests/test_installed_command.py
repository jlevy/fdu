from __future__ import annotations

import json
import shutil
import stat
import sys
import tempfile
import unittest
from contextlib import AbstractContextManager
from pathlib import Path
from typing import Any
from unittest import mock

from benchmarks.realtree import installed_command, provenance


class InstalledCommandTests(unittest.TestCase):
    def _stable_interactive_shell(
        self, selected: Path
    ) -> AbstractContextManager[mock.Mock]:
        """Fix login-shell resolution while retaining real scans and clean-shell lookup."""
        real_run = installed_command.subprocess.run
        marker = "__FDU_EFFECTIVE_RESOLUTION__="

        def run(argv: list[str], *args: Any, **kwargs: Any) -> Any:
            if "-lic" in argv:
                return installed_command.subprocess.CompletedProcess(
                    argv,
                    0,
                    stdout=f"{marker}{selected.resolve()}\n".encode(),
                    stderr=b"",
                )
            return real_run(argv, *args, **kwargs)

        return mock.patch(
            "benchmarks.realtree.installed_command.subprocess.run",
            side_effect=run,
        )

    def _fixture(self, root: Path) -> tuple[Path, dict]:
        executable = root / "bin" / "fdu"
        executable.parent.mkdir()
        report = {
            "complete": True,
            "errors": [],
            "freshness": "fresh",
            "reports": [
                {
                    "summary": {
                        "allocated": 0,
                        "bytes": 0,
                        "dirs": 0,
                        "files": 0,
                        "newest_mtime_ns": None,
                    },
                    "view": "summary",
                }
            ],
            "schema": "fdu.report/1",
            "source": "cold_scan",
        }
        script = (
            f"#!{sys.executable}\n"
            "import json, sys\n"
            "if '--version' in sys.argv:\n"
            "    print('fdu fixture 1.0')\n"
            "else:\n"
            f"    print(json.dumps({report!r}))\n"
        )
        executable.write_text(script, encoding="utf-8")
        executable.chmod(executable.stat().st_mode | stat.S_IXUSR)
        provenance_document = {
            "claim_grade": True,
            "artifacts": {
                "native": {
                    "kind": "native-fdu",
                    "files": provenance._file_identities([executable]),
                }
            },
            "schema": provenance.SCHEMA,
        }
        provenance_document["manifest_id"] = provenance._manifest_id(provenance_document)
        return executable, provenance_document

    @unittest.skipIf(shutil.which("bash") is None, "bash is unavailable")
    def test_capture_and_verify_an_isolated_installed_command(self) -> None:
        with tempfile.TemporaryDirectory() as raw:
            scratch = Path(raw)
            subject = scratch / "subject"
            subject.mkdir()
            executable, provenance_document = self._fixture(scratch)

            with self._stable_interactive_shell(executable):
                document = installed_command.capture(
                    executable=executable,
                    artifact_label="native",
                    kind="native-cargo",
                    root=subject,
                    provenance_document=provenance_document,
                    shells=("bash",),
                )
                verified = installed_command.verify(
                    document,
                    executable=executable,
                    root=subject,
                    provenance_document=provenance_document,
                )

        self.assertTrue(document["claim_grade"])
        self.assertEqual(verified, document)
        self.assertTrue(document["shells"]["bash"]["matches"])
        self.assertNotIn(raw, json.dumps(document))

    @unittest.skipIf(shutil.which("bash") is None, "bash is unavailable")
    def test_attestation_fails_when_the_subject_or_identity_changes(self) -> None:
        with tempfile.TemporaryDirectory() as raw:
            scratch = Path(raw)
            subject = scratch / "subject"
            subject.mkdir()
            executable, provenance_document = self._fixture(scratch)
            with self._stable_interactive_shell(executable):
                document = installed_command.capture(
                    executable=executable,
                    artifact_label="native",
                    kind="native-cargo",
                    root=subject,
                    provenance_document=provenance_document,
                    shells=("bash",),
                )
                tampered = json.loads(json.dumps(document))
                tampered["kind"] = "different"
                with self.assertRaisesRegex(installed_command.InstallationError, "identity"):
                    installed_command.verify(
                        tampered,
                        executable=executable,
                        root=subject,
                        provenance_document=provenance_document,
                    )

                (subject / "new.txt").write_text("changed", encoding="utf-8")
                with self.assertRaisesRegex(installed_command.InstallationError, "contents"):
                    installed_command.verify(
                        document,
                        executable=executable,
                        root=subject,
                        provenance_document=provenance_document,
                    )

    def test_interactive_shell_shadowing_invalidates_capture(self) -> None:
        with tempfile.TemporaryDirectory() as raw:
            scratch = Path(raw)
            subject = scratch / "subject"
            subject.mkdir()
            executable, provenance_document = self._fixture(scratch)
            shell_result = {
                "available": True,
                "matches": True,
                "diagnostic_interactive": {
                    "bytes": 1,
                    "exit_code": 0,
                    "matches": False,
                    "reason": "different path",
                    "sha256": "0" * 64,
                },
            }

            with mock.patch(
                "benchmarks.realtree.installed_command._shell_resolution",
                return_value=shell_result,
            ):
                document = installed_command.capture(
                    executable=executable,
                    artifact_label="native",
                    kind="native-cargo",
                    root=subject,
                    provenance_document=provenance_document,
                    shells=("bash",),
                )

            self.assertFalse(document["claim_grade"])
            self.assertIn("interactive login", "\n".join(document["invalidation_reasons"]))

            with self._stable_interactive_shell(executable):
                valid = installed_command.capture(
                    executable=executable,
                    artifact_label="native",
                    kind="native-cargo",
                    root=subject,
                    provenance_document=provenance_document,
                    shells=("bash",),
                )
            with self._stable_interactive_shell(scratch / "shadowed-fdu"):
                with self.assertRaisesRegex(
                    installed_command.InstallationError, "interactive login shell shadows"
                ):
                    installed_command.verify(
                        valid,
                        executable=executable,
                        root=subject,
                        provenance_document=provenance_document,
                    )

    def test_unverified_provenance_cannot_attest_an_installation(self) -> None:
        with tempfile.TemporaryDirectory() as raw:
            scratch = Path(raw)
            subject = scratch / "subject"
            subject.mkdir()
            executable, provenance_document = self._fixture(scratch)
            provenance_document["claim_grade"] = False
            provenance_document["manifest_id"] = provenance._manifest_id(provenance_document)

            with self._stable_interactive_shell(executable):
                document = installed_command.capture(
                    executable=executable,
                    artifact_label="native",
                    kind="native-cargo",
                    root=subject,
                    provenance_document=provenance_document,
                    shells=("bash",),
                )

            self.assertFalse(document["claim_grade"])
            self.assertIn("provenance:", "\n".join(document["invalidation_reasons"]))
            with self.assertRaisesRegex(
                installed_command.InstallationError, "claim-grade provenance"
            ):
                installed_command.verify(
                    document,
                    executable=executable,
                    root=subject,
                    provenance_document=provenance_document,
                )
            with self._stable_interactive_shell(executable):
                self.assertEqual(
                    installed_command.verify(
                        document,
                        executable=executable,
                        root=subject,
                        provenance_document=provenance_document,
                        require_claim_grade=False,
                    ),
                    document,
                )


if __name__ == "__main__":
    unittest.main()
