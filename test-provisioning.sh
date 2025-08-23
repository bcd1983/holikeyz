#!/bin/bash

echo "Elgato Ring Light Provisioning Test"
echo "===================================="
echo ""

# Check if running as root (needed for network operations)
if [ "$EUID" -ne 0 ]; then 
    echo "Warning: Some provisioning features require root access for network configuration"
    echo "You may need to run: sudo $0"
    echo ""
fi

# Build the project
echo "Building the provisioning service..."
cargo build --bin holikeyz-provisioning --release

echo ""
echo "Available commands:"
echo "1. Start provisioning server:"
echo "   cargo run --bin holikeyz-provisioning -- server --port 9124"
echo ""
echo "2. Scan WiFi networks:"
echo "   cargo run --bin holikeyz-provisioning -- scan"
echo ""
echo "3. Discover devices on network:"
echo "   cargo run --bin holikeyz-provisioning -- discover --timeout 30"
echo ""
echo "4. Provision an Elgato device:"
echo "   cargo run --bin holikeyz-provisioning -- provision \\"
echo "       --device-type elgato \\"
echo "       --target-ssid 'YourWiFi' \\"
echo "       --target-password 'YourPassword'"
echo ""
echo "5. Run interactive example client:"
echo "   cargo run --example provisioning_client"
echo ""

# Test discovery functionality
echo "Testing device discovery (5 seconds)..."
cargo run --bin holikeyz-provisioning -- discover --timeout 5

echo ""
echo "Test complete. Use the commands above to provision your Elgato Ring Light."