#!/bin/bash

# Test script for Elgato provisioning methods

echo "🔍 Elgato Provisioning Test"
echo "============================"
echo ""

# Check if connected to device
echo "First, make sure you're connected to the Elgato's WiFi network."
echo "The network name should be something like 'Elgato Ring Light XXXX'"
echo ""

# Build if needed
if [ ! -f "./target/release/elgato-debug" ]; then
    echo "Building debug tool..."
    cargo build --release --bin elgato-debug
fi

# Step 1: Check endpoints
echo "Step 1: Checking device endpoints..."
echo "-------------------------------------"
./target/release/elgato-debug check
echo ""

# Step 2: Get device info
echo "Step 2: Getting device information..."
echo "-------------------------------------"
./target/release/elgato-debug info
echo ""

# Step 3: Get WiFi info
echo "Step 3: Getting current WiFi info..."
echo "------------------------------------"
./target/release/elgato-debug wifi
echo ""

# Step 4: Test provisioning
echo "Step 4: Test provisioning methods"
echo "---------------------------------"
echo "Enter the target WiFi network details:"
read -p "SSID: " ssid
read -sp "Password: " password
echo ""
echo ""

echo "Testing all provisioning methods..."
./target/release/elgato-debug test-all "$ssid" "$password"

echo ""
echo "✅ Test complete!"
echo ""
echo "If any method succeeded, the device should reboot and connect to your network."
echo "You can verify by running: ./target/release/elgato-enhanced discover --all"