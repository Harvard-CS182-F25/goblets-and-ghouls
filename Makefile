PY              ?= python3
MATURIN         ?= maturin
CARGO           ?= cargo
PY_CRATE        ?= gg_core_py/Cargo.toml
BIN             ?= stub_gen
FEAT_PYMODULE   ?= pymodule,bevy
PROFILE         ?= release
CARGO_FLAGS     ?= --no-default-features
CARGO_TARGET    ?=
DEV_RUN         ?= dev_run.py
CONFIG          ?=
POLICY          ?=
VALUE           ?=
GENERATION_SEED ?=
EPISODE_SEED    ?=
HEADLESS        ?=
DEV_RUN_ARGS    ?=

export PYO3_PYTHON := $(shell command -v $(PY))

.PHONY: help develop stubs wheel clean fmt test check-arch show-config dev-run

help:
	@echo "Targets:"
	@echo "  make develop        - maturin develop (installs the Python extension into current venv)"
	@echo "  make stubs          - develop the extension, then run stub generator bin"
	@echo "  make wheel          - build wheels for distribution"
	@echo "  make dev-run        - run the local Python Bevy/debug helper"
	@echo "Vars (override: make VAR=value):"
	@echo "  PY=$(PY) | PY_CRATE=$(PY_CRATE) | BIN=$(BIN) | FEAT_PYMODULE=$(FEAT_PYMODULE) | PROFILE=$(PROFILE)"
	@echo "  CONFIG=... | POLICY=... | VALUE=... | GENERATION_SEED=... | EPISODE_SEED=... | HEADLESS=1 | DEV_RUN_ARGS='...'"

develop:
	@echo ">> Using Python: $(PYO3_PYTHON)"
	$(MATURIN) develop -m $(PY_CRATE) $(RELEASE_FLAG) $(if $(FEAT_PYMODULE),--features $(FEAT_PYMODULE),)

stubs: develop
	@echo ">> Running stub generator bin: $(BIN)"
	$(CARGO) run -p gg_core_py $(CARGO_TARGET) --bin $(BIN) $(CARGO_FLAGS)

wheel:
	@echo ">> Building wheels via maturin"
	$(MATURIN) build -m $(PY_CRATE) $(RELEASE_FLAG) $(if $(FEAT_PYMODULE),--features $(FEAT_PYMODULE),)

clean:
	$(CARGO) clean
	@rm -rf target/wheels || true

dev-run: develop
	@echo ">> Running $(DEV_RUN)"
	$(PY) $(DEV_RUN) \
		$(if $(CONFIG),--config $(CONFIG),) \
		$(if $(POLICY),--policy $(POLICY),) \
		$(if $(VALUE),--value $(VALUE),) \
		$(if $(GENERATION_SEED),--generation-seed $(GENERATION_SEED),) \
		$(if $(EPISODE_SEED),--episode-seed $(EPISODE_SEED),) \
		$(if $(HEADLESS),--headless,) \
		$(DEV_RUN_ARGS)
