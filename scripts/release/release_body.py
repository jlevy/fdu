#!/usr/bin/env python3
"""Derive and check the GitHub release body from a notes file.

GitHub treats a single newline in a release body as a hard line break, so the body is
the notes with HTML comments removed and each paragraph unwrapped. The HTML-comment
count and the whitespace-only difference against the unwrapped body are checked here.
The GitHub markdown `<br>` check stays a documented `gh api` call: it needs the
network (fdu-ixtn).
"""

from __future__ import annotations

import argparse
import subprocess
import sys
import tempfile
from collections.abc import Callable
from pathlib import Path

# Release notes keep one HTML comment: the common-doc-guidelines footer. A second
# comment is an unfilled draft placeholder the one-liner used to strip silently.
EXPECTED_HTML_COMMENTS = 1

Unwrap = Callable[[str], str]


def require(condition: bool, message: str) -> None:
    """Raise a stable validation error when the release-body contract is violated."""
    if not condition:
        raise ValueError(message)


def _tick_run(text: str, index: int) -> int:
    """Count consecutive backticks starting at `index`."""
    length = 0
    while index + length < len(text) and text[index + length] == "`":
        length += 1
    return length


def _fence_span(text: str, index: int) -> tuple[str, int] | None:
    """Return `(marker, span)` when `index` is a CommonMark fence opener or closer."""
    if index != 0 and text[index - 1] != "\n":
        return None
    indent = 0
    while indent < 3 and index + indent < len(text) and text[index + indent] == " ":
        indent += 1
    cursor = index + indent
    if cursor >= len(text) or text[cursor] not in "`~":
        return None
    mark = text[cursor]
    length = 0
    while cursor + length < len(text) and text[cursor + length] == mark:
        length += 1
    if length < 3:
        return None
    return mark * length, indent + length


def scan_html_comments(markdown: str, *, strip: bool) -> tuple[str, int]:
    """Walk `markdown`, counting HTML comments that are not inside code.

    Fenced blocks and inline code spans keep their text, including a `<!--` that is
    documentation rather than a draft comment. When `strip` is true, real comments
    and the newlines that follow them are omitted, matching the previous one-liner.
    """
    output: list[str] = []
    comments = 0
    index = 0
    length = len(markdown)
    fence: str | None = None
    inline_ticks = 0
    while index < length:
        if fence is not None:
            closing = _fence_span(markdown, index)
            if closing is not None and closing[0][0] == fence[0] and len(closing[0]) >= len(fence):
                marker, span = closing
                output.append(markdown[index : index + span])
                index += span
                fence = None
                continue
            output.append(markdown[index])
            index += 1
            continue
        if inline_ticks:
            run = _tick_run(markdown, index)
            if run == inline_ticks:
                output.append("`" * run)
                index += run
                inline_ticks = 0
                continue
            output.append(markdown[index])
            index += 1
            continue
        opening = _fence_span(markdown, index)
        if opening is not None:
            marker, span = opening
            fence = marker
            output.append(markdown[index : index + span])
            index += span
            continue
        if markdown.startswith("<!--", index):
            end = markdown.find("-->", index + 4)
            require(end != -1, "unclosed HTML comment")
            comments += 1
            close = end + 3
            if strip:
                index = close
                while index < length and markdown[index] == "\n":
                    index += 1
            else:
                output.append(markdown[index:close])
                index = close
            continue
        if markdown[index] == "`":
            inline_ticks = _tick_run(markdown, index)
            output.append("`" * inline_ticks)
            index += inline_ticks
            continue
        output.append(markdown[index])
        index += 1
    return "".join(output), comments


def strip_html_comments(markdown: str) -> str:
    """Return `markdown` with HTML comments outside code removed."""
    text, _count = scan_html_comments(markdown, strip=True)
    return text


def html_comment_count(markdown: str) -> int:
    """Count HTML comments that are not inside fenced or inline code."""
    _text, count = scan_html_comments(markdown, strip=False)
    return count


def fold_whitespace(text: str) -> str:
    """Squeeze whitespace the way `tr -s '[:space:]' ' '` then strip does."""
    return " ".join(text.split())


def check_release_body(notes: str, source: str, body: str) -> None:
    """Require one real HTML comment and a whitespace-only unwrap."""
    count = html_comment_count(notes)
    require(
        count == EXPECTED_HTML_COMMENTS,
        f"expected {EXPECTED_HTML_COMMENTS} HTML comment (the guideline footer), found {count}",
    )
    require(
        fold_whitespace(source) == fold_whitespace(body),
        "unwrapped release body differs from stripped notes by more than whitespace",
    )


def unwrap_with_flowmark(source: str, *, root: Path) -> str:
    """Join wrapped paragraphs with the repository's pinned flowmark."""
    with tempfile.TemporaryDirectory() as directory:
        incoming = Path(directory) / "notes-source.md"
        outgoing = Path(directory) / "notes.md"
        incoming.write_text(source, encoding="utf-8")
        subprocess.run(
            [
                "uv",
                "run",
                "--project",
                str(root / "explorations" / "benchmarks"),
                "--frozen",
                "--only-group",
                "docs",
                "flowmark",
                "--width",
                "0",
                "--output",
                str(outgoing),
                str(incoming),
            ],
            check=True,
            cwd=root,
        )
        return outgoing.read_text(encoding="utf-8")


def derive_release_body(
    notes: str,
    *,
    unwrap: Unwrap,
) -> tuple[str, str]:
    """Return `(stripped source, unwrapped body)` for `notes`."""
    source = strip_html_comments(notes)
    return source, unwrap(source)


def parser() -> argparse.ArgumentParser:
    """Build the command-line parser."""
    result = argparse.ArgumentParser()
    result.add_argument("--notes", type=Path, required=True)
    result.add_argument("--source", type=Path, required=True)
    result.add_argument("--body", type=Path, required=True)
    result.add_argument("--root", type=Path, default=Path.cwd())
    result.add_argument(
        "--unwrap",
        choices=("flowmark", "identity"),
        default="flowmark",
        help="flowmark is the release path; identity is for tests that do not unwrap",
    )
    return result


def main(argv: list[str] | None = None) -> int:
    """Check the stripped source and unwrapped body, then write both."""
    args = parser().parse_args(argv)
    notes = args.notes.read_text(encoding="utf-8")
    root = args.root.resolve()

    def unwrap(text: str) -> str:
        if args.unwrap == "identity":
            return text
        return unwrap_with_flowmark(text, root=root)

    source, body = derive_release_body(notes, unwrap=unwrap)
    try:
        check_release_body(notes, source, body)
    except ValueError as exc:
        print(exc, file=sys.stderr)
        return 1
    args.source.parent.mkdir(parents=True, exist_ok=True)
    args.body.parent.mkdir(parents=True, exist_ok=True)
    args.source.write_text(source, encoding="utf-8")
    args.body.write_text(body, encoding="utf-8")
    return 0


if __name__ == "__main__":
    sys.exit(main())
