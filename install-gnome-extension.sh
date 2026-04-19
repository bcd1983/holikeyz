#!/bin/bash

set -e

echo "Installing Holikeyz Ring Light GNOME Extension..."

# Check if GNOME Shell is running
if ! command -v gnome-shell &> /dev/null; then
    echo "Error: GNOME Shell is not installed"
    exit 1
fi

# Build the Rust components first
echo "Building Rust components..."
cargo build --release

# Install the D-Bus service
echo "Installing D-Bus service..."
sudo cp target/release/holikeyz-service /usr/local/bin/
sudo cp dbus/com.holikeyz.RingLight.service /usr/share/dbus-1/services/

# Install systemd user service
echo "Installing systemd user service..."
mkdir -p ~/.config/systemd/user
cp systemd/holikeyz-ring-light.service ~/.config/systemd/user/

# Update the IP address in the service file
read -p "Enter your Ring Light IP address [192.168.7.80]: " ip_address
ip_address=${ip_address:-192.168.7.80}

# Validate IPv4 format before substitution to prevent sed injection
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

sed -i "s|RING_LIGHT_IP=.*|RING_LIGHT_IP=${ip_address}|" ~/.config/systemd/user/holikeyz-ring-light.service

# Install GNOME extension
echo "Installing GNOME Shell extension..."
mkdir -p ~/.local/share/gnome-shell/extensions/holikeyz-ring-light@example.com
cp -r gnome-extension/holikeyz-ring-light@example.com/* ~/.local/share/gnome-shell/extensions/holikeyz-ring-light@example.com/

# Enable and start the D-Bus service
echo "Starting D-Bus service..."
systemctl --user daemon-reload
systemctl --user enable holikeyz-ring-light.service
systemctl --user start holikeyz-ring-light.service

echo ""
echo "Installation complete!"
echo ""
echo "Next steps:"
echo "1. Restart GNOME Shell (Alt+F2, type 'r', press Enter)"
echo "2. Enable the extension using GNOME Extensions app or run:"
echo "   gnome-extensions enable holikeyz-ring-light@example.com"
echo ""
echo "The extension icon should appear in your top panel."
echo ""
echo "To test the CLI directly, run:"
echo "   holikeyz-cli status"