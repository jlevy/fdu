# Local development workflows. `make check` is the handoff gate: if it passes, CI should.

.DEFAULT_GOAL := help

CARGO ?= cargo
NPM ?= npm
MSRV ?= 1.85.0
NODE_INSTALL_STAMP := node_modules/.package-lock.json

.PHONY: help build release test rust-test test-golden test-performance golden-update check supply-chain fix fmt fmt-check clippy docs lib-only msrv audit npm-audit python-concurrency python-smoke clean cli

help:
	@echo "make build      Debug build of the core library and CLI, all features"
	@echo "make release    Optimized build of the core library and CLI"
	@echo "make test       Run Rust, CLI golden, and performance-harness tests"
	@echo "make test-golden  Build and compare the CLI golden contract"
	@echo "make test-performance  Test the performance corpus and oracle tooling"
	@echo "make golden-update  Regenerate intentional golden changes, then compare"
	@echo "make check      Handoff gate: tests, audits, docs, and installed-wheel smoke"
	@echo "make supply-chain  Verify release age, provenance, pins, and CI trust controls"
	@echo "make msrv       Compile all features and test the core contract on Rust $(MSRV)"
	@echo "make fix        Apply formatting and machine-applicable lint fixes"
	@echo "make audit      Dependency advisory and license audit (needs cargo-deny)"
	@echo "make python-concurrency  Prove Python GIL release and runtime borrow exclusion"
	@echo "make python-smoke  Build, install, and smoke-test the locked Python wheel"
	@echo "make cli        Build and run the CLI against this repo"

build:
	$(CARGO) build --locked -p fdu --all-features

release:
	$(CARGO) build --locked --release -p fdu --all-features

test: rust-test test-golden test-performance

rust-test:
	$(CARGO) test --locked --all-features

test-golden: build $(NODE_INSTALL_STAMP)
	$(NPM) run test:golden

test-performance:
	uv run --no-project python -m unittest discover -s benchmarks/tests -p 'test_*.py'

# Tryscript returns nonzero when it updates a previously failing block. The immediate
# comparison is authoritative and catches execution failures or incomplete updates.
golden-update: build $(NODE_INSTALL_STAMP)
	-$(NPM) run test:golden:update
	$(NPM) run test:golden

$(NODE_INSTALL_STAMP): package.json package-lock.json .npmrc
	$(NPM) ci

# Everything CI enforces, in the order that fails fastest.
check: supply-chain fmt-check clippy test docs lib-only msrv audit npm-audit python-concurrency python-smoke

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
	$(CARGO) run --locked --release --bin fdu -- --no-cache -d 2 .

clean:
	$(CARGO) clean
