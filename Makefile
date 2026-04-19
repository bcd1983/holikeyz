.PHONY: all build build-debug test install clean run-cli run-service install-extension uninstall-extension enable-service disable-service

PREFIX ?= /usr/local
BINDIR = $(PREFIX)/bin
SYSTEMD_USER_DIR = $(HOME)/.config/systemd/user
DBUS_SERVICE_DIR = /usr/share/dbus-1/services
GNOME_EXT_DIR = $(HOME)/.local/share/gnome-shell/extensions
EXT_UUID = holikeyz-ring-light@example.com

all: build

build:
	cargo build --release

build-debug:
	cargo build

test:
	cargo test

run-cli:
	cargo run --bin holikeyz-cli -- $(ARGS)

run-service:
	RUST_LOG=info cargo run --bin holikeyz-service

install: build
	install -Dm755 target/release/holikeyz-cli $(DESTDIR)$(BINDIR)/holikeyz-cli
	install -Dm755 target/release/holikeyz-service $(DESTDIR)$(BINDIR)/holikeyz-service
	install -Dm644 systemd/holikeyz-ring-light.service $(SYSTEMD_USER_DIR)/holikeyz-ring-light.service
	install -Dm644 dbus/com.holikeyz.RingLight.service $(DESTDIR)$(DBUS_SERVICE_DIR)/com.holikeyz.RingLight.service

install-extension:
	mkdir -p $(GNOME_EXT_DIR)/$(EXT_UUID)
	cp -r gnome-extension/$(EXT_UUID)/* $(GNOME_EXT_DIR)/$(EXT_UUID)/
	@echo "Extension installed. Restart GNOME Shell (Alt+F2, 'r') and enable it with: gnome-extensions enable $(EXT_UUID)"

uninstall-extension:
	rm -rf $(GNOME_EXT_DIR)/$(EXT_UUID)

enable-service:
	systemctl --user daemon-reload
	systemctl --user enable holikeyz-ring-light.service
	systemctl --user start holikeyz-ring-light.service

disable-service:
	systemctl --user stop holikeyz-ring-light.service
	systemctl --user disable holikeyz-ring-light.service

clean:
	cargo clean
	rm -f $(DESTDIR)$(BINDIR)/holikeyz-cli
	rm -f $(DESTDIR)$(BINDIR)/holikeyz-service
