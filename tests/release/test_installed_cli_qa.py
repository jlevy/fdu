"""Tests for the installed-CLI QA harness's reading of `/usr/bin/time` output."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from scripts import run_installed_cli_qa as qa


def parsed(text: str) -> tuple[float | None, float | None]:
    with tempfile.TemporaryDirectory() as temporary:
        path = Path(temporary) / "time.txt"
        path.write_text(text, encoding="utf-8")
        return qa.parse_time_file(path)


class TimeFileTests(unittest.TestCase):
    def test_gnu_output_after_a_non_zero_exit_reads_the_labelled_fields(self) -> None:
        # GNU time prefixes this line on any non-zero exit; its status once became the wall
        # time, so a 0.02 s usage error read as 2.000 s (fdu-6zsp).
        real, rss = parsed("Command exited with non-zero status 2\nreal 0.02\nmaxrss 6144\n")
        self.assertEqual(real, 0.02)
        self.assertEqual(rss, 6.0)

    def test_gnu_output_after_a_clean_exit(self) -> None:
        self.assertEqual(parsed("real 1.25\nmaxrss 2048\n"), (1.25, 2.0))

    def test_a_signal_line_is_not_a_time_either(self) -> None:
        text = "Command terminated by signal 9\nreal 3.50\nmaxrss 1024\n"
        self.assertEqual(parsed(text), (3.5, 1.0))

    def test_bsd_output(self) -> None:
        text = (
            "        0.02 real         0.00 user         0.00 sys\n"
            "             2097152  maximum resident set size\n"
            "                   0  average shared memory size\n"
        )
        self.assertEqual(parsed(text), (0.02, 2.0))

    def test_a_missing_or_empty_file_has_no_numbers(self) -> None:
        self.assertEqual(parsed(""), (None, None))


if __name__ == "__main__":
    unittest.main()
