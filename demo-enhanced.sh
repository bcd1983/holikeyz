#!/bin/bash

# Demo script for Enhanced Elgato Ring Light Controller
# This demonstrates the key improvements:
# 1. Automatic credential prompting when not found
# 2. Secure storage using OS keyring
# 3. Multi-device support

echo "🚀 Enhanced Elgato Ring Light Controller Demo"
echo "============================================="
echo ""
echo "Key Features:"
echo "✅ Automatic credential prompting if not saved"
echo "✅ Secure credential storage in OS keyring"
echo "✅ Support for multiple devices"
echo "✅ Network auto-restoration after setup"
echo ""

# Check if binary exists
if [ ! -f "./target/release/elgato-enhanced" ]; then
    echo "Building the enhanced controller..."
    cargo build --release --bin elgato-enhanced
fi

echo "Starting interactive setup with automatic credential handling..."
echo ""
echo "The system will:"
echo "1. Try to find saved credentials for your WiFi"
echo "2. If not found, prompt you to enter them"
echo "3. Offer to save them securely for next time"
echo ""

./target/release/elgato-enhanced setup --use-saved