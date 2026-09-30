# Local development workflows. `make check` is the handoff gate: if it passes, CI should.

.DEFAULT_GOAL := help

CARGO ?= cargo
NODE ?= node
NPM ?= npm
UV ?= uv
MSRV ?= 1.85.0
NODE_INSTALL_STAMP := node_modules/.package-lock.json

# Where cargo writes build output, asked of cargo rather than assumed to be `target/`:
# CARGO_TARGET_DIR and build.target-dir both move it, and a binary left behind at the
# assumed path would then be tested or measured in place of the one just built
# (fdu-bi9a, fdu-dfbu). Evaluated once, on first use, so a target that never builds never
# asks. CARGO_TARGET_DIR, then `$(CURDIR)/target`, are only the fallback for a cargo that
# cannot answer, as in scripts/cargo-target.mjs, and then the build before it fails first.
CARGO_TARGET = $(eval CARGO_TARGET := $(or $(shell $(CARGO) metadata --format-version 1 --no-deps 2>/dev/null | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p'),$(if $(CARGO_TARGET_DIR),$(abspath $(CARGO_TARGET_DIR))),$(CURDIR)/target))$(CARGO_TARGET)
DEBUG_FDU = $(CARGO_TARGET)/debug/fdu

# A target directory shared by several checkouts serves one checkout's build to another.
# Cargo judges a workspace crate fresh when its outputs are newer than its sources, and
# another checkout's newer outputs pass that test however different its sources are: a
# test binary then runs without the code under test, and nothing prints `Compiling`
# (fdu-8whh). So every target that builds a workspace crate first records which checkout
# owns the target directory. When the owner changes, or is unrecorded -- as for any
# directory built before this check -- it removes the workspace crates' fingerprints, and
# cargo rebuilds them from this checkout's sources. Dependencies keep theirs: they are
# registry releases, identical in every checkout. The `fdu-*` pattern names exactly the
# workspace packages (fdu, fdu-core, fdu-py); no dependency's name starts with it.
TARGET_OWNER_STAMP = $(CARGO_TARGET)/.fdu-checkout

target-owner:
	@target="$(CARGO_TARGET)"; stamp="$(TARGET_OWNER_STAMP)"; \
	owner="$$(cat "$$stamp" 2>/dev/null)"; \
	if [ -d "$$target" ] && [ "$$owner" != "$(CURDIR)" ]; then \
		echo "note: $$target was last built from $${owner:-an unrecorded checkout};"; \
		echo "      removing its fdu crate fingerprints so cargo rebuilds them from $(CURDIR)"; \
		find "$$target" -maxdepth 3 -type d -name .fingerprint -prune \
			-exec sh -c 'for dir; do rm -rf -- "$$dir"/fdu-*; done' sh {} + || exit 1; \
	fi; \
	mkdir -p "$$target" && printf '%s\n' "$(CURDIR)" > "$$stamp"

# Every target whose recipe compiles a workspace crate, through cargo or maturin; the
# recipe-coverage test in scripts/cargo-target.test.mjs keeps this list complete.
TARGET_OWNER_TARGETS := build release rust-test reference-model opened-root-golden \
	opened-root-golden-update yaml-selfcheck performance-probe clippy cross-lint docs \
	lib-only msrv fix parity-venv python-check python-concurrency python-smoke \
	release-rehearse cli perf-probe-release perf-probe-profiling

$(TARGET_OWNER_TARGETS): target-owner

.PHONY: help target-owner build release test rust-test reference-model test-golden opened-root-golden opened-root-golden-lint opened-root-golden-update golden-invocations golden-observability portability parity-venv test-parity parity-check parity-update test-path-independence path-independence path-independence-full path-independence-record content-selfcheck yaml-selfcheck performance-probe test-performance golden-update check uv-version permission-bits supply-chain rust-module-names admission-sites atomic-writes fix fmt fmt-check clippy docs docs-format docs-format-check lib-only msrv audit npm-audit python-check python-concurrency python-smoke python-sdist-smoke wheel-python release-test test-terminal release-rehearse semver-check release-preflight release-candidate release-body release-verify-tag release-published release-announced release-cleanup release-audit clean cli perf-help verify-beads

help:
	@echo "make build      Debug build of the core library and CLI, all features"
	@echo "make release    Optimized build of the core library and CLI"
	@echo "make test       Run Rust, CLI golden, and performance-harness tests"
	@echo "make reference-model  Compare generated index transitions with the independent model"
	@echo "make test-golden  Build and compare the CLI golden contract"
	@echo "make opened-root-golden  Compare the transparent opened-root sessions"
	@echo "make opened-root-golden-update SCENARIO=name  Update one opened-root session"
	@echo "make golden-invocations  Check the corpus never resolves fdu through PATH"
	@echo "make golden-observability  Reject goldens that hide product output behind parsers"
	@echo "make portability  Check committed test data names no machine"
	@echo "make test-parity  Replay the corpus against the Python surface"
	@echo "make path-independence  Check answers against cold runs across histories (subset)"
	@echo "make path-independence-full  The full matrix; path-independence-record rewrites the registry"
	@echo "make parity-update  Re-record the Python surface deviations"
	@echo "make content-selfcheck  Analyze an archive of tracked repository files"
	@echo "make test-performance  Test the performance harness and every fdu probe job"
	@echo "make golden-update  Regenerate intentional golden changes, then compare"
	@echo "make check      Handoff gate: tests, audits, docs, and installed-wheel smoke"
	@echo "make supply-chain  Verify release age, provenance, pins, and CI trust controls"
	@echo "make rust-module-names  Check Rust source filenames for ambiguity"
	@echo "make admission-sites  Check every filesystem producer routes through admission"
	@echo "make atomic-writes  Check every file is written whole, through the atomic helpers"
	@echo "make msrv       Compile all features and test the core contract on Rust $(MSRV)"
	@echo "make fix        Apply formatting and machine-applicable lint fixes"
	@echo "make audit      Dependency advisory and license audit (needs cargo-deny)"
	@echo "make python-concurrency  Prove Python GIL release and runtime borrow exclusion"
	@echo "make python-smoke  Build, install, and smoke-test the locked Python wheel"
	@echo "make release-rehearse  Build and inspect this host's release artifacts locally"
	@echo "make semver-check  Check the Rust API against the last compatible release (needs cargo-semver-checks)"
	@echo "make release-preflight  First maintainer release step (docs/project/guides/release-process.md)"
	@echo "make cli        Build and run the CLI against this repo"
	@echo "make docs-format  Auto-format all Markdown with flowmark"
	@echo ""
	@echo "Performance loop (not part of check; see docs/project/guides/performance-loop.md)"
	@echo "make perf-baseline  Fingerprint the reference tree named by PERF_TREE"
	@echo "make perf-profile   Attribute time to functions on a symbol-bearing build"
	@echo "make perf-compare   Measure a candidate against CONTROL, interleaved and paired"
	@echo "make perf-content-profile  Attribute basic content, cache-hit, and query time"
	@echo "make perf-content-compare  Compare content jobs in 12 paired trials"
	@echo "make perf-store RUN=run.json OUT=.../run.json.gz  Commit a run, gzipped deterministically"
	@echo "make perf-test      Test the real-tree harness itself"
	@echo "make perf-ledger    Regenerate the experiment ledger from its artifacts"
	@echo "make perf-report    Regenerate the charted performance report from the same artifacts"
	@echo "make perf-report-check  Fail if the committed report has drifted from the artifacts"
	@echo "make perf-evidence-check  Fail if any experiment record disagrees with its own measurements"

build:
	$(CARGO) build --locked -p fdu --all-features

release:
	$(CARGO) build --locked --release -p fdu --all-features

test: rust-test test-golden content-selfcheck yaml-selfcheck test-performance

rust-test:
	$(CARGO) test --locked --all-features

# The permission fixtures induce a real EACCES by removing read or search permission and
# assert that it happened. A process not subject to mode bits — anything running as
# root, the normal case inside a container or a Claude Code web session — reads the file
# anyway, so the tests refuse to run those fixtures and panic rather than pass vacuously.
# Each panic names the cause, but a dozen of them across three crates reads as a dozen
# unrelated failures; the uv floor once cost a session the same way. Probe the way the
# tests do — a mode-000 file this process can still read — and say it once, before any
# test target runs. The opt-out is the one the tests honour, and CI leaves it unset so a
# passing test proves its assertions ran. Windows has no mode bits to enforce, and the
# fixtures are Unix-only there.
permission-bits:
	@case "$$(uname -s)" in MINGW*|MSYS*|CYGWIN*|Windows_NT) exit 0;; esac; \
	if [ "$${FDU_TEST_ALLOW_NO_PERMISSION_BITS:-}" = 1 ]; then exit 0; fi; \
	probe="$$(mktemp "$${TMPDIR:-/tmp}/fdu-permission.XXXXXX")" || exit 1; \
	chmod 000 "$$probe"; \
	if cat "$$probe" >/dev/null 2>&1; then \
		rm -f "$$probe"; \
		echo "error: this process can read a mode-000 file, so the host does not enforce Unix"; \
		echo "       permission bits for it (root or CAP_DAC_OVERRIDE, the normal case inside a"; \
		echo "       container). The permission fixtures would fail, one panic per test."; \
		echo "       Run the tests on a host that enforces mode bits, or declare this one unable"; \
		echo "       to with:"; \
		echo "         FDU_TEST_ALLOW_NO_PERMISSION_BITS=1 make $(if $(MAKECMDGOALS),$(MAKECMDGOALS),check)"; \
		echo "       CI leaves the variable unset so a passing test proves its assertions ran."; \
		exit 1; \
	fi; \
	rm -f "$$probe"

# Every target that runs the crates' tests, so the preflight fires before the first one.
PERMISSION_FIXTURE_TARGETS := rust-test lib-only msrv

$(PERMISSION_FIXTURE_TARGETS): permission-bits

reference-model:
	$(CARGO) test --locked -p fdu-core --test reference_model --no-default-features

opened-root-golden:
	$(CARGO) test --locked -p fdu-core --all-features \
		opened::golden_tests::opened_root_session_goldens -- --exact
	$(MAKE) opened-root-golden-lint

opened-root-golden-lint:
	$(NODE) --test scripts/check-opened-root-goldens.test.mjs
	$(NODE) scripts/check-opened-root-goldens.mjs

opened-root-golden-update:
	@test -n "$(SCENARIO)" || { \
		echo "error: SCENARIO is required; refusing to update the full opened-root corpus"; \
		exit 2; \
	}
	$(NODE) scripts/check-opened-root-goldens.mjs --scenario "$(SCENARIO)"
	FDU_UPDATE_OPENED_ROOT_GOLDEN="$(SCENARIO)" $(CARGO) test --locked -p fdu-core \
		--all-features opened::golden_tests::opened_root_session_goldens -- --exact
	$(MAKE) opened-root-golden

test-golden: build $(NODE_INSTALL_STAMP)
	$(NPM) run test:golden

yaml-selfcheck: build $(NODE_INSTALL_STAMP)
	$(CARGO) build --locked -p fdu-core --example format_conformance --features watch
	node scripts/check-yaml.mjs

content-selfcheck: build
	$(NODE) scripts/content-selfcheck.mjs

performance-probe:
	$(CARGO) test --locked -p fdu-core --example perf_probe --no-default-features
	$(CARGO) build --locked -p fdu-core --example perf_probe --no-default-features

# The first suite runs without a project, so nothing else names its interpreter: uv would
# take the host's python3, and on 3.11 the suite fails for want of 3.12 (fdu-kiuu).
test-performance: performance-probe
	CARGO_TARGET_DIR="$(CARGO_TARGET)" PYTHONPATH=explorations $(UV) run --no-project --python 3.12 python -m unittest discover -s explorations/benchmarks/tests -p 'test_*.py'
	$(PERF_UV) --group dev python -m unittest discover -s explorations/benchmarks/realtree/tests -p 'test_*.py'

# Tryscript returns nonzero when it updates a previously failing block. The immediate
# comparison is authoritative and catches execution failures or incomplete updates.
golden-update: build $(NODE_INSTALL_STAMP)
	-$(NPM) run test:golden:update
	$(NPM) run test:golden

$(NODE_INSTALL_STAMP): package.json package-lock.json .npmrc
	$(NPM) ci

# Everything CI enforces, in the order that fails fastest.
check: uv-version wheel-python supply-chain rust-module-names admission-sites atomic-writes golden-invocations golden-observability opened-root-golden-lint portability fmt-check clippy test docs docs-format-check perf-test perf-schema-check perf-evidence-check perf-ledger-check perf-report-check lib-only msrv audit npm-audit python-check python-concurrency python-smoke python-sdist-smoke parity-check test-path-independence path-independence release-test test-terminal

# The uv.toml files express the supply-chain cool-off as a relative `exclude-newer`
# ("14 days"). uv releases older than this cannot parse that form: they abort with
# `failed to parse year in date "14 days"`, which reads like a corrupt config rather
# than a stale tool, and it takes out every uv-backed target (docs formatting, the
# performance harness, the Python jobs) at once. Fail early and say so instead.
#
# CI installs uv through astral-sh/setup-uv in .github/workflows/ci.yml. The
# supply-chain policy verifies that this floor and both CI pins remain identical.
UV_MIN_VERSION := 0.12.1

uv-version:
	@command -v "$(UV)" >/dev/null 2>&1 || { \
		echo "error: uv is not installed, and this repository needs uv >= $(UV_MIN_VERSION)."; \
		echo "       Install the reviewed $(UV_MIN_VERSION) release using the official instructions:"; \
		echo "       https://docs.astral.sh/uv/getting-started/installation/"; \
		exit 1; }
	@version_output=$$("$(UV)" --version 2>/dev/null) || { \
		echo "error: could not run uv --version; reinstall the reviewed $(UV_MIN_VERSION) release."; \
		exit 1; \
	}; \
	have=$$(printf '%s\n' "$$version_output" | awk 'NF >= 2 && $$1 == "uv" { print $$2; exit }'); \
	relation=$$(awk -v have="$$have" -v need="$(UV_MIN_VERSION)" 'BEGIN { \
		if (have !~ /^[0-9]+\.[0-9]+\.[0-9]+$$/ || need !~ /^[0-9]+\.[0-9]+\.[0-9]+$$/) { print "invalid"; exit; } \
		split(have, actual, "."); split(need, minimum, "."); \
		for (i = 1; i <= 3; i++) { \
			if (actual[i] + 0 < minimum[i] + 0) { print "old"; exit; } \
			if (actual[i] + 0 > minimum[i] + 0) { print "ok"; exit; } \
		} \
		print "ok"; \
	}'); \
	if [ "$$relation" = "old" ]; then \
		echo "error: uv $$have is too old; this repository needs uv >= $(UV_MIN_VERSION)"; \
		echo "       (the version CI pins in .github/workflows/ci.yml)."; \
		echo "       Older releases cannot parse the relative 'exclude-newer' in the uv.toml"; \
		echo "       files and fail with a misleading TOML date error."; \
		echo "       Upgrade to the reviewed release with:"; \
		echo "         curl -LsSf https://astral.sh/uv/$(UV_MIN_VERSION)/install.sh | sh"; \
		echo "       'uv self update $(UV_MIN_VERSION)' also works, but only when uv owns its own"; \
		echo "       install; it fails with 'not found for the app uv' under an external manager."; \
		exit 1; \
	elif [ "$$relation" != "ok" ]; then \
		echo "error: could not determine a stable uv version from: $$version_output"; \
		echo "       Reinstall the reviewed $(UV_MIN_VERSION) release before continuing."; \
		exit 1; \
	fi

# Standalone entry points must fail before any recipe asks uv to parse repository
# configuration. Keep this list aligned with the recipe-coverage test.
UV_BACKED_TARGETS := test-performance test-path-independence path-independence path-independence-full path-independence-record python-check python-concurrency python-smoke python-sdist-smoke release-test test-terminal release-rehearse semver-check docs-format docs-format-check \
	perf-baseline perf-profile perf-content-profile perf-compare perf-content-compare \
	perf-compare-tools perf-floor perf-record perf-store perf-subjects perf-subjects-check perf-test perf-ledger perf-ledger-check perf-report perf-report-check perf-schema perf-schema-check perf-evidence-check

$(UV_BACKED_TARGETS): uv-version

# Verify that synced beads match the local database, field by field.
#
# Deliberately outside `check`: it compares against `origin/tbd-sync`, a branch other
# working copies push to independently, so a shared-branch race would fail a PR for
# something the PR did not do. Run it before a handoff, or when a sync looked odd.
verify-beads:
	git fetch --quiet origin tbd-sync
	python3 scripts/verify_bead_sync.py --quiet

supply-chain:
	$(NPM) run test:supply-chain
	$(NPM) run check:supply-chain

rust-module-names:
	$(NODE) --test scripts/check-rust-module-names.test.mjs
	$(NODE) scripts/check-rust-module-names.mjs

admission-sites:
	$(NODE) --test scripts/check-admission-sites.test.mjs
	$(NODE) scripts/check-admission-sites.mjs

# A reader sees a whole old file or a whole new one: every write goes through a helper
# that stages, syncs, and renames, or is listed in the check with its reason.
atomic-writes:
	$(NODE) --test scripts/check-atomic-writes.test.mjs scripts/atomic-write.test.mjs
	$(NODE) scripts/check-atomic-writes.mjs

# The corpus selects its binary by full path. This keeps a bare `fdu` -- which PATH
# would happily resolve to an installed build -- from creeping back in (fdu-9h2w).
golden-invocations:
	$(NODE) scripts/check-golden-invocations.mjs

golden-observability:
	$(NODE) --test scripts/check-golden-observability.test.mjs
	$(NODE) scripts/check-golden-observability.mjs

# Committed test data must not name the machine that recorded it. `tryscript run --update`
# writes what it saw, so it expands named patterns into literals -- which passes forever
# on the recording machine and nowhere else.
portability:
	$(NODE) scripts/check-portability.mjs

# The interpreter every environment that installs fdu is created with. The wheel is
# cp312-abi3, which a free-threaded CPython cannot install, and uv picks a free-threaded
# build when it manages one, so an unpinned `uv venv` fails the gate on such a host
# (fdu-pd1b). The default is the version CI's Python quality job pins; UV_PYTHON, or
# WHEEL_PYTHON on the command line, chooses another GIL-enabled CPython.
WHEEL_PYTHON ?= $(or $(UV_PYTHON),3.12)

# An explicit free-threaded request would reach `uv venv` and fail on wheel tags, a
# message that names neither the request nor the remedy. uv reads a version followed by
# `t`, or `td` for the debug build, as free-threaded in every request form (`3.14t`,
# `cpython@3.14t`, `cpython-3.14t-macos-aarch64-none`), and so is `+freethreaded`.
# Splitting on `-` separates the version from the rest of the full form. The check is a
# Make function in the recipe, so `make -n` refuses it too.
WHEEL_PYTHON_FREE_THREADED_SUFFIXES := $(foreach digit,0 1 2 3 4 5 6 7 8 9,%$(digit)t %$(digit)td)
WHEEL_PYTHON_FREE_THREADED = $(filter $(WHEEL_PYTHON_FREE_THREADED_SUFFIXES),$(subst -, ,$(WHEEL_PYTHON)))$(findstring freethreaded,$(WHEEL_PYTHON))
WHEEL_PYTHON_REFUSAL = WHEEL_PYTHON=$(WHEEL_PYTHON) is a free-threaded CPython, which cannot install the cp312-abi3 wheel; set UV_PYTHON or WHEEL_PYTHON to a GIL-enabled CPython such as 3.12

wheel-python:
	$(if $(WHEEL_PYTHON_FREE_THREADED),$(error $(WHEEL_PYTHON_REFUSAL)),@:)

parity-venv python-smoke python-sdist-smoke: wheel-python

# The parity surface needs the wheel installed, not the working tree: a shim importing
# python/fdu/ directly would pass while the built package was broken, which is the
# failure public_smoke already exists to prevent.
parity-venv: uv-version
	cd crates/fdu-py && wheel_dir="$$(mktemp -d "$${TMPDIR:-/tmp}/fdu-parity.XXXXXX")" && \
		trap 'rm -r -- "$$wheel_dir"' EXIT && \
		$(UV) run --frozen --only-group dev maturin build --locked --release --out "$$wheel_dir" && \
		$(UV) venv --clear --python $(WHEEL_PYTHON) .venv-parity && \
		$(UV) pip install --python .venv-parity --no-index --find-links "$$wheel_dir" fdu

# The two interpreters the parity harness can run against, named once. `parity-venv`
# builds the first; `python-smoke` installs the wheel into the second, which is why the
# gate reuses it rather than paying for a third build.
PARITY_PYTHON := crates/fdu-py/.venv-parity/bin/python
SMOKE_PYTHON := crates/fdu-py/.venv-smoke/bin/python

# Replay the golden corpus against the Python surface. The committed deviation file is
# non-empty by construction, so an empty result means the shim never ran (fdu-9h2w).
test-parity: build parity-venv $(NODE_INSTALL_STAMP)
	$(NPM) run test:parity-classes
	FDU_PARITY_PYTHON=$(PARITY_PYTHON) $(NODE) scripts/run-parity.mjs

# Used by the gate, where python-smoke has already installed the wheel into
# .venv-smoke; standalone runs want test-parity, which builds its own.
parity-check: build $(NODE_INSTALL_STAMP)
	$(NPM) run test:parity-classes
	FDU_PARITY_PYTHON=$(SMOKE_PYTHON) $(NODE) scripts/run-parity.mjs

parity-update: build parity-venv $(NODE_INSTALL_STAMP)
	FDU_PARITY_PYTHON=$(PARITY_PYTHON) $(NODE) scripts/run-parity.mjs --update

# The path-independence harness (tests/path_independence): an answer must not depend on
# cache history, cache policy, file changes since warming, or which surface asked. The
# unit tests need no build. The matrix runs the debug binary and, for the Python routes,
# an installed wheel -- the gate's .venv-smoke, or .venv-parity from `make parity-venv`
# when run on its own. The gate requires an empty known-violations.toml;
# `path-independence-record` records diagnostic evidence and cannot waive a failure.
PATH_INDEPENDENCE_PYTHON ?= $(PARITY_PYTHON)
check: PATH_INDEPENDENCE_PYTHON = $(SMOKE_PYTHON)
PATH_INDEPENDENCE_ENV = FDU_BIN="$(DEBUG_FDU)" \
	FDU_PYTHON="$(abspath $(PATH_INDEPENDENCE_PYTHON))"
PATH_INDEPENDENCE_PYTHON_REQUIRED = @test -x "$(PATH_INDEPENDENCE_PYTHON)" || \
	{ echo "error: $(PATH_INDEPENDENCE_PYTHON) is missing; build it with 'make parity-venv' (or 'make python-smoke' for .venv-smoke)"; exit 1; }

test-path-independence:
	$(UV) run --no-project --python 3.12 python -m unittest discover -s tests/path_independence -p 'test_harness.py'

path-independence: build
	$(PATH_INDEPENDENCE_PYTHON_REQUIRED)
	$(PATH_INDEPENDENCE_ENV) FDU_PI_TIER=subset \
		$(UV) run --no-project --python 3.12 python -m unittest discover -s tests/path_independence -p 'test_path_independence.py'

path-independence-full: build
	$(PATH_INDEPENDENCE_PYTHON_REQUIRED)
	$(PATH_INDEPENDENCE_ENV) FDU_PI_TIER=full \
		$(UV) run --no-project --python 3.12 python -m unittest discover -s tests/path_independence -p 'test_path_independence.py'

path-independence-record: build
	$(PATH_INDEPENDENCE_PYTHON_REQUIRED)
	$(PATH_INDEPENDENCE_ENV) $(UV) run --no-project --python 3.12 python tests/path_independence/runner.py --tier full --record

fmt:
	$(CARGO) fmt --all

fmt-check:
	$(CARGO) fmt --all --check

clippy:
	$(CARGO) clippy --locked --all-targets --all-features -- -D warnings

# Lint the code the host platform's build never sees.
#
# `cfg(target_os = ...)` code is invisible to a single-platform clippy run, and this
# repository keeps its unsafe exceptions behind exactly such gates — the macOS
# `getattrlistbulk` reader among them. CI lints on ubuntu only, so before this target that
# module had never been linted anywhere. Three separate platform-gated defects reached CI
# in one session for want of it.
#
# The Linux `getdents64` reader is gated on glibc as well as Linux, so a 64-bit glibc host
# sees only one shape of it: i686 glibc checks it at 32-bit widths (`c_long`, `time_t`,
# the `statx` size assertion), x86_64 musl checks that the gate leaves the portable
# path compiling cleanly without it, and aarch64 glibc checks the shape the arm64 wheel
# ships: its own syscall numbers and open flags, and an unsigned `c_char`. CI's arm64 job
# also runs that shape's tests.
#
# Checking, not building: no linker for the other platforms is needed, so this runs
# anywhere. Add the targets once with
#   rustup target add x86_64-apple-darwin x86_64-pc-windows-msvc \
#     i686-unknown-linux-gnu x86_64-unknown-linux-musl aarch64-unknown-linux-gnu
# and this target skips any that are missing rather than failing, so it stays usable on
# a machine that has not installed them.
CROSS_TARGETS := x86_64-apple-darwin x86_64-pc-windows-msvc i686-unknown-linux-gnu \
	x86_64-unknown-linux-musl aarch64-unknown-linux-gnu

cross-lint:
	@installed="$$(rustup target list --installed 2>/dev/null)"; \
	for target in $(CROSS_TARGETS); do \
		if echo "$$installed" | grep -qx "$$target"; then \
			echo "== clippy: $$target"; \
			$(CARGO) clippy --locked --all-targets --target "$$target" -- -D warnings || exit 1; \
		else \
			echo "== skipping $$target (rustup target add $$target)"; \
		fi; \
	done

docs:
	RUSTDOCFLAGS="-D warnings" $(CARGO) doc --locked --no-deps --all-features

# How library consumers build: the minimal core, then the additive watch layer, without
# relying on what the binary enables. Explicit `--no-default-features` pins the empty
# build-feature floor as a contract. `.gitignore` handling is not a build feature and is
# in both shapes; whether a scan reads control files is a runtime setting, which the
# tests cover.
# The dependency guard proves the crate split stuck -- a library that pulls in an
# argument parser has back the dependency the split removed.
#
# The guard captures `cargo tree` before testing it, rather than piping straight into
# grep. A pipeline's status is its last command's, so a failing `cargo tree` -- renamed
# package, manifest error, resolver failure -- would hand grep empty input, grep would
# return 1, `!` would invert it to 0, and the check that proves the split would report
# success having checked nothing (fdu-cqtk).
#
# The command line makes the same promise from the other side: crates/fdu/Cargo.toml
# says it builds without `watch`, which is what keeps that layer deletable. Nothing
# compiled the featureless command line, so it quietly stopped building (fdu-2wlp). A
# clippy run over every target holds the promise and lints the shape too, since the
# workspace's pedantic lints are clippy's and the Clippy job lints only all features
# (fdu-kaog); it stays cheap because nothing links. The library's own tests then run in
# that shape, because compiling a test is not running it: the guide named `--watch` to a
# binary without it while every featureless build passed (fdu-224p).
lib-only:
	$(CARGO) test --locked -p fdu-core --no-default-features
	$(CARGO) test --locked -p fdu-core --no-default-features --features watch
	$(CARGO) clippy --locked -p fdu --no-default-features --all-targets -- -D warnings
	$(CARGO) test --locked -p fdu --no-default-features --lib
	@tree="$$($(CARGO) tree -p fdu-core --all-features --prefix none)" || exit 1; \
		! printf '%s\n' "$$tree" | grep -qE '^(clap|anyhow) ' \
		|| { echo 'fdu-core must not depend on clap or anyhow; they belong to fdu'; exit 1; }

# The Windows target is checked on the MSRV as well, because platform-gated code is
# invisible to a check on the host target and CI's MSRV job runs on ubuntu; the job
# installs the target, and locally this skips it when the MSRV toolchain lacks it, as
# cross-lint does, rather than failing a machine that has not added it.
msrv:
	$(CARGO) +$(MSRV) check --locked --all-features
	$(CARGO) +$(MSRV) test --locked -p fdu-core --no-default-features
	@if rustup +$(MSRV) target list --installed 2>/dev/null | grep -qx x86_64-pc-windows-msvc; then \
		echo "== msrv check: x86_64-pc-windows-msvc"; \
		$(CARGO) +$(MSRV) check --locked --all-features --all-targets --target x86_64-pc-windows-msvc || exit 1; \
	else \
		echo "== skipping x86_64-pc-windows-msvc on $(MSRV) (rustup +$(MSRV) target add x86_64-pc-windows-msvc)"; \
	fi

fix:
	$(CARGO) fmt --all
	$(CARGO) clippy --locked --all-targets --all-features --fix --allow-dirty --allow-staged

audit:
	$(CARGO) deny --locked check

npm-audit: $(NODE_INSTALL_STAMP)
	$(NPM) audit --audit-level=moderate

python-concurrency:
	$(UV) run --directory crates/fdu-py --frozen --only-group dev \
		python tests/run_concurrency.py

# The explicit --config keeps one lint standard for the package, its examples, and the
# repository-level release scripts and tests, which have no pyproject of their own.
PYTHON_LINT_PATHS := python tests examples ../../scripts/atomic_write.py ../../scripts/release ../../scripts/run_installed_cli_qa.py ../../scripts/qa_peer_agreement.py ../../tests/release ../../tests/parity ../../tests/path_independence ../../tests/correctness ../../tests/terminal ../../explorations/benchmarks/realtree/validate.py ../../explorations/benchmarks/realtree/tests/test_validate.py

# pytest imports the editable install, whose compiled half uv rebuilds only when a cache
# key changes -- by default Python metadata files, never the Rust. A reused .venv then
# ran current wrappers against an extension from before the last native change
# (fdu-35b1, fdu-ukg6). Cache keys cannot express it here: uv reads them only from
# pyproject.toml, and warns on every run that the adjacent uv.toml overrides them. So the
# test run always rebuilds the editable extension; cargo decides what is actually stale,
# and target-owner keeps that decision honest in a shared target directory.
python-check:
	$(UV) run --directory crates/fdu-py --frozen --only-group dev \
		ruff format --check --config pyproject.toml $(PYTHON_LINT_PATHS)
	$(UV) run --directory crates/fdu-py --frozen --only-group dev \
		ruff check --config pyproject.toml $(PYTHON_LINT_PATHS)
	$(UV) run --directory crates/fdu-py --frozen --only-group dev basedpyright
	$(UV) run --directory crates/fdu-py --frozen --group dev --reinstall-package fdu pytest

python-smoke:
	cd crates/fdu-py && wheel_dir="$$(mktemp -d "$${TMPDIR:-/tmp}/fdu-wheel.XXXXXX")" && \
		type_dir="$$(mktemp -d "$${TMPDIR:-/tmp}/fdu-typecheck.XXXXXX")" && \
		trap 'rm -r -- "$$wheel_dir" "$$type_dir"' EXIT && \
		$(UV) run --frozen --only-group dev maturin build --locked --release --out "$$wheel_dir" && \
		$(UV) venv --clear --python $(WHEEL_PYTHON) .venv-smoke && \
		$(UV) pip install --python .venv-smoke --no-index --find-links "$$wheel_dir" fdu && \
		$(UV) run --no-project --python .venv-smoke python tests/public_smoke.py && \
		$(UV) run --no-project --python .venv-smoke python tests/smoke.py && \
		cp tests/typecheck/consumer.py tests/typecheck/pyrightconfig.json "$$type_dir/" && \
		$(UV) run --frozen --only-group dev basedpyright --pythonpath .venv-smoke/bin/python \
			--project "$$type_dir/pyrightconfig.json" && \
		wheel_path="$$(find "$$wheel_dir" -maxdepth 1 -type f -name '*.whl' -print -quit)" && \
		$(UV) tool run --isolated --no-index --python $(WHEEL_PYTHON) --from "$$wheel_path" fdu --version

# The sdist must build from its own contents. Under an exported CARGO_TARGET_DIR -- which
# AGENTS.md recommends -- cargo would build it into the checkout's target directory, where
# the same crates' newer outputs can pass for fresh and install instead (fdu-8whh).
python-sdist-smoke:
	cd crates/fdu-py && sdist_dir="$$(mktemp -d "$${TMPDIR:-/tmp}/fdu-sdist.XXXXXX")" && \
		trap 'rm -r -- "$$sdist_dir"' EXIT && \
		$(UV) build --no-sources --sdist --out-dir "$$sdist_dir" && \
		$(UV) venv --clear --python $(WHEEL_PYTHON) .venv-sdist && \
		env -u CARGO_TARGET_DIR $(UV) pip install --python .venv-sdist "$$sdist_dir/fdu-"*.tar.gz && \
		$(UV) run --no-project --python .venv-sdist python tests/public_smoke.py

# uv provides the interpreter so the release gates never depend on the host's system
# python3, whose version nothing else checks (the scripts need tomllib, so 3.11+).
release-test:
	$(UV) run --no-project --python 3.12 python -m unittest discover -s tests/release -p 'test_*.py'

# The progress indicator in a real pseudo-terminal: drawn, erased before the report,
# cleared on Ctrl-C with death by the signal, and absent when stderr is not a terminal.
# Unix only; Python has no pty on Windows, so the test skips itself there.
test-terminal: build
	FDU_BIN="$${FDU_BIN:-$(DEBUG_FDU)}" \
		$(UV) run --no-project --python 3.12 python -m unittest discover -s tests/terminal -p 'test_*.py'

# Build and inspect the host artifacts without contacting either registry. The explicit
# release tag exercises exact-version behavior even though a rehearsal runs on a branch.
#
# One `cargo package` naming both crates, not two invocations: `fdu` depends on `fdu-core`,
# which is not on crates.io, so packaging `fdu` alone fails to resolve it. Packaging the
# sibling first in a separate run does not help -- that puts a `.crate` in <target>/package,
# not in the index. Naming both in one invocation makes cargo verify each against the
# just-packaged sibling (fdu-pj9w).
#
# The crate smoke is the release workflow's own step, run on the copied artifacts: it
# installs the packaged `fdu`, locked, against the packaged `fdu-core` (fdu-y5zc).
release-rehearse: release-test
	artifact_dir="$$(mktemp -d "$${TMPDIR:-/tmp}/fdu-release.XXXXXX")" && \
		smoke_dir="$$(mktemp -d "$${TMPDIR:-/tmp}/fdu-crate-smoke.XXXXXX")" && \
		trap 'rm -r -- "$$artifact_dir" "$$smoke_dir"' EXIT && \
		version="$$($(UV) run --no-project --python 3.12 python -c 'import pathlib,tomllib; print(tomllib.loads(pathlib.Path("crates/fdu/Cargo.toml").read_text())["package"]["version"])')" && \
		export FDU_RELEASE_TAG="v$$version" && \
		$(CARGO) package --locked -p fdu-core -p fdu --allow-dirty && \
		cp "$(CARGO_TARGET)/package/fdu-core-$$version.crate" "$(CARGO_TARGET)/package/fdu-$$version.crate" "$$artifact_dir/" && \
		$(UV) run --no-project --python 3.12 python scripts/release/smoke_crate.py "$$artifact_dir" --version "$$version" \
			--work-dir "$$smoke_dir" --cargo "$(CARGO)" && \
		$(UV) build --directory crates/fdu-py --no-sources --sdist --out-dir "$$artifact_dir" && \
		$(UV) run --directory crates/fdu-py --frozen --only-group dev maturin build --locked --release --out "$$artifact_dir" && \
		$(UV) run --no-project --python 3.12 python scripts/release/inspect_artifacts.py "$$artifact_dir" --version "$$version" \
			--manifest "$$artifact_dir/manifest.json" --checksums "$$artifact_dir/SHA256SUMS"

# Compare fdu-core and fdu with the published release they must stay compatible with, as
# the release workflow's semver job does: a patch release never breaks the Rust API.
# Before the version bump it asks whether this tree could still ship as a patch. It
# reads crates.io, so it is outside `check`, and it needs the reviewed cargo-semver-checks,
# whose install command it prints when that is missing or another version.
semver-check:
	$(UV) run --no-project --python 3.12 python scripts/release/semver_check.py $(ARGS)

# The maintainer's release checklist, one step per target in order, then the recovery
# audit; see docs/project/guides/release-process.md. Each reads VERSION, COMMIT, RELEASE,
# and SIGNING_KEY from the environment, and ARGS passes a step's own options. None of
# them tags, dispatches a publishing run, approves, or announces: those stay the
# maintainer's.
release-preflight release-candidate release-body release-verify-tag release-published release-announced release-cleanup release-audit: uv-version
	$(UV) run --no-project --python 3.12 python scripts/release/maintainer.py $(patsubst release-%,%,$@) $(ARGS)

cli:
	$(CARGO) run --locked --release --bin fdu -- --cache off -d 2 .

# --- Documentation ----------------------------------------------------------
#
# `--auto` owns repository-wide file discovery and applicable cleanups. The committed
# tooling lock pins the native Rust formatter used locally and in CI. Generated Markdown
# uses this same path after generation, so regenerating it cannot create format drift.
FLOWMARK := $(UV) run --project explorations/benchmarks --frozen --only-group docs flowmark

docs-format:
	@$(FLOWMARK) --auto .

# Fails when a document is not in normal form, so drift is caught rather than
# accumulating until someone reformats a file and buries a real change in noise.
docs-format-check:
	@$(FLOWMARK) --auto --check .

# --- Performance loop -------------------------------------------------------
#
# Deliberately outside `check`. This is a development workflow that needs a large
# real tree and a quiet machine, neither of which CI has; a timing gate on a shared
# runner measures the runner. See docs/project/guides/performance-loop.md.
#
# PERF_TREE names the reference tree. Freeze all writers for the whole run; the
# harness rejects any difference between its immediate pre/post fingerprints.

PERF_TREE ?= explorations/benchmarks
PERF_LABEL ?= benchmarks-self-contained
PERF_RESULTS ?= /tmp/fdu-realtree/results
PERF_SCRATCH ?= /tmp/fdu-realtree/scratch
PERF_BASELINE ?= $(PERF_RESULTS)/tree-$(PERF_LABEL).json
# The probe is measured where cargo wrote it: see CARGO_TARGET.
PERF_TARGET_DIR = $(CARGO_TARGET)
PERF_RELEASE = $(PERF_TARGET_DIR)/release/examples/perf_probe
PERF_PROFILING = $(PERF_TARGET_DIR)/profiling/examples/perf_probe
# Evidence qualifiers default to exploration. A held-out run must opt into a controlled
# host regime and provide the manifests that make its source, corpus, and installation
# independently verifiable.
PERF_STAGE ?= exploratory
PERF_HOST_REGIME ?= uncontrolled
PERF_BACKGROUND_LOAD_WORKERS ?=
PERF_PROVENANCE ?=
PERF_CORPUS_MANIFEST ?=
PERF_INSTALLATION_ATTESTATION ?=
PERF_TOOL_SUPPORTING_ARGS ?=
PERF_EVIDENCE_ARGS = --stage $(PERF_STAGE) --host-regime $(PERF_HOST_REGIME) \
	$(if $(strip $(PERF_BACKGROUND_LOAD_WORKERS)),--background-load-workers $(PERF_BACKGROUND_LOAD_WORKERS)) \
	$(if $(strip $(PERF_PROVENANCE)),--provenance-manifest "$(PERF_PROVENANCE)")
PERF_MEASURE_EVIDENCE_ARGS = $(PERF_EVIDENCE_ARGS) \
	$(if $(strip $(PERF_CORPUS_MANIFEST)),--corpus-manifest "$(PERF_CORPUS_MANIFEST)")
PERF_TOOL_EVIDENCE_ARGS = $(PERF_EVIDENCE_ARGS) \
	$(if $(strip $(PERF_INSTALLATION_ATTESTATION)),--installation-attestation "$(PERF_INSTALLATION_ATTESTATION)") \
	$(PERF_TOOL_SUPPORTING_ARGS)
# The harness runs from the repo root against a committed, frozen environment, so a
# benchmark run resolves nothing at invocation time. `--project` (not `--directory`)
# keeps the working directory here, and `PYTHONPATH` puts the harness's parent on the
# import path; together they are what make `-m benchmarks.realtree` work. The package is
# still `benchmarks` -- only the directory holding it moved under `explorations/`.
PERF_UV := PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=explorations $(UV) run --project explorations/benchmarks --frozen
PERF_RUN := $(PERF_UV) python -m benchmarks.realtree

.PHONY: perf-floor perf-probe-release perf-probe-profiling perf-baseline perf-profile perf-compare perf-content-profile perf-content-compare perf-compare-tools perf-record perf-store perf-subjects perf-subjects-check perf-test perf-ledger perf-ledger-check perf-report perf-report-check perf-schema perf-schema-check perf-evidence-check

perf-probe-release:
	$(CARGO) build --locked --release -p fdu-core --example perf_probe --no-default-features

perf-probe-profiling:
	$(CARGO) build --locked --profile profiling -p fdu-core --example perf_probe --no-default-features

# Record what the tree looks like now, so later runs can prove they measured the same one.
perf-baseline:
	$(PERF_RUN) baseline --root $(PERF_TREE) --label $(PERF_LABEL) \
		--output $(PERF_BASELINE)

# Where does the time go? Attribution only; never a timing claim.
perf-profile: perf-probe-profiling
	$(PERF_RUN) profile --root $(PERF_TREE) --binary $(PERF_PROFILING) \
		--job cold-scan-index --job warm-revalidate --label $(or $(NAME),latest) \
		--scratch $(PERF_SCRATCH) \
		--output $(PERF_RESULTS)/profile-$(or $(NAME),latest).json

perf-content-profile: perf-probe-profiling
	$(PERF_RUN) profile --root $(PERF_TREE) --binary $(PERF_PROFILING) \
		--job content-basic --job content-cache-hit --job code-sloc \
		--job code-sloc-cache-hit --job text-prose --job markdown-prose \
		--job document-cache-hit --job content-query \
		--label $(or $(NAME),content-latest)

# Is the candidate faster than the control? Set CONTROL to a saved reference binary.
CONTROL ?= $(PERF_RELEASE)
# JOBS selects the jobs for one round; the default is the metadata set. An experiment
# whose hypothesis names one tier should measure that tier's job and the jobs its
# mechanism could move, not all six: every extra job is wall time the subject spends
# drifting between the pairs that matter.
PERF_DEFAULT_JOBS := aggregate-summary cold-scan-index cold-scan-producer \
	cold-snapshot-save warm-revalidate warm-snapshot-load
PERF_JOB_ARGS = $(foreach job,$(or $(JOBS),$(PERF_DEFAULT_JOBS)),--job $(job))

perf-compare: perf-probe-release
	$(PERF_RUN) measure --root $(PERF_TREE) --label $(PERF_LABEL) \
		--variant "control=$(CONTROL)" \
		--variant "candidate=$(PERF_RELEASE)" \
		--reference dust=$(shell command -v dust 2>/dev/null || echo /usr/bin/du) \
		$(PERF_JOB_ARGS) \
		--trials $(or $(TRIALS),12) \
		--scratch $(PERF_SCRATCH) --output-dir $(PERF_RESULTS) \
		--baseline-fingerprint $(PERF_BASELINE) \
		--name $(or $(NAME),adhoc) $(PERF_MEASURE_EVIDENCE_ARGS)

# JOBS selects the jobs for one content round, exactly as it does for `perf-compare`;
# the default is the whole content set.
PERF_CONTENT_DEFAULT_JOBS := content-basic content-cache-hit code-sloc \
	code-sloc-cache-hit text-prose markdown-prose document-cache-hit content-query
PERF_CONTENT_JOB_ARGS = $(foreach job,$(or $(JOBS),$(PERF_CONTENT_DEFAULT_JOBS)),--job $(job))

perf-content-compare: perf-probe-release
	$(PERF_RUN) measure --root $(PERF_TREE) --label $(PERF_LABEL) \
		--variant "control=$(CONTROL)" \
		--variant "candidate=$(PERF_RELEASE)" \
		$(PERF_CONTENT_JOB_ARGS) \
		--trials $(or $(TRIALS),12) \
		--scratch $(PERF_SCRATCH) --output-dir $(PERF_RESULTS) \
		--baseline-fingerprint $(PERF_BASELINE) \
		--name $(or $(NAME),content-adhoc) $(PERF_MEASURE_EVIDENCE_ARGS)

# Compare one immutable fdu release binary with external tools on the same live tree.
# TOOL_ARGS supplies repeated `--tool name=/path/to/binary` arguments. Results must
# stay outside PERF_TREE so the evidence write cannot invalidate its own subject.
PERF_TOOL_RESULTS ?= /tmp/fdu-tool-comparison/results
PERF_TOOL_BASELINE ?= $(PERF_TOOL_RESULTS)/tree-$(PERF_LABEL).json
PERF_TOOL_CONTROL ?=
PERF_TOOL_LABEL ?= fdu
PERF_TOOL_CONTRACT ?= fdu-transient-summary
perf-compare-tools:
	@test -n "$(PERF_TOOL_CONTROL)" || \
		{ echo "PERF_TOOL_CONTROL must name an immutable fdu CLI binary outside PERF_TREE" >&2; exit 2; }
	$(PERF_RUN).compare_tools --root $(PERF_TREE) --label $(PERF_LABEL) \
		--anchor "$(PERF_TOOL_LABEL):$(PERF_TOOL_CONTRACT)=$(PERF_TOOL_CONTROL)" $(TOOL_ARGS) \
		--trials $(or $(TRIALS),12) --warmups $(or $(WARMUPS),3) \
		--baseline-output $(PERF_TOOL_BASELINE) \
		--output-dir $(PERF_TOOL_RESULTS) --name $(or $(NAME),tool-comparison) \
		--storage "$(or $(STORAGE),local storage)" $(PERF_TOOL_EVIDENCE_ARGS)

# Record an experiment artifact from a completed measurement run.
# This host's nominated real-tree subject set.
#
# Campaign 2 requires at least one nominated real tree behind any accept decision, and
# nothing could express that before: a run names one `--root` and the record had no
# notion of which trees are fit to decide. The nominations file is local and gitignored
# because it holds absolute paths, which the loop never records; the document written
# here is redacted and committable.
#
# Not in `check`: it walks trees only this machine has.
#
# One document per host class, because `root_id` hashes an absolute path and a set is a
# fact about one machine: a Linux host running this must not overwrite the macOS set.
PERF_HOST_CLASS := $(shell uname -s | tr '[:upper:]' '[:lower:]')-$(shell uname -m)
PERF_SUBJECTS ?= docs/project/reports/nominated-subjects-$(PERF_HOST_CLASS).json

perf-subjects:
	$(PERF_RUN) subjects --out $(PERF_SUBJECTS)

# Re-observe the nominated trees and report what moved. A nominated tree is somebody's
# live working directory, so drift is expected -- what matters is that a reader is told
# before they compare last month's number with today's.
perf-subjects-check:
	$(PERF_RUN) subjects --check $(PERF_SUBJECTS)

# The tier-by-subject floor scoreboard: where each tier sits against what the machine
# charges for the work fdu cannot avoid. Campaign 2 orders work by that distance, so this
# is the scoreboard every accepted change re-runs, and what makes its termination
# criteria checkable rather than asserted.
#
# Not in `check`, and not a verdict harness: `perf-compare` decides whether a change is
# kept, under the paired accept rule. This divides two absolute numbers to say where a
# tier stands, which is a different question and needs the host to itself just as much.
#
# Linux only, and it refuses rather than falling back: `parfloor.c` is the denominator
# every x-floor threshold is defined against and it issues SYS_getdents64 and statx
# directly. A macOS scoreboard needs a getattrlistbulk floor (fdu-9hdc) or a different
# floor set with the regime difference recorded -- a decision for the campaign plan
# rather than one a harness makes by substituting a denominator and printing the same
# column heading. See fdu-33ri.
#
# SUBJECTS takes repeated LABEL=PATH pairs; a subject decides only if it is dense and at
# least 50,000 entries, and smaller ones screen.
#
# The probe is the one `perf-probe-release` builds, handed over by path, so a scoreboard
# and a verdict run always score the same binary.
#
# The regime defaults to quiet. PERF_HOST_REGIME overrides it only when set on the command
# line or in the environment: its file default above is `uncontrolled`, which would
# otherwise always win.
PERF_FLOOR_OUT ?= /tmp/fdu-floor
PERF_FLOOR_SUBJECT_ARGS = $(foreach subject,$(SUBJECTS),--subject $(subject))
PERF_FLOOR_HOST_REGIME = $(if $(filter command line environment environment override,$(origin PERF_HOST_REGIME)),$(PERF_HOST_REGIME),quiet)
perf-floor: perf-probe-release
	@test -n "$(SUBJECTS)" || \
		{ echo "SUBJECTS must name at least one LABEL=PATH tree to score" >&2; exit 2; }
	$(PERF_UV) python -m benchmarks.realtree.floor \
		--probe "$(PERF_RELEASE)" \
		$(PERF_FLOOR_SUBJECT_ARGS) \
		--trials $(or $(TRIALS),30) --warmups $(or $(WARMUPS),3) \
		--host-regime $(PERF_FLOOR_HOST_REGIME) \
		--output $(PERF_FLOOR_OUT)/scoreboard-$(or $(NAME),latest).json \
		--markdown $(PERF_FLOOR_OUT)/scoreboard-$(or $(NAME),latest).md

perf-record:
	$(PERF_UV) --group dev python -m benchmarks.realtree.record $(ARGS)

# Commit a run beside its record, gzipped with no file name and an mtime of 0 so the same
# run always compresses to the same bytes, and written whole.
perf-store:
	$(PERF_RUN) store --run $(RUN) --out $(OUT)

# In `check` even though the measurement loop is not, and the distinction is the point:
# running an experiment needs a large real tree and a quiet machine, but the harness that
# decides what an experiment *means* -- the accept arithmetic, the identifier checks, the
# rendering a reader quotes from -- is ordinary code that CI can and should test. It went
# untested there long enough for four rendering branches to ship uncovered.
perf-test:
	$(PERF_UV) --group dev python -m unittest discover -s explorations/benchmarks/realtree/tests -p 'test_*.py'

PERF_LEDGER := docs/project/reports/report-2026-08-10-fdu-performance-experiments.md

# Regenerate the ledger from the committed experiment artifacts. Every number in it
# is read back out of a validated artifact, so the report cannot drift from the record.
# --group dev because the ledger validates every artifact on the way in, and the
# validator lives in that group.
perf-ledger:
	$(PERF_UV) --group dev python -m benchmarks.realtree.summary --out $(PERF_LEDGER)
	$(MAKE) docs-format

# The charted view of the same artifacts the ledger renders. Two steps because they are
# two jobs: the projection reads and validates every artifact, and the renderer draws a
# page from the projection without touching the record. Committing the projection means a
# reviewer can diff what the page is claiming, not only how it looks.
PERF_REPORT_DIR := docs/project/reports/performance-evidence

# PREPARED stamps the one date the page prints. Omit it and the existing date is kept,
# so regenerating after an artifact change does not silently redate the report.
perf-report:
	$(PERF_UV) --group dev python -m benchmarks.realtree.timeline \
		--out $(PERF_REPORT_DIR)/timeline.json \
		$(if $(PREPARED),--prepared $(PREPARED),)
	$(PERF_UV) --group dev python -m benchmarks.realtree.report_html \
		--data $(PERF_REPORT_DIR)/timeline.json --out $(PERF_REPORT_DIR)/index.html

# Both committed files are claims about the artifacts, and nothing stops a person editing
# an experiment and forgetting to regenerate. The page would then keep asserting the old
# numbers with the record's authority behind it, which is worse than having no page.
perf-report-check:
	$(PERF_UV) --group dev python -m benchmarks.realtree.timeline \
		--out $(PERF_REPORT_DIR)/timeline.json --check
	$(PERF_UV) --group dev python -m benchmarks.realtree.report_html \
		--data $(PERF_REPORT_DIR)/timeline.json --out $(PERF_REPORT_DIR)/index.html --check

# The ledger is the file people actually read, and it was the one generated view with no
# drift gate: `perf-report-check` covers the projection and the page, and AGENTS.md
# promised the ledger was covered too. It is generated and *then* formatted, so the check
# has to reproduce both steps rather than compare against raw output.
perf-ledger-check:
	@scratch=$$(mktemp -d "$${TMPDIR:-/tmp}/fdu-ledger.XXXXXX") && trap 'rm -rf "$$scratch"' EXIT && \
		$(PERF_UV) --group dev python -m benchmarks.realtree.summary \
			--out "$$scratch/ledger.md" && \
		$(FLOWMARK) --auto "$$scratch/ledger.md" >/dev/null && \
		if ! diff -u "$(PERF_LEDGER)" "$$scratch/ledger.md"; then \
			echo "$(PERF_LEDGER) is stale: the artifacts no longer produce it." >&2; \
			echo "Run \`make perf-ledger\` and commit the result." >&2; \
			exit 1; \
		fi

# The drift gates above prove the generated views match the records. This one proves the
# records agree with themselves: the verdict's headline is the figure its own results
# hold, and the kept arm is one the verdict can name. Regeneration cannot satisfy it,
# which is the point -- a wrong headline was twice regenerated into every view and
# shipped green. It also refuses to pass over zero records, or over records none of
# which stated a headline, so an empty or mis-pointed run cannot look clean.
perf-evidence-check:
	$(PERF_UV) --group dev python -m benchmarks.realtree.validate --experiments docs/project/experiments

# The experiment contract is compiled from the Pydantic model; --check fails on drift.
# Pinned in explorations/benchmarks/pyproject.toml, not `@latest`: this validator is the
# reproducibility boundary for committed evidence, so an artifact that validated
# yesterday must validate identically today.
SOFTSCHEMA ?= $(PERF_UV) --group dev softschema
SCHEMA_QUIET := python3 -c "import json,sys; d=json.load(sys.stdin); print('schema', d['out_path'], 'drift:', d['drift'])"

perf-schema:
	@PYTHONPATH=explorations $(SOFTSCHEMA) compile benchmarks.realtree.experiment:Experiment \
		--out docs/project/experiments/experiment.schema.yaml \
		--contract fdu.performance:Experiment/v1 | $(SCHEMA_QUIET)

perf-schema-check:
	@PYTHONPATH=explorations $(SOFTSCHEMA) compile benchmarks.realtree.experiment:Experiment \
		--out docs/project/experiments/experiment.schema.yaml \
		--contract fdu.performance:Experiment/v1 --check | $(SCHEMA_QUIET)

clean:
	$(CARGO) clean
