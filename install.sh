#!/bin/bash
set -euo pipefail

echo "Installing Holikeyz Ring Light Controller..."

if ! command -v gnome-shell &> /dev/null; then
    echo "Error: GNOME Shell is not installed" >&2
    exit 1
fi

echo "Building Rust components..."
cargo build --release --bin holikeyz-service --bin holikeyz-cli

echo "Stopping existing service if running..."
systemctl --user stop holikeyz-ring-light.service 2>/dev/null || true

echo "Installing binaries..."
sudo install -Dm755 target/release/holikeyz-service /usr/local/bin/holikeyz-service
sudo install -Dm755 target/release/holikeyz-cli /usr/local/bin/holikeyz-cli

echo "Installing D-Bus service file..."
sudo install -Dm644 dbus/com.holikeyz.RingLight.service \
    /usr/share/dbus-1/services/com.holikeyz.RingLight.service

echo "Installing systemd user service..."
mkdir -p ~/.config/systemd/user

read -p "Enter your Ring Light IP address [192.168.7.80]: " ip_address
ip_address=${ip_address:-192.168.7.80}

# Validate IPv4 format to prevent injection into the systemd unit file
if ! [[ "$ip_address" =~ ^([0-9]{1,3}\.){3}[0-9]{1,3}$ ]]; then
    echo "Error: '$ip_address' is not a valid IPv4 address" >&2
    exit 1
fi
IFS='.' read -r a b c d <<< "$ip_address"
for octet in "$a" "$b" "$c" "$d"; do
    if (( octet > 255 )); then
        echo "Error: '$ip_address' is not a valid IPv4 address" >&2
        exit 1
    fi
done

tee ~/.config/systemd/user/holikeyz-ring-light.service > /dev/null <<EOF
[Unit]
Description=Holikeyz Ring Light D-Bus Service
After=graphical-session.target

[Service]
Type=simple
Environment="RING_LIGHT_IP=${ip_address}"
Environment="RING_LIGHT_PORT=9123"
Environment="RUST_LOG=info"
ExecStart=/usr/local/bin/holikeyz-service
Restart=always
RestartSec=5

[Install]
WantedBy=default.target
EOF

echo "Installing GNOME Shell extension..."
ext_dir=~/.local/share/gnome-shell/extensions/holikeyz-ring-light@example.com
mkdir -p "$ext_dir"
cp gnome-extension/holikeyz-ring-light@example.com/metadata.json "$ext_dir/"
cp gnome-extension/holikeyz-ring-light@example.com/extension.js "$ext_dir/"
cp gnome-extension/holikeyz-ring-light@example.com/stylesheet.css "$ext_dir/" 2>/dev/null || true
cp -r gnome-extension/holikeyz-ring-light@example.com/images "$ext_dir/" 2>/dev/null || true

echo "Starting D-Bus service..."
systemctl --user daemon-reload
systemctl --user enable holikeyz-ring-light.service
systemctl --user restart holikeyz-ring-light.service

cat <<'MSG'

Installation complete.

Next steps:
  1. Restart GNOME Shell (Alt+F2, type 'r', press Enter) — or log out and back in on Wayland.
  2. Enable the extension:
       gnome-extensions enable holikeyz-ring-light@example.com

Service management:
  systemctl --user status holikeyz-ring-light.service
  journalctl --user -u holikeyz-ring-light.service -f
MSG
