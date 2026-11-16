# Root orchestration. `make help` lists everything. Run inside `nix develop` (or direnv allow).
SHELL := bash
.DEFAULT_GOAL := help

ESP_EXPORT ?= $(HOME)/export-esp.sh
GRAPHIFY   := $(shell which graphify 2>/dev/null || echo $(HOME)/.local/bin/graphify)
OPENSPEC   := $(shell which openspec 2>/dev/null || echo npx --yes @fission-ai/openspec@1.14.2)
PORT       ?= /dev/ttyACM0
# Device authentication key (64 hex chars). Untracked; created once by `make fw-key`.
KEY_FILE   := firmware/.beewave-key
# Firmware update signing: private key (untracked, never leaves your machine) and its public half.
OTA_KEY    := firmware/.beewave-ota.pem
OTA_PUB    := firmware/.beewave-ota.pub
# Load the Xtensa env and the keys, then run in firmware/
FW = . $(ESP_EXPORT) && cd firmware && export BEEWAVE_KEY=$$(cat .beewave-key) BEEWAVE_OTA_PUBKEY=$$(cat .beewave-ota.pub) &&

.PHONY: help
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
.PHONY: web-dev web-mock web-lan web-build web-test
web-dev: ## Dev server (https needed for BLE off-localhost)
	cd web && pnpm run dev
web-mock: ## Dev server with the fake device (open /?mock)
	cd web && pnpm run dev --open '/?mock'
web-lan: ## HTTPS dev server reachable from your phone on the same Wi-Fi
	cd web && HTTPS=1 pnpm run dev --host
web-build: ## Production PWA bundle -> web/dist
	cd web && pnpm run build
web-test: ## Web unit tests (codec vs Rust golden vectors)
	cd web && pnpm test

# ---- firmware ----------------------------------------------------------------
.PHONY: fw-build fw-flash fw-monitor fw-run fw-image fw-size fw-clean board-info
fw-key: ## Create the device key once (untracked) and print it for the PWA
	@test -f $(KEY_FILE) || (umask 077 && head -c 32 /dev/urandom | od -An -tx1 | tr -d ' \n' > $(KEY_FILE))
	@echo "Device key (enter it in the PWA, keep it secret): $$(cat $(KEY_FILE))"

fw-ota-key: ## Create the OTA signing key once (untracked) and its public key for the firmware
	@test -f $(OTA_KEY) || (umask 077 && openssl genpkey -algorithm ed25519 -out $(OTA_KEY))
	@openssl pkey -in $(OTA_KEY) -pubout -outform DER | tail -c 32 | od -An -tx1 | tr -d ' \n' > $(OTA_PUB)
	@echo "OTA signing public key (built into the firmware): $$(cat $(OTA_PUB))"

fw-build: fw-key fw-ota-key ## Build release firmware
	$(FW) cargo build --release
fw-flash: fw-key fw-ota-key ## Flash release firmware to $(PORT)
	$(FW) cargo espflash flash --release --port $(PORT)
fw-monitor: ## Serial monitor
	$(FW) cargo espflash monitor --port $(PORT)
fw-run: fw-key fw-ota-key ## Build + flash + monitor
	$(FW) cargo espflash flash --release --port $(PORT) --monitor
fw-image: fw-key fw-ota-key ## Signed OTA update: dist/beewave-app.bin + .sig (+ .sha256)
	$(FW) cargo espflash save-image --release --chip esp32 ../dist/beewave-app.bin
	cd dist && sha256sum beewave-app.bin | tee beewave-app.bin.sha256
	# Sign the image's SHA-256 digest (what the device checks) with Ed25519.
	cd dist && openssl dgst -sha256 -binary beewave-app.bin > beewave-app.digest \
	  && openssl pkeyutl -sign -inkey ../$(OTA_KEY) -rawin -in beewave-app.digest -out beewave-app.bin.sig \
	  && rm beewave-app.digest && ls -l beewave-app.bin.sig
fw-size: fw-key fw-ota-key ## Flash/RAM usage of the release build
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
