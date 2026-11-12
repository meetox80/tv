.PHONY: dev build release size clean install lint install-root web-install web-build web-dev error

TAURI      := bun run tauri
WEB_SETUP  := src-tauri/web/setup
WEB_PANEL  := src-tauri/web/panel
WEB_VOTE   := src-tauri/web/vote
WEB_DIRS   := $(WEB_SETUP) $(WEB_PANEL) $(WEB_VOTE)
ROOT_DEPS  := node_modules/.bin/tauri
WEB_DEPS   := $(addsuffix /node_modules/.bin/vite,$(WEB_DIRS))
BIN        := src-tauri/target/release/zstio-tv
BUNDLE     := src-tauri/target/release/bundle

define bun_deps
$(1)/node_modules/.bin/$(2): $(1)/package.json $(1)/bun.lock
	cd $(1) && bun install
	@test -x $$@ || { echo ">>> $(1)/node_modules is incomplete, reinstalling from scratch"; rm -rf $(1)/node_modules; cd $(1) && bun install; }
	@test -x $$@ || { echo ">>> $(1): bun install finished but $(2) is still missing"; exit 1; }
endef

$(foreach d,$(WEB_DIRS),$(eval $(call bun_deps,$(d),vite)))

$(ROOT_DEPS): package.json bun.lock
	bun install
	@test -x $@ || { echo ">>> node_modules is incomplete, reinstalling from scratch"; rm -rf node_modules; bun install; }
	@test -x $@ || { echo ">>> bun install finished but the tauri CLI is still missing"; exit 1; }

install-root: $(ROOT_DEPS)

web-install: $(WEB_DEPS)

dev: install-root web-build
	ZSTIO_TV=1 $(TAURI) dev

build: install-root web-build
	$(TAURI) build

release: build size

size:
	@du -h $(BIN) 2>/dev/null || echo "no binary at $(BIN)"
	@find $(BUNDLE) -mindepth 1 -maxdepth 2 -exec du -sh {} + 2>/dev/null || true

build-windows: install-root web-build
	$(TAURI) build --target x86_64-pc-windows-msvc

build-linux: install-root web-build
	$(TAURI) build --target x86_64-unknown-linux-gnu

build-macos-intel: install-root web-build
	$(TAURI) build --target x86_64-apple-darwin

build-macos-arm: install-root web-build
	$(TAURI) build --target aarch64-apple-darwin

build-macos-universal: install-root web-build
	$(TAURI) build --target universal-apple-darwin

web-build: web-install
	cd $(WEB_SETUP) && bun run build
	cd $(WEB_PANEL) && bun run build
	cd $(WEB_VOTE) && bun run build

web-dev: $(WEB_SETUP)/node_modules/.bin/vite
	cd $(WEB_SETUP) && bun run dev

clean:
	cargo clean -p zstio-tv --manifest-path src-tauri/Cargo.toml
	rm -rf node_modules dist src-tauri/target $(WEB_SETUP)/dist $(WEB_SETUP)/node_modules $(WEB_PANEL)/dist $(WEB_PANEL)/node_modules $(WEB_VOTE)/dist $(WEB_VOTE)/node_modules

install:
	rm -rf node_modules $(addsuffix /node_modules,$(WEB_DIRS))
	bun install
	cd $(WEB_SETUP) && bun install
	cd $(WEB_PANEL) && bun install
	cd $(WEB_VOTE) && bun install

lint: install-root web-install
	bun run vue-tsc --noEmit
	cd $(WEB_SETUP) && bun run vue-tsc --noEmit
	cd $(WEB_PANEL) && bun run vue-tsc --noEmit
	cd $(WEB_VOTE) && bun run vue-tsc --noEmit
	cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings

CASE ?= all

error:
	-cargo run -q --manifest-path src-tauri/Cargo.toml --example errors -- $(CASE)
