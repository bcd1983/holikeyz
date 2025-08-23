#!/bin/bash

set -e

echo "Installing Enhanced Holikeyz Ring Light Controller..."
echo "======================================================"
echo ""

# Check if GNOME Shell is running
if ! command -v gnome-shell &> /dev/null; then
    echo "Error: GNOME Shell is not installed"
    exit 1
fi

# Build the Rust components first
echo "Building enhanced Rust components..."
cargo build --release --bin holikeyz-service-enhanced
cargo build --release --bin holikeyz-cli

# Stop existing service if running
echo "Stopping existing service if running..."
systemctl --user stop holikeyz-ring-light.service 2>/dev/null || true

# Install the enhanced D-Bus service
echo "Installing enhanced D-Bus service..."
sudo cp target/release/holikeyz-service-enhanced /usr/local/bin/holikeyz-service
sudo cp target/release/holikeyz-cli /usr/local/bin/

# Create D-Bus service file
echo "Installing D-Bus service file..."
sudo tee /usr/share/dbus-1/services/com.holikeyz.RingLight.service > /dev/null <<EOF
[D-BUS Service]
Name=com.holikeyz.RingLight
Exec=/usr/local/bin/holikeyz-service
EOF

# Install systemd user service
echo "Installing systemd user service..."
mkdir -p ~/.config/systemd/user

# Update the IP address in the service file
read -p "Enter your Ring Light IP address [192.168.7.80]: " ip_address
ip_address=${ip_address:-192.168.7.80}

tee ~/.config/systemd/user/holikeyz-ring-light.service > /dev/null <<EOF
[Unit]
Description=Holikeyz Ring Light D-Bus Service (Enhanced)
After=graphical-session.target

[Service]
Type=simple
Environment="RING_LIGHT_IP=$ip_address"
Environment="RING_LIGHT_PORT=9123"
Environment="RUST_LOG=info"
ExecStart=/usr/local/bin/holikeyz-service
Restart=always
RestartSec=5

[Install]
WantedBy=default.target
EOF

# Install enhanced GNOME extension
echo "Installing enhanced GNOME Shell extension..."
mkdir -p ~/.local/share/gnome-shell/extensions/holikeyz-ring-light@example.com

# Copy the enhanced extension
cp gnome-extension/holikeyz-ring-light@example.com/metadata.json ~/.local/share/gnome-shell/extensions/holikeyz-ring-light@example.com/
cp gnome-extension/holikeyz-ring-light@example.com/extension-enhanced.js ~/.local/share/gnome-shell/extensions/holikeyz-ring-light@example.com/extension.js
cp gnome-extension/holikeyz-ring-light@example.com/stylesheet.css ~/.local/share/gnome-shell/extensions/holikeyz-ring-light@example.com/ 2>/dev/null || true

# Enable and start the D-Bus service
echo "Starting enhanced D-Bus service..."
systemctl --user daemon-reload
systemctl --user enable holikeyz-ring-light.service
systemctl --user restart holikeyz-ring-light.service

echo ""
echo "======================================================"
echo "Enhanced Installation complete!"
echo ""
echo "New features available:"
echo "  ✓ Multiple light control support"
echo "  ✓ Device information access"
echo "  ✓ Power-on behavior settings"
echo "  ✓ Advanced settings management"
echo "  ✓ Individual light control (if multiple lights)"
echo ""
echo "Next steps:"
echo "1. Restart GNOME Shell (Alt+F2, type 'r', press Enter)"
echo "2. Enable the extension using GNOME Extensions app or run:"
echo "   gnome-extensions enable holikeyz-ring-light@example.com"
echo ""
echo "To test the new features, run:"
echo "   ./test-enhanced-features.sh"
echo ""
echo "To check service status:"
echo "   systemctl --user status holikeyz-ring-light.service"
echo ""
echo "To view logs:"
echo "   journalctl --user -u holikeyz-ring-light.service -f"