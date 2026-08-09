# Local development workflows. `make check` is the handoff gate: if it passes, CI should.

.DEFAULT_GOAL := help

CARGO ?= cargo

.PHONY: help build release test check fix fmt fmt-check clippy docs audit python-smoke clean cli

help:
	@echo "make build      Debug build of the core library and CLI, all features"
	@echo "make release    Optimized build of the core library and CLI"
	@echo "make test       Run the test suite"
	@echo "make check      Handoff gate: Rust gates, audit, and installed-wheel smoke"
	@echo "make fix        Apply formatting and machine-applicable lint fixes"
	@echo "make audit      Dependency advisory and license audit (needs cargo-deny)"
	@echo "make python-smoke  Build, install, and smoke-test the locked Python wheel"
	@echo "make cli        Build and run the CLI against this repo"

build:
	$(CARGO) build --locked -p fdu --all-features

release:
	$(CARGO) build --locked --release -p fdu --all-features

test:
	$(CARGO) test --locked --all-features

# Everything CI enforces, in the order that fails fastest.
check: fmt-check clippy test docs lib-only audit python-smoke

fmt:
	$(CARGO) fmt --all

fmt-check:
	$(CARGO) fmt --all --check

clippy:
	$(CARGO) clippy --locked --all-targets --all-features -- -D warnings

docs:
	RUSTDOCFLAGS="-D warnings" $(CARGO) doc --locked --no-deps --all-features

# Library consumers take `default-features = false`; this proves that path still
# compiles and tests, rather than only ever exercising the CLI-enabled build.
lib-only:
	$(CARGO) test --locked -p fdu --no-default-features

fix:
	$(CARGO) fmt --all
	$(CARGO) clippy --locked --all-targets --all-features --fix --allow-dirty --allow-staged

audit:
	$(CARGO) deny --locked check

python-smoke:
	cd crates/fdu-py && wheel_dir="$$(mktemp -d "$${TMPDIR:-/tmp}/fdu-wheel.XXXXXX")" && \
		trap 'rm -r -- "$$wheel_dir"' EXIT && \
		uv run --frozen --only-group dev maturin build --locked --release --out "$$wheel_dir" && \
		uv venv --clear .venv-smoke && \
		uv pip install --python .venv-smoke "$$wheel_dir"/*.whl && \
		uv run --no-project --python .venv-smoke python tests/smoke.py

cli:
	$(CARGO) run --locked --release --bin fdu -- --no-cache -d 2 .

clean:
	$(CARGO) clean
