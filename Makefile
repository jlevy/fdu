# Local development workflows. `make check` is the handoff gate: if it passes, CI should.

.DEFAULT_GOAL := help

CARGO ?= cargo
NPM ?= npm
MSRV ?= 1.85.0
NODE_INSTALL_STAMP := node_modules/.package-lock.json

.PHONY: help build release test rust-test test-golden performance-probe test-performance golden-update check supply-chain fix fmt fmt-check clippy docs docs-format docs-format-check lib-only msrv audit npm-audit python-concurrency python-smoke clean cli perf-help verify-beads

help:
	@echo "make build      Debug build of the core library and CLI, all features"
	@echo "make release    Optimized build of the core library and CLI"
	@echo "make test       Run Rust, CLI golden, and performance-harness tests"
	@echo "make test-golden  Build and compare the CLI golden contract"
	@echo "make test-performance  Test the performance harness and every fdu probe job"
	@echo "make golden-update  Regenerate intentional golden changes, then compare"
	@echo "make check      Handoff gate: tests, audits, docs, and installed-wheel smoke"
	@echo "make supply-chain  Verify release age, provenance, pins, and CI trust controls"
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
	@echo "make perf-test      Test the real-tree harness itself"
	@echo "make perf-ledger    Regenerate the experiment ledger from its artifacts"

build:
	$(CARGO) build --locked -p fdu --all-features

release:
	$(CARGO) build --locked --release -p fdu --all-features

test: rust-test test-golden test-performance

rust-test:
	$(CARGO) test --locked --all-features

test-golden: build $(NODE_INSTALL_STAMP)
	$(NPM) run test:golden

performance-probe:
	$(CARGO) build --locked -p fdu --example perf_probe --no-default-features

test-performance: performance-probe
	uv run --no-project python -m unittest discover -s benchmarks/tests -p 'test_*.py'

# Tryscript returns nonzero when it updates a previously failing block. The immediate
# comparison is authoritative and catches execution failures or incomplete updates.
golden-update: build $(NODE_INSTALL_STAMP)
	-$(NPM) run test:golden:update
	$(NPM) run test:golden

$(NODE_INSTALL_STAMP): package.json package-lock.json .npmrc
	$(NPM) ci

# Everything CI enforces, in the order that fails fastest.
check: supply-chain fmt-check clippy test docs docs-format-check lib-only msrv audit npm-audit python-concurrency python-smoke

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

fmt:
	$(CARGO) fmt --all

fmt-check:
	$(CARGO) fmt --all --check

clippy:
	$(CARGO) clippy --locked --all-targets --all-features -- -D warnings

docs:
	RUSTDOCFLAGS="-D warnings" $(CARGO) doc --locked --no-deps --all-features

# Library consumers take `default-features = false`; prove both the minimal core and
# the additive watch layer without accidentally relying on CLI defaults.
lib-only:
	$(CARGO) test --locked -p fdu --no-default-features
	$(CARGO) test --locked -p fdu --no-default-features --features watch

msrv:
	$(CARGO) +$(MSRV) check --locked --all-features
	$(CARGO) +$(MSRV) test --locked -p fdu --no-default-features

fix:
	$(CARGO) fmt --all
	$(CARGO) clippy --locked --all-targets --all-features --fix --allow-dirty --allow-staged

audit:
	$(CARGO) deny --locked check

npm-audit: $(NODE_INSTALL_STAMP)
	$(NPM) audit --audit-level=moderate

python-concurrency:
	uv run --directory crates/fdu-py --frozen --only-group dev \
		cargo test --locked -p fdu-py --lib --no-default-features

python-smoke:
	cd crates/fdu-py && wheel_dir="$$(mktemp -d "$${TMPDIR:-/tmp}/fdu-wheel.XXXXXX")" && \
		trap 'rm -r -- "$$wheel_dir"' EXIT && \
		uv run --frozen --only-group dev maturin build --locked --release --out "$$wheel_dir" && \
		uv venv --clear .venv-smoke && \
		uv pip install --python .venv-smoke --no-index --find-links "$$wheel_dir" fdu && \
		uv run --no-project --python .venv-smoke python tests/smoke.py && \
		wheel_path="$$(find "$$wheel_dir" -maxdepth 1 -type f -name '*.whl' -print -quit)" && \
		uvx --isolated --no-index --from "$$wheel_path" fdu --version

cli:
	$(CARGO) run --locked --release --bin fdu -- --cache off -d 2 .

# --- Documentation ----------------------------------------------------------
#
# `--auto` owns repository-wide file discovery and applicable cleanups. The committed
# tooling lock pins the native Rust formatter used locally and in CI. Generated Markdown
# uses this same path after generation, so regenerating it cannot create format drift.
FLOWMARK := uv run --project benchmarks --frozen --only-group docs flowmark

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
# PERF_TREE names the reference tree. Clone a real checkout with `cp -cR` first so
# the tree cannot change underneath a run.

PERF_TREE ?= benchmarks/corpus/realtree/metabrowser
PERF_LABEL ?= $(notdir $(PERF_TREE))
PERF_RELEASE := target/release/examples/perf_probe
PERF_PROFILING := target/profiling/examples/perf_probe
# The harness runs from the repo root against a committed, frozen environment, so a
# benchmark run resolves nothing at invocation time. `--project` (not `--directory`)
# keeps the working directory here, which is what makes `-m benchmarks.realtree` work.
PERF_UV := uv run --project benchmarks --frozen
PERF_RUN := $(PERF_UV) python -m benchmarks.realtree

.PHONY: perf-probe-release perf-probe-profiling perf-baseline perf-profile perf-compare perf-record perf-test perf-ledger perf-schema perf-schema-check

perf-probe-release:
	$(CARGO) build --locked --release -p fdu --example perf_probe --no-default-features

perf-probe-profiling:
	$(CARGO) build --locked --profile profiling -p fdu --example perf_probe --no-default-features

# Record what the tree looks like now, so later runs can prove they measured the same one.
perf-baseline:
	$(PERF_RUN) baseline --root $(PERF_TREE) --label $(PERF_LABEL)

# Where does the time go? Attribution only; never a timing claim.
perf-profile: perf-probe-profiling
	$(PERF_RUN) profile --root $(PERF_TREE) --binary $(PERF_PROFILING) \
		--job cold-scan-index --job warm-revalidate --label $(or $(NAME),latest)

# Is the candidate faster than the control? Set CONTROL to a saved reference binary.
CONTROL ?= $(PERF_RELEASE)
perf-compare: perf-probe-release
	$(PERF_RUN) measure --root $(PERF_TREE) --label $(PERF_LABEL) \
		--variant "control=$(CONTROL)" \
		--variant "candidate=$(PERF_RELEASE)" \
		--reference dust=$(shell command -v dust 2>/dev/null || echo /usr/bin/du) \
		--job cold-scan-index --job warm-revalidate \
		--trials $(or $(TRIALS),12) \
		--baseline-fingerprint benchmarks/results/realtree/tree-$(PERF_LABEL).json \
		--name $(or $(NAME),adhoc)

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
