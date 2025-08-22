#!/bin/bash

set -e

echo "Installing Elgato Ring Light GNOME Extension..."

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
sudo cp target/release/elgato-dbus-service /usr/local/bin/
sudo cp dbus/com.elgato.RingLight.service /usr/share/dbus-1/services/

# Install systemd user service
echo "Installing systemd user service..."
mkdir -p ~/.config/systemd/user
cp systemd/elgato-ring-light.service ~/.config/systemd/user/

# Update the IP address in the service file
read -p "Enter your Elgato Ring Light IP address [192.168.7.80]: " ip_address
ip_address=${ip_address:-192.168.7.80}
sed -i "s/ELGATO_IP=.*/ELGATO_IP=$ip_address/" ~/.config/systemd/user/elgato-ring-light.service

# Install GNOME extension
echo "Installing GNOME Shell extension..."
mkdir -p ~/.local/share/gnome-shell/extensions/elgato-ring-light@example.com
cp -r gnome-extension/elgato-ring-light@example.com/* ~/.local/share/gnome-shell/extensions/elgato-ring-light@example.com/

# Enable and start the D-Bus service
echo "Starting D-Bus service..."
systemctl --user daemon-reload
systemctl --user enable elgato-ring-light.service
systemctl --user start elgato-ring-light.service

echo ""
echo "Installation complete!"
echo ""
echo "Next steps:"
echo "1. Restart GNOME Shell (Alt+F2, type 'r', press Enter)"
echo "2. Enable the extension using GNOME Extensions app or run:"
echo "   gnome-extensions enable elgato-ring-light@example.com"
echo ""
echo "The extension icon should appear in your top panel."
echo ""
echo "To test the CLI directly, run:"
echo "   elgato-cli status"