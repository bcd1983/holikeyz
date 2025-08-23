#!/bin/bash

# Safe provisioning script for Elgato Ring Light
# Handles API failures with web interface fallback

echo "🔒 Safe Elgato Provisioning Script"
echo "==================================="
echo ""
echo "This script will:"
echo "1. Try automatic provisioning via encrypted API"
echo "2. Fall back to web interface if API fails"
echo "3. Save credentials securely for future use"
echo ""

# Check for required tools
command -v nmcli >/dev/null 2>&1 || { echo "❌ nmcli is required but not installed."; exit 1; }

# Build if needed
if [ ! -f "./target/release/elgato-enhanced" ]; then
    echo "Building enhanced controller..."
    cargo build --release --bin elgato-enhanced || exit 1
fi

echo "📡 Starting device setup..."
echo ""

# Run the enhanced setup with saved credentials
# This will now handle failures gracefully
./target/release/elgato-enhanced setup --use-saved

echo ""
echo "✅ Provisioning process complete!"
echo ""
echo "If the device didn't connect automatically, you can:"
echo "1. Try again with: ./target/release/elgato-enhanced setup"
echo "2. Use the web interface at http://192.168.62.1:9123"
echo "3. Use the Elgato Control Center mobile app"