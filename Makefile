.PHONY: all build install clean test run-cli run-service install-extension uninstall-extension

PREFIX ?= /usr/local
BINDIR = $(PREFIX)/bin
SYSTEMD_USER_DIR = $(HOME)/.config/systemd/user
GNOME_EXT_DIR = $(HOME)/.local/share/gnome-shell/extensions

all: build

build:
	cargo build --release

build-debug:
	cargo build

test:
	cargo test

run-cli:
	cargo run --bin elgato-cli -- $(ARGS)

run-service:
	RUST_LOG=info cargo run --bin elgato-dbus-service

install: build
	install -Dm755 target/release/elgato-cli $(DESTDIR)$(BINDIR)/elgato-cli
	install -Dm755 target/release/elgato-dbus-service $(DESTDIR)$(BINDIR)/elgato-dbus-service
	install -Dm644 systemd/elgato-ring-light.service $(SYSTEMD_USER_DIR)/elgato-ring-light.service
	install -Dm644 dbus/com.elgato.RingLight.service /usr/share/dbus-1/services/com.elgato.RingLight.service

install-extension:
	mkdir -p $(GNOME_EXT_DIR)/elgato-ring-light@example.com
	cp -r gnome-extension/elgato-ring-light@example.com/* $(GNOME_EXT_DIR)/elgato-ring-light@example.com/
	@echo "Extension installed. Please restart GNOME Shell (Alt+F2, then 'r') and enable it in Extensions app"

uninstall-extension:
	rm -rf $(GNOME_EXT_DIR)/elgato-ring-light@example.com

enable-service:
	systemctl --user daemon-reload
	systemctl --user enable elgato-ring-light.service
	systemctl --user start elgato-ring-light.service

disable-service:
	systemctl --user stop elgato-ring-light.service
	systemctl --user disable elgato-ring-light.service

clean:
	cargo clean
	rm -f $(DESTDIR)$(BINDIR)/elgato-cli
	rm -f $(DESTDIR)$(BINDIR)/elgato-dbus-service