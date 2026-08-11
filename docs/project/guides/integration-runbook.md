# fdu Integration Runbook

A linear walkthrough of setting this repository up and proving, end to end, that the
pieces work together — including that issue tracking survives a synchronization round
trip with its comments and metadata intact.

This is deliberately **not** automated.
`make check` proves the code; this runbook proves the *workflow*: toolchain, build,
tests, goldens, benchmarks, the Python wheel, and issue sync.
Automating it would hide exactly the setup steps a new contributor or agent needs to see
once, and several steps need a human to look at output and judge it.

Run it top to bottom on a fresh clone, or run one section when you have changed the
thing it covers. Every step states what correct looks like, so a failure is unambiguous.

## Conventions

- Commands run from the repository root unless a step says otherwise.
- `✅` marks the observable outcome that means the step passed.
- Steps that write outside the repository (the user cache directory, the tbd sync
  branch) say so before they run.
- Nothing here needs network access except the toolchain install, `tbd sync`, and the
  optional comparison-tool steps.

## 1. Toolchain

```shell
rustc --version            # must satisfy rust-toolchain.toml and the MSRV in Cargo.toml
cargo --version
python3 --version          # 3.11+ for the benchmark harness
uv --version               # optional; only the wheel and benchmark steps use it
node --version             # optional; only the tryscript golden runner uses it
```

✅ `rustc` matches the pinned toolchain; the MSRV job in CI uses the same floor, so a
local version below it will pass here and fail there.

## 2. Build

```shell
make build
```

✅ Debug binaries for the workspace, with no warnings — `warnings = "deny"` is on, so a
warning is a build failure.

```shell
cargo build --locked -p fdu --no-default-features
```

✅ The library builds without the `cli` feature.
This is the shape library consumers get and is otherwise never exercised locally; it
catches a `query` or `index` change that accidentally depends on clap.

## 3. Unit and integration tests

```shell
make test
```

✅ All tests pass. The count grows over time; what matters is zero failures and zero
ignored tests that were not deliberately ignored.

Worth knowing when a failure looks strange:

- Roll-up comparisons must go through `by_ext_named()`, never raw interned extension
  ids. Ids are assigned in first-seen order, so a serial walk and a parallel walk
  legitimately assign them differently — a test comparing `RollUp` values directly fails
  on macOS CI and passes locally, which is exactly how that bug was found.
- Timestamps in tests come from injected constants, never `SystemTime::now()`. A test
  that reads the clock is a test that fails at midnight or in another timezone.

## 4. Golden CLI tests

The golden suite is the text contract for the CLI: every human and machine surface, its
exit codes, and its cache behavior.

```shell
npx tryscript@latest docs        # syntax reference, if you are editing the goldens
make golden                      # or the project's documented golden target
```

✅ Every block matches.
When output legitimately changed, re-record and then **read the diff**: that review is
where golden testing earns its keep, and an unexplained change in a block you did not
intend to touch is a finding, not a nuisance.

Discipline to preserve when adding blocks:

- Classify every field as stable or unstable.
  Paths, sizes, counts, kinds, and schema strings are stable and must match exactly.
  Timestamps, allocated sizes, inode-derived values, and absolute paths are unstable and
  get a **named pattern** (`[SCAN_PATH]`, `[ALLOCATED]`, `[MTIME_NS]`) — never a bare
  `...` elision that hides the whole line.
- Keep the pinned environment (`NO_COLOR`, `LANG`/`LC_ALL=C`, `TZ=UTC`, a scratch
  `XDG_CACHE_HOME`) so a golden cannot depend on the developer’s locale or real cache.
- Machine output is schema-versioned.
  A schema change without a version bump must fail a golden; if you changed a field and
  nothing went red, the schema-bump test is missing.

## 5. Cache behavior by hand

Automated goldens cover this, but running it once by hand is what makes the freshness
model concrete.

```shell
export XDG_CACHE_HOME="$(mktemp -d)"     # never touch your real cache while testing
./target/debug/fdu --json . | head -20   # first run
./target/debug/fdu --json . | head -20   # second run
```

✅ The first run reports `"source": "cold_scan"`; the second reports `"warm_revalidate"`.
Both report the same totals.
If the second run reports a cold scan, the snapshot was not written — check the cache
directory and any warning on stderr.

```shell
ls "$XDG_CACHE_HOME"/fdu                 # a snapshot file exists, named by root hash
```

✅ Exactly one snapshot for the root you scanned.

## 6. Python wheel

```shell
make wheel                               # or: uv build / maturin build
uv run --with dist/fdu-*.whl python crates/fdu-py/tests/smoke.py
```

✅ The smoke test passes against the **installed wheel**, not the source tree.
This is the only step that proves packaging: import paths, the `watch` feature actually
being absent from the wheel, and the console entry point all break here first.

## 7. Benchmarks (optional, and never a pass/fail gate)

```shell
uv run --no-project python -m benchmarks.realtree baseline --root <tree> --label mytree
uv run --no-project python -m benchmarks.realtree measure --root <tree> --label mytree \
    --variant control=<binary> --variant candidate=target/release/examples/perf_probe \
    --job cold-scan-index --job warm-revalidate --trials 12
```

✅ Every trial’s engine digest matches the independent oracle, and the tree fingerprint
is unchanged from first sample to last.
A timing number from a run whose digest disagreed is not a result — it is a bug report.

Benchmarks are a development loop, not CI. Nothing here blocks a merge.

## 8. Issue tracking: the synchronization round trip

This is the step most easily assumed rather than verified.
Beads carry more than a title: status, priority, labels, parent and blocker edges, a
spec path, a close reason, and free-form **notes** that act as the comment stream.
All of it has to survive the trip through git and come back identical, or two agents
working from two clones are working from two different plans.

**This step writes to the shared `tbd-sync` branch.** It is safe and idempotent, but it
is not local-only.

### 8.1 Write metadata locally

```shell
tbd update <bead-id> \
  --notes "Runbook check <date>: what changed and why." \
  --add-label runbook-verified
```

✅ `✓ Updated <bead-id>`.

### 8.2 Push it

```shell
tbd sync
```

✅ `✓ Synced: sent N updated`. A failure here saves to an outbox rather than losing the
change; re-run after fixing the cause rather than re-entering the note.

### 8.3 Read the local side

```shell
tbd show <bead-id> --json > scratch-local-bead.json
```

### 8.4 Read the other side, out of git

Beads live on the `tbd-sync` branch as one Markdown file per issue, with YAML
frontmatter for the structured fields and a `## Notes` section for the comment stream.
Read the file straight out of the branch — this is what another clone will pull:

```shell
git fetch origin tbd-sync
git show origin/tbd-sync --stat | head            # find the file your sync touched
git show origin/tbd-sync:.tbd/data-sync/issues/<internal-id>.md > scratch-remote-bead.md
```

✅ The frontmatter shows your `labels`, `status`, `priority`, `spec_path`, `parent_id`,
`dependencies`, and `close_reason`; the body shows the description and a `## Notes`
section holding the note you just wrote.

### 8.5 Prove both sides match

Eyeballing two files is how a mismatch gets missed.
Compare the fields mechanically:

```shell
python3 scripts/compare_bead_sync.py     # see below; or inline the equivalent
```

✅ Every compared field reports `OK` and the script prints `both sides match`. A mismatch
in `notes` or `labels` means metadata is being dropped somewhere in the round trip,
which is a tooling bug worth stopping for — those fields are where the reasoning lives,
and losing them silently is worse than failing to sync at all.

The comparison this runbook was validated with, kept small on purpose:

```python
import json, pathlib, re

remote = pathlib.Path("scratch-remote-bead.md").read_text()
parsed = json.loads(pathlib.Path("scratch-local-bead.json").read_text())
local = parsed if isinstance(parsed, dict) else parsed[0]

frontmatter = remote.split("---")[1]
notes_remote = remote.split("## Notes", 1)[1].strip() if "## Notes" in remote else ""

checks = {
    "title": (local["title"], re.search(r'title: "(.*)"', frontmatter).group(1)),
    "status": (local["status"], re.search(r"status: (\S+)", frontmatter).group(1)),
    "priority": (str(local["priority"]), re.search(r"priority: (\S+)", frontmatter).group(1)),
    "labels": (",".join(local.get("labels") or []),
               ",".join(re.findall(r"^  - (\S+)$", frontmatter, re.M))),
    "notes": ((local.get("notes") or "").strip(), notes_remote),
}
mismatches = [field for field, (lhs, rhs) in checks.items() if lhs != rhs]
for field, (lhs, rhs) in checks.items():
    print(f"{'OK      ' if lhs == rhs else 'MISMATCH'} {field}: {lhs[:58]!r}")
print("\nRESULT:", "both sides match" if not mismatches else f"MISMATCHES: {mismatches}")
```

### 8.6 Prove a second clone agrees

The strongest form of the check, and the one that catches a stale local database:

```shell
cd <another-clone-or-worktree>
tbd sync                                  # pull
tbd show <bead-id>
```

✅ The note, the label, and the status read the same as they did in step 8.3. Two working
copies now agree, which is the property the whole tracking system rests on.

### 8.7 Clean up

```shell
rm -f scratch-local-bead.json scratch-remote-bead.md
tbd update <bead-id> --remove-label runbook-verified   # optional
```

## 9. The handoff gate

```shell
make check
```

✅ Formatting, clippy (pedantic, `unsafe_code` denied), the full test suite, docs, the
`--no-default-features` library build, and the audit all pass.
If this passes, CI should.

Then, before handing off:

```shell
tbd ready                # what is unblocked next
tbd sync                 # leave no local-only state
git status               # leave no unintended working-tree changes
```

## What this runbook does not cover

- **Performance claims.** Timings live in the experiment ledger with their own protocol;
  nothing here produces a number worth quoting.
- **Cross-platform behavior.** Windows and Linux specifics are proven by the CI matrix,
  not by a local run. A macOS-only pass says nothing about the other two.
- **Watch mode under real churn.** The watch layer has its own tests; exercising a live
  watcher by hand needs a scratch tree and a second terminal, and belongs in its own
  runbook once the CLI surface lands.

* * *

*Part of the fdu project documentation.
See [AGENTS.md](../../../AGENTS.md).*
