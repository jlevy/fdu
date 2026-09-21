"""Check every committed experiment record against itself, and refuse to pass on nothing.

    python -m benchmarks.realtree.validate [--experiments docs/project/experiments]

The other evidence gates prove that the generated views match the records:
``perf-ledger-check`` re-derives the ledger, ``perf-report-check`` the projection and the
page. Neither can see a record that is wrong about itself, because regenerating makes the
views agree with whatever the record says. Twice a verdict carried a cross-job ratio
where its own results held the paired figure, and both shipped green: the gates verified
the laundering, not the evidence.

This gate reads the corpus through the same loader the ledger and the projection use, so
the model's consistency checks run on every record, and then it counts. A validator that
can pass by validating nothing is the same defect one level up -- a harness silently
testing the wrong directory would look like the best possible outcome -- so a run over
zero records, or over records none of which stated a headline to compare, fails.
"""

from __future__ import annotations

import argparse
import sys
from collections.abc import Sequence
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from pydantic import ValidationError

from benchmarks.realtree import experiment as experiment_model
from benchmarks.realtree.summary import EXPERIMENTS_DIR, SummaryError, load_experiments


class CorpusError(RuntimeError):
    """The corpus did not validate, or the validation could not have meant anything."""


@dataclass
class CorpusReport:
    """What the run actually checked, so an empty run cannot be mistaken for a clean one."""

    records: int = 0
    #: Records whose verdict stated a headline that was compared with its measurement.
    compared: int = 0
    #: Records that state `verdict.kept` rather than leaving it to the decision.
    stated_kept: int = 0
    failures: list[str] = field(default_factory=list)

    def summary(self) -> str:
        return (
            f"validated {self.records} records: {self.compared} verdict figures compared "
            f"with their measurements, {self.stated_kept} kept arms stated explicitly"
        )


def validate_corpus(directory: Path) -> CorpusReport:
    """Validate every record under ``directory`` and say what was checked.

    Raises:
        CorpusError: when any record fails, when there are no records, or when no
            record stated a headline -- the last two being the ways a validator passes
            without having validated.
    """
    report = CorpusReport()
    try:
        experiments = load_experiments(directory)
    except SummaryError as error:
        raise CorpusError(str(error)) from error

    for experiment in experiments:
        payload: dict[str, Any] = {
            key: value for key, value in experiment.items() if key != "_path"
        }
        # The loader already ran the model; running it again here is deliberate. The
        # counts below are claims about what this gate checked, and they are only true
        # if this gate did the checking rather than trusting that something else had.
        try:
            validated = experiment_model.Experiment.model_validate(payload)
        except ValidationError as error:
            report.failures.append(f"{experiment['_path']}: {error}")
            continue
        report.records += 1
        if validated.verdict.change_pct is not None:
            report.compared += 1
        if validated.verdict.kept is not None:
            report.stated_kept += 1

    if report.failures:
        raise CorpusError(
            "experiment records do not agree with their own measurements:\n  - "
            + "\n  - ".join(report.failures)
        )
    if report.records == 0:
        raise CorpusError(f"validated no records under {directory}: nothing was checked")
    if report.compared == 0:
        raise CorpusError(
            f"validated {report.records} records but none stated a verdict.change_pct, "
            "so no headline was compared with its measurement: nothing was checked"
        )
    return report


def main(argv: Sequence[str]) -> int:
    parser = argparse.ArgumentParser(prog="benchmarks.realtree.validate", description=__doc__)
    parser.add_argument("--experiments", type=Path, default=EXPERIMENTS_DIR)
    arguments = parser.parse_args(list(argv))
    try:
        report = validate_corpus(arguments.experiments)
    except CorpusError as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    print(report.summary(), file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
