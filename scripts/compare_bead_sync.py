"""Compare one bead's local database state against its synced git representation."""

import json
import pathlib
import re

remote = pathlib.Path("scratch-remote-bead.md").read_text()
parsed = json.loads(pathlib.Path("scratch-local-bead.json").read_text())
local = parsed if isinstance(parsed, dict) else parsed[0]

frontmatter = remote.split("---")[1]
notes_remote = remote.split("## Notes", 1)[1].strip() if "## Notes" in remote else ""

checks = {
    "title": (local["title"], re.search(r'title: "(.*)"', frontmatter).group(1)),
    "status": (local["status"], re.search(r"status: (\S+)", frontmatter).group(1)),
    "priority": (str(local["priority"]), re.search(r"priority: (\S+)", frontmatter).group(1)),
    "labels": (
        ",".join(local.get("labels") or []),
        ",".join(re.findall(r"^  - (\S+)$", frontmatter, re.M)),
    ),
    "notes": ((local.get("notes") or "").strip(), notes_remote),
}

mismatches = [field for field, (lhs, rhs) in checks.items() if lhs != rhs]
for field, (lhs, rhs) in checks.items():
    marker = "OK      " if lhs == rhs else "MISMATCH"
    print(f"{marker} {field}: {lhs[:58]!r}")

print()
print("RESULT:", "both sides match" if not mismatches else f"MISMATCHES: {mismatches}")
