# Local development workflows. `make check` is the handoff gate: if it passes, CI should.

.DEFAULT_GOAL := help

CARGO ?= cargo

.PHONY: help build release test check fix fmt fmt-check clippy docs audit clean cli

help:
	@echo "make build      Debug build, all features"
	@echo "make release    Optimized build"
	@echo "make test       Run the test suite"
	@echo "make check      Handoff gate: fmt, clippy, tests, docs, lib-only build"
	@echo "make fix        Apply formatting and machine-applicable lint fixes"
	@echo "make audit      Dependency advisory and license audit (needs cargo-deny)"
	@echo "make cli        Build and run the CLI against this repo"

build:
	$(CARGO) build --all-features

release:
	$(CARGO) build --release --all-features

test:
	$(CARGO) test --all-features

# Everything CI enforces, in the order that fails fastest.
check: fmt-check clippy test docs lib-only

fmt:
	$(CARGO) fmt --all

fmt-check:
	$(CARGO) fmt --all --check

clippy:
	$(CARGO) clippy --all-targets --all-features -- -D warnings

docs:
	RUSTDOCFLAGS="-D warnings" $(CARGO) doc --no-deps --all-features

# Library consumers take `default-features = false`; this proves that path still
# compiles and tests, rather than only ever exercising the CLI-enabled build.
lib-only:
	$(CARGO) test -p fdu --no-default-features

fix:
	$(CARGO) fmt --all
	$(CARGO) clippy --all-targets --all-features --fix --allow-dirty --allow-staged

audit:
	$(CARGO) deny check

cli:
	$(CARGO) run --release --bin fdu -- --no-cache -d 2 .

clean:
	$(CARGO) clean
