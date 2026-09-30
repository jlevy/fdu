# Ported: the external volume path comes from $EXTERNAL_VOLUME instead of a literal.
#!/usr/bin/env python3
"""Copy result JSON files into a results directory with private absolute paths removed.

  sanitize.py SRC_DIR DEST_DIR PREFIX [PREFIX...]

Every string value is rewritten: each PREFIX (an absolute path) becomes "<fixtures>", the
macOS per-user temp dir becomes "<user-temp>", and any remaining "/Users/<name>" becomes
"/Users/<user>". The `mount` dump recorded by semantics.py is reduced to the lines about
the fixture's volume. Files are written with the same basename.
"""
import json
import os
import re
import sys
# The repository root, for the shared atomic writer in scripts/atomic_write.py.
sys.path.insert(0, os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "..", "..")))
from scripts.atomic_write import write_text_atomic  # noqa: E402


def main():
    src, dest, prefixes = sys.argv[1], sys.argv[2], sorted(sys.argv[3:], key=len, reverse=True)
    os.makedirs(dest, exist_ok=True)
    user_tmp = os.popen("getconf DARWIN_USER_TEMP_DIR").read().strip().rstrip("/")

    def fix(s):
        for p in prefixes:
            s = s.replace(p.rstrip("/"), "<fixtures>")
        if user_tmp:
            s = s.replace(user_tmp, "<user-temp>")
        s = re.sub(r"/Users/[^/\s]+", "/Users/<user>", s)
        s = s.replace(os.environ.get("EXTERNAL_VOLUME", "/Volumes/external"), "<external-volume>")
        s = re.sub(r"/private/var/folders/[^\s]+", "<user-temp>", s)
        return s

    def walk(o, key=None):
        if isinstance(o, dict):
            return {k: walk(v, k) for k, v in o.items()}
        if isinstance(o, list):
            return [walk(v) for v in o]
        if isinstance(o, str):
            if key == "mount":
                return "\n".join(fix(l) for l in o.splitlines() if os.path.basename(os.environ.get("EXTERNAL_VOLUME", "/Volumes/external")) in l or "/System/Volumes/Data " in l or "dsverify" in l)
            return fix(o)
        return o

    for name in sorted(os.listdir(src)):
        if not name.endswith(".json"):
            continue
        with open(os.path.join(src, name)) as fh:
            data = json.load(fh)
        write_text_atomic(os.path.join(dest, name), json.dumps(walk(data), indent=1))
        print("sanitized", name)


if __name__ == "__main__":
    main()
