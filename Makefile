# Root orchestration. `make help` lists everything. Run inside `nix develop` (or direnv allow).
SHELL := bash
.DEFAULT_GOAL := help

ESP_EXPORT ?= $(HOME)/export-esp.sh
GRAPHIFY   := $(shell which graphify 2>/dev/null || echo $(HOME)/.local/bin/graphify)
OPENSPEC   := $(shell which openspec 2>/dev/null || echo npx --yes @fission-ai/openspec@1.14.2)
PORT       ?= /dev/ttyACM0
# Device authentication key (64 hex chars). Untracked; created once by `make fw-key`.
KEY_FILE   := firmware/.beewave-key
# How the device gets its key. `auto` (default): none built in, the first phone to connect claims the
# device and its key is stored in flash. `file`: bake firmware/.beewave-key in (the old behaviour).
FW_KEY     ?= auto
FW_KEY_DEP := $(if $(filter file,$(FW_KEY)),fw-key)
KEY_EXPORT := $(if $(filter file,$(FW_KEY)),export BEEWAVE_KEY=$$(cat .beewave-key),unset BEEWAVE_KEY)
# Firmware update signing: private key (untracked, never leaves your machine) and its public half.
OTA_KEY    := firmware/.beewave-ota.pem
OTA_PUB    := firmware/.beewave-ota.pub
# Optional firmware cargo features, e.g. `make fw-run FEATURES=coex_debug` (latency logging).
FEATURES   ?=
CARGO_FEAT  = $(if $(FEATURES),--features $(FEATURES))
# Load the Xtensa env and the keys, then run in firmware/
FW = . $(ESP_EXPORT) && cd firmware && $(KEY_EXPORT) && export BEEWAVE_OTA_PUBKEY=$$(cat .beewave-ota.pub) &&

.PHONY: help logo
help: ## Show this help
	@awk 'BEGIN{FS=":.*##"} /^[a-zA-Z0-9_.-]+:.*##/ {printf "  \033[1m%-16s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# ---- setup -----------------------------------------------------------------
.PHONY: setup doctor
setup: ## One-time: ESP Xtensa Rust toolchain + web deps
	espup install
	cd web && pnpm install --frozen-lockfile

doctor: ## Check tools, toolchain and board
	@for t in cargo rustup node pnpm espflash ldproxy espup; do printf "%-10s" $$t; command -v $$t >/dev/null && echo ok || echo MISSING; done
	@test -f $(ESP_EXPORT) && echo "esp env   ok ($(ESP_EXPORT))" || echo "esp env   MISSING: run 'make setup'"
	@test -r $(PORT) -a -w $(PORT) && echo "board     ok ($(PORT))" || echo "board     not accessible ($(PORT))"

# ---- host crates (core + protocol) -----------------------------------------
.PHONY: test test-rust vectors check-nostd
test: test-rust web-test ## All host tests (no hardware needed)

test-rust: ## Rust core + protocol unit tests
	cargo test --workspace --features beewave-core/std

vectors: ## Regenerate golden wire vectors consumed by the web tests
	cargo test -p beewave-protocol --features std --test vectors

check-nostd: ## Core must build without std
	cargo build -p beewave-core

# ---- web ---------------------------------------------------------------------
.PHONY: web-dev web-mock web-lan web-build web-test logo
web-dev: ## Dev server (https needed for BLE off-localhost)
	cd web && pnpm run dev
web-mock: ## Dev server with the fake device (open /?mock)
	cd web && pnpm run dev --open '/?mock'
web-lan: ## HTTPS dev server reachable from your phone on the same Wi-Fi
	cd web && HTTPS=1 pnpm run dev --host
logo: ## Regenerate the logo assets (SVG and PNG icons) from web/src/shared/brand/crest.svg
	./scripts/make-logo.sh

web-build: ## Production PWA bundle -> web/dist
	cd web && pnpm run build
web-test: ## Web unit tests (codec vs Rust golden vectors)
	cd web && pnpm test

# ---- firmware ----------------------------------------------------------------
.PHONY: fw-build fw-flash fw-monitor fw-run fw-image fw-size fw-clean fw-unclaim fw-fingerprint board-info
fw-key: ## Create the device key once (untracked) and print it for the PWA
	@test -f $(KEY_FILE) || (umask 077 && head -c 32 /dev/urandom | od -An -tx1 | tr -d ' \n' > $(KEY_FILE))
	@echo "Device key (enter it in the PWA, keep it secret): $$(cat $(KEY_FILE))"

fw-ota-key: ## Create the OTA signing key once (untracked) and its public key for the firmware
	@test -f $(OTA_KEY) || (umask 077 && openssl genpkey -algorithm ed25519 -out $(OTA_KEY))
	@openssl pkey -in $(OTA_KEY) -pubout -outform DER | tail -c 32 | od -An -tx1 | tr -d ' \n' > $(OTA_PUB)
	@echo "OTA signing public key (built into the firmware): $$(cat $(OTA_PUB))"

fw-build: $(FW_KEY_DEP) fw-ota-key ## Build release firmware
	$(FW) cargo build --release $(CARGO_FEAT)
fw-flash: $(FW_KEY_DEP) fw-ota-key ## Flash release firmware to $(PORT)
	$(FW) cargo espflash flash --release --port $(PORT) $(CARGO_FEAT)
fw-fingerprint: ## Fingerprint of the OTA signing public key (the board shows the same value in Settings > OTA)
	@test -f $(OTA_PUB) || (echo "$(OTA_PUB) missing: run 'make fw-ota-key'" >&2 && exit 1)
	@python3 -c "import hashlib,sys; print(hashlib.sha256(bytes.fromhex(open(sys.argv[1]).read().strip())).hexdigest()[:32])" $(OTA_PUB)
fw-unclaim: ## Forget the stored device key (erases the NVS partition) so the next phone can claim it
	. $(ESP_EXPORT) && espflash erase-region --port $(PORT) 0x9000 0x6000
fw-monitor: ## Serial monitor
	$(FW) cargo espflash monitor --port $(PORT)
fw-run: $(FW_KEY_DEP) fw-ota-key ## Build + flash + monitor
	$(FW) cargo espflash flash --release --port $(PORT) --monitor $(CARGO_FEAT)
fw-image: $(FW_KEY_DEP) fw-ota-key ## OTA update image dist/beewave-app.bin (+ .sha256), signed (.sig)
	$(FW) cargo espflash save-image --release --chip esp32 ../dist/beewave-app.bin $(CARGO_FEAT)
	cd dist && sha256sum beewave-app.bin | tee beewave-app.bin.sha256
	$(MAKE) --no-print-directory fw-sign

fw-sign: fw-ota-key ## Sign dist/beewave-app.bin with the OTA key -> dist/beewave-app.bin.sig
	@test -f dist/beewave-app.bin || (echo "dist/beewave-app.bin missing: run 'make fw-image'" && exit 1)
	@echo "OTA signing key fingerprint: $$($(MAKE) --no-print-directory -s fw-fingerprint)"
	cd dist && openssl dgst -sha256 -binary beewave-app.bin > beewave-app.digest \
	  && openssl pkeyutl -sign -inkey ../$(OTA_KEY) -rawin -in beewave-app.digest -out beewave-app.bin.sig \
	  && rm beewave-app.digest && ls -l beewave-app.bin.sig
fw-size: $(FW_KEY_DEP) fw-ota-key ## Flash/RAM usage of the release build
	$(FW) cargo espflash save-image --release --chip esp32 /tmp/beewave.bin >/dev/null && ls -l /tmp/beewave.bin
fw-clean: ## Remove firmware build output (keeps downloaded ESP-IDF in firmware/.embuild)
	cd firmware && cargo clean
board-info: ## Detect the chip on $(PORT)
	cargo +stable espflash board-info --port $(PORT)

# ---- specs & code map -------------------------------------------------------
.PHONY: spec graphify
spec: ## Validate OpenSpec specs and changes
	$(OPENSPEC) validate --all --strict --no-interactive
graphify: ## AST knowledge graph -> graphify-out/ (GRAPH_REPORT.md, graph.html)
	$(GRAPHIFY) . --code-only
	$(GRAPHIFY) cluster-only . --no-label

# ---- umbrella ----------------------------------------------------------------
.PHONY: ci check fmt fmt-check clean
check: fmt-check test check-nostd web-build spec ## Full local gate (no hardware)
ci: check ## What CI runs
fmt-check: ## Rust format check
	cargo fmt --all -- --check
fmt: ## Format Rust + Nix
	cargo fmt --all
	nix fmt 2>/dev/null || true
clean: ## Remove host build output
	cargo clean
	rm -rf web/dist
