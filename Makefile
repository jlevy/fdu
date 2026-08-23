# Local development workflows. `make check` is the handoff gate: if it passes, CI should.

.DEFAULT_GOAL := help

CARGO ?= cargo
NODE ?= node
NPM ?= npm
UV ?= uv
MSRV ?= 1.85.0
NODE_INSTALL_STAMP := node_modules/.package-lock.json

.PHONY: help build release test rust-test test-golden golden-invocations portability parity-venv test-parity parity-check parity-update content-selfcheck yaml-selfcheck performance-probe test-performance golden-update check uv-version supply-chain rust-module-names fix fmt fmt-check clippy docs docs-format docs-format-check lib-only msrv audit npm-audit python-check python-concurrency python-smoke python-sdist-smoke release-test release-rehearse clean cli perf-help verify-beads

help:
	@echo "make build      Debug build of the core library and CLI, all features"
	@echo "make release    Optimized build of the core library and CLI"
	@echo "make test       Run Rust, CLI golden, and performance-harness tests"
	@echo "make test-golden  Build and compare the CLI golden contract"
	@echo "make golden-invocations  Check the corpus never resolves fdu through PATH"
	@echo "make portability  Check committed test data names no machine"
	@echo "make test-parity  Replay the corpus against the Python surface"
	@echo "make parity-update  Re-record the Python surface deviations"
	@echo "make content-selfcheck  Analyze an archive of tracked repository files"
	@echo "make test-performance  Test the performance harness and every fdu probe job"
	@echo "make golden-update  Regenerate intentional golden changes, then compare"
	@echo "make check      Handoff gate: tests, audits, docs, and installed-wheel smoke"
	@echo "make supply-chain  Verify release age, provenance, pins, and CI trust controls"
	@echo "make rust-module-names  Check Rust source filenames for ambiguity"
	@echo "make msrv       Compile all features and test the core contract on Rust $(MSRV)"
	@echo "make fix        Apply formatting and machine-applicable lint fixes"
	@echo "make audit      Dependency advisory and license audit (needs cargo-deny)"
	@echo "make python-concurrency  Prove Python GIL release and runtime borrow exclusion"
	@echo "make python-smoke  Build, install, and smoke-test the locked Python wheel"
	@echo "make cli        Build and run the CLI against this repo"
	@echo "make docs-format  Auto-format all Markdown with flowmark"
	@echo ""
	@echo "Performance loop (not part of check; see docs/project/guides/performance-loop.md)"
	@echo "make perf-baseline  Fingerprint the reference tree named by PERF_TREE"
	@echo "make perf-profile   Attribute time to functions on a symbol-bearing build"
	@echo "make perf-compare   Measure a candidate against CONTROL, interleaved and paired"
	@echo "make perf-content-profile  Attribute basic content, cache-hit, and query time"
	@echo "make perf-content-compare  Compare content jobs in 12 paired trials"
	@echo "make perf-test      Test the real-tree harness itself"
	@echo "make perf-ledger    Regenerate the experiment ledger from its artifacts"
	@echo "make perf-report    Regenerate the charted performance report from the same artifacts"
	@echo "make perf-report-check  Fail if the committed report has drifted from the artifacts"

build:
	$(CARGO) build --locked -p fdu --all-features

release:
	$(CARGO) build --locked --release -p fdu --all-features

test: rust-test test-golden content-selfcheck yaml-selfcheck test-performance

rust-test:
	$(CARGO) test --locked --all-features

test-golden: build $(NODE_INSTALL_STAMP)
	$(NPM) run test:golden

yaml-selfcheck: build $(NODE_INSTALL_STAMP)
	node scripts/check-yaml.mjs

content-selfcheck: build
	$(NODE) scripts/content-selfcheck.mjs

performance-probe:
	$(CARGO) build --locked -p fdu-core --example perf_probe --no-default-features

test-performance: performance-probe
	$(UV) run --no-project python -m unittest discover -s benchmarks/tests -p 'test_*.py'
	$(PERF_UV) --group dev python -m unittest discover -s benchmarks/realtree/tests -p 'test_*.py'

# Tryscript returns nonzero when it updates a previously failing block. The immediate
# comparison is authoritative and catches execution failures or incomplete updates.
golden-update: build $(NODE_INSTALL_STAMP)
	-$(NPM) run test:golden:update
	$(NPM) run test:golden

$(NODE_INSTALL_STAMP): package.json package-lock.json .npmrc
	$(NPM) ci

# Everything CI enforces, in the order that fails fastest.
check: uv-version supply-chain rust-module-names golden-invocations portability fmt-check clippy test docs docs-format-check perf-report-check lib-only msrv audit npm-audit python-check python-concurrency python-smoke python-sdist-smoke parity-check release-test

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
UV_BACKED_TARGETS := test-performance python-check python-concurrency python-smoke python-sdist-smoke release-test release-rehearse docs-format docs-format-check \
	perf-baseline perf-profile perf-content-profile perf-compare perf-content-compare \
	perf-compare-tools perf-record perf-test perf-ledger perf-report perf-report-check perf-schema perf-schema-check

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

# The corpus selects its binary by full path. This keeps a bare `fdu` -- which PATH
# would happily resolve to an installed build -- from creeping back in (fdu-9h2w).
golden-invocations:
	$(NODE) scripts/check-golden-invocations.mjs

# Committed test data must not name the machine that recorded it. `tryscript run --update`
# writes what it saw, so it expands named patterns into literals -- which passes forever
# on the recording machine and nowhere else.
portability:
	$(NODE) scripts/check-portability.mjs

# The parity surface needs the wheel installed, not the working tree: a shim importing
# python/fdu/ directly would pass while the built package was broken, which is the
# failure public_smoke already exists to prevent.
parity-venv: uv-version
	cd crates/fdu-py && wheel_dir="$$(mktemp -d "$${TMPDIR:-/tmp}/fdu-parity.XXXXXX")" && \
		trap 'rm -r -- "$$wheel_dir"' EXIT && \
		$(UV) run --frozen --only-group dev maturin build --locked --release --out "$$wheel_dir" && \
		$(UV) venv --clear .venv-parity && \
		$(UV) pip install --python .venv-parity --no-index --find-links "$$wheel_dir" fdu

# The two interpreters the parity harness can run against, named once. `parity-venv`
# builds the first; `python-smoke` installs the wheel into the second, which is why the
# gate reuses it rather than paying for a third build.
PARITY_PYTHON := crates/fdu-py/.venv-parity/bin/python
SMOKE_PYTHON := crates/fdu-py/.venv-smoke/bin/python

# Replay the golden corpus against the Python surface. The committed deviation file is
# non-empty by construction, so an empty result means the shim never ran (fdu-9h2w).
test-parity: build parity-venv $(NODE_INSTALL_STAMP)
	FDU_PARITY_PYTHON=$(PARITY_PYTHON) $(NODE) scripts/run-parity.mjs

# Used by the gate, where python-smoke has already installed the wheel into
# .venv-smoke; standalone runs want test-parity, which builds its own.
parity-check: build $(NODE_INSTALL_STAMP)
	FDU_PARITY_PYTHON=$(SMOKE_PYTHON) $(NODE) scripts/run-parity.mjs

parity-update: build parity-venv $(NODE_INSTALL_STAMP)
	FDU_PARITY_PYTHON=$(PARITY_PYTHON) $(NODE) scripts/run-parity.mjs --update

fmt:
	$(CARGO) fmt --all

fmt-check:
	$(CARGO) fmt --all --check

clippy:
	$(CARGO) clippy --locked --all-targets --all-features -- -D warnings

# Lint the code the host platform's build never sees.
#
# `cfg(target_os = ...)` code is invisible to a single-platform clippy run, and this
# repository keeps its one unsafe exception behind exactly such a gate — the macOS
# `getattrlistbulk` reader. CI lints on ubuntu only, so before this target that module
# had never been linted anywhere. Three separate platform-gated defects reached CI in
# one session for want of it.
#
# Checking, not building: no linker for the other platforms is needed, so this runs
# anywhere. Add the targets once with
#   rustup target add x86_64-apple-darwin x86_64-pc-windows-msvc
# and this target skips any that are missing rather than failing, so it stays usable on
# a machine that has not installed them.
CROSS_TARGETS := x86_64-apple-darwin x86_64-pc-windows-msvc

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

# How library consumers build: `default-features = false` for the minimal core, then the
# additive watch layer, neither relying on what the binary enables. The dependency guard
# proves the crate split stuck -- a library that pulls in an argument parser has back the
# dependency the split removed.
#
# The guard captures `cargo tree` before testing it, rather than piping straight into
# grep. A pipeline's status is its last command's, so a failing `cargo tree` -- renamed
# package, manifest error, resolver failure -- would hand grep empty input, grep would
# return 1, `!` would invert it to 0, and the check that proves the split would report
# success having checked nothing (fdu-cqtk).
lib-only:
	$(CARGO) test --locked -p fdu-core --no-default-features
	$(CARGO) test --locked -p fdu-core --no-default-features --features watch
	@tree="$$($(CARGO) tree -p fdu-core --all-features --prefix none)" || exit 1; \
		! printf '%s\n' "$$tree" | grep -qE '^(clap|anyhow) ' \
		|| { echo 'fdu-core must not depend on clap or anyhow; they belong to fdu'; exit 1; }

msrv:
	$(CARGO) +$(MSRV) check --locked --all-features
	$(CARGO) +$(MSRV) test --locked -p fdu-core --no-default-features

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
PYTHON_LINT_PATHS := python tests examples ../../scripts/release ../../tests/release ../../tests/parity

python-check:
	$(UV) run --directory crates/fdu-py --frozen --only-group dev \
		ruff format --check --config pyproject.toml $(PYTHON_LINT_PATHS)
	$(UV) run --directory crates/fdu-py --frozen --only-group dev \
		ruff check --config pyproject.toml $(PYTHON_LINT_PATHS)
	$(UV) run --directory crates/fdu-py --frozen --only-group dev basedpyright
	$(UV) run --directory crates/fdu-py --frozen --group dev pytest

python-smoke:
	cd crates/fdu-py && wheel_dir="$$(mktemp -d "$${TMPDIR:-/tmp}/fdu-wheel.XXXXXX")" && \
		type_dir="$$(mktemp -d "$${TMPDIR:-/tmp}/fdu-typecheck.XXXXXX")" && \
		trap 'rm -r -- "$$wheel_dir" "$$type_dir"' EXIT && \
		$(UV) run --frozen --only-group dev maturin build --locked --release --out "$$wheel_dir" && \
		$(UV) venv --clear .venv-smoke && \
		$(UV) pip install --python .venv-smoke --no-index --find-links "$$wheel_dir" fdu && \
		$(UV) run --no-project --python .venv-smoke python tests/public_smoke.py && \
		$(UV) run --no-project --python .venv-smoke python tests/smoke.py && \
		cp tests/typecheck/consumer.py tests/typecheck/pyrightconfig.json "$$type_dir/" && \
		$(UV) run --frozen --only-group dev basedpyright --pythonpath .venv-smoke/bin/python \
			--project "$$type_dir/pyrightconfig.json" && \
		wheel_path="$$(find "$$wheel_dir" -maxdepth 1 -type f -name '*.whl' -print -quit)" && \
		$(UV) tool run --isolated --no-index --from "$$wheel_path" fdu --version

python-sdist-smoke:
	cd crates/fdu-py && sdist_dir="$$(mktemp -d "$${TMPDIR:-/tmp}/fdu-sdist.XXXXXX")" && \
		trap 'rm -r -- "$$sdist_dir"' EXIT && \
		$(UV) build --no-sources --sdist --out-dir "$$sdist_dir" && \
		$(UV) venv --clear .venv-sdist && \
		$(UV) pip install --python .venv-sdist "$$sdist_dir/fdu-"*.tar.gz && \
		$(UV) run --no-project --python .venv-sdist python tests/public_smoke.py

# uv provides the interpreter so the release gates never depend on the host's system
# python3, whose version nothing else checks (the scripts need tomllib, so 3.11+).
release-test:
	$(UV) run --no-project --python 3.12 python -m unittest discover -s tests/release -p 'test_*.py'

# Build and inspect the host artifacts without contacting either registry. The explicit
# release tag exercises exact-version behavior even though a rehearsal runs on a branch.
#
# One `cargo package` naming both crates, not two invocations: `fdu` depends on `fdu-core`,
# which is not on crates.io, so packaging `fdu` alone fails to resolve it. Packaging the
# sibling first in a separate run does not help -- that puts a `.crate` in target/package,
# not in the index. Naming both in one invocation makes cargo verify each against the
# just-packaged sibling (fdu-pj9w).
release-rehearse: release-test
	artifact_dir="$$(mktemp -d "$${TMPDIR:-/tmp}/fdu-release.XXXXXX")" && \
		trap 'rm -r -- "$$artifact_dir"' EXIT && \
		version="$$($(UV) run --no-project --python 3.12 python -c 'import pathlib,tomllib; print(tomllib.loads(pathlib.Path("crates/fdu/Cargo.toml").read_text())["package"]["version"])')" && \
		export FDU_RELEASE_TAG="v$$version" && \
		$(CARGO) package --locked -p fdu-core -p fdu --allow-dirty && \
		cp "target/package/fdu-core-$$version.crate" "target/package/fdu-$$version.crate" "$$artifact_dir/" && \
		$(UV) build --directory crates/fdu-py --no-sources --sdist --out-dir "$$artifact_dir" && \
		$(UV) run --directory crates/fdu-py --frozen --only-group dev maturin build --locked --release --out "$$artifact_dir" && \
		$(UV) run --no-project --python 3.12 python scripts/release/inspect_artifacts.py "$$artifact_dir" --version "$$version" \
			--manifest "$$artifact_dir/manifest.json" --checksums "$$artifact_dir/SHA256SUMS"

cli:
	$(CARGO) run --locked --release --bin fdu -- --cache off -d 2 .

# --- Documentation ----------------------------------------------------------
#
# `--auto` owns repository-wide file discovery and applicable cleanups. The committed
# tooling lock pins the native Rust formatter used locally and in CI. Generated Markdown
# uses this same path after generation, so regenerating it cannot create format drift.
FLOWMARK := $(UV) run --project benchmarks --frozen --only-group docs flowmark

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

PERF_TREE ?= benchmarks
PERF_LABEL ?= benchmarks-self-contained
PERF_RESULTS ?= /tmp/fdu-realtree/results
PERF_SCRATCH ?= /tmp/fdu-realtree/scratch
PERF_BASELINE ?= $(PERF_RESULTS)/tree-$(PERF_LABEL).json
PERF_RELEASE := target/release/examples/perf_probe
PERF_PROFILING := target/profiling/examples/perf_probe
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
# keeps the working directory here, which is what makes `-m benchmarks.realtree` work.
PERF_UV := PYTHONDONTWRITEBYTECODE=1 $(UV) run --project benchmarks --frozen
PERF_RUN := $(PERF_UV) python -m benchmarks.realtree

.PHONY: perf-probe-release perf-probe-profiling perf-baseline perf-profile perf-compare perf-content-profile perf-content-compare perf-compare-tools perf-record perf-test perf-ledger perf-report perf-report-check perf-schema perf-schema-check

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
perf-compare: perf-probe-release
	$(PERF_RUN) measure --root $(PERF_TREE) --label $(PERF_LABEL) \
		--variant "control=$(CONTROL)" \
		--variant "candidate=$(PERF_RELEASE)" \
		--reference dust=$(shell command -v dust 2>/dev/null || echo /usr/bin/du) \
		--job cold-scan-index --job cold-scan-producer \
		--job cold-snapshot-save --job warm-revalidate \
		--job warm-snapshot-load \
		--trials $(or $(TRIALS),12) \
		--scratch $(PERF_SCRATCH) --output-dir $(PERF_RESULTS) \
		--baseline-fingerprint $(PERF_BASELINE) \
		--name $(or $(NAME),adhoc) $(PERF_MEASURE_EVIDENCE_ARGS)

perf-content-compare: perf-probe-release
	$(PERF_RUN) measure --root $(PERF_TREE) --label $(PERF_LABEL) \
		--variant "control=$(CONTROL)" \
		--variant "candidate=$(PERF_RELEASE)" \
		--job content-basic --job content-cache-hit --job code-sloc \
		--job code-sloc-cache-hit --job text-prose --job markdown-prose \
		--job document-cache-hit --job content-query \
		--trials $(or $(TRIALS),12) \
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
perf-record:
	$(PERF_UV) --group dev python -m benchmarks.realtree.record $(ARGS)

perf-test:
	$(PERF_UV) --group dev python -m unittest discover -s benchmarks/realtree/tests -p 'test_*.py'

# Regenerate the ledger from the committed experiment artifacts. Every number in it
# is read back out of a validated artifact, so the report cannot drift from the record.
# --group dev because the ledger validates every artifact on the way in, and the
# validator lives in that group.
perf-ledger:
	$(PERF_UV) --group dev python -m benchmarks.realtree.summary
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

# The experiment contract is compiled from the Pydantic model; --check fails on drift.
# Pinned in benchmarks/pyproject.toml, not `@latest`: this validator is the
# reproducibility boundary for committed evidence, so an artifact that validated
# yesterday must validate identically today.
SOFTSCHEMA ?= $(PERF_UV) --group dev softschema
SCHEMA_QUIET := python3 -c "import json,sys; d=json.load(sys.stdin); print('schema', d['out_path'], 'drift:', d['drift'])"

perf-schema:
	@PYTHONPATH=. $(SOFTSCHEMA) compile benchmarks.realtree.experiment:Experiment \
		--out docs/project/experiments/experiment.schema.yaml \
		--contract fdu.performance:Experiment/v1 | $(SCHEMA_QUIET)

perf-schema-check:
	@PYTHONPATH=. $(SOFTSCHEMA) compile benchmarks.realtree.experiment:Experiment \
		--out docs/project/experiments/experiment.schema.yaml \
		--contract fdu.performance:Experiment/v1 --check | $(SCHEMA_QUIET)

clean:
	$(CARGO) clean
