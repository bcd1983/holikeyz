#!/bin/bash

echo "Detecting Elgato Ring Light devices..."
echo ""

# Scan for networks and find Elgato devices
ELGATO_DEVICE=$(curl -s http://localhost:9124/provisioning/scan | jq -r '.networks[] | select(.ssid | startswith("Elgato Ring Light")) | .ssid' | head -1)

if [ -n "$ELGATO_DEVICE" ]; then
    echo "✅ Found Elgato device: $ELGATO_DEVICE"
    echo ""
    echo "Signal strength and details:"
    curl -s http://localhost:9124/provisioning/scan | jq --arg device "$ELGATO_DEVICE" '.networks[] | select(.ssid == $device)'
    echo ""
    echo "To provision this device:"
    echo "1. Connect your computer to the WiFi network: '$ELGATO_DEVICE'"
    echo "2. Run: cargo run --example provisioning_client"
else
    echo "❌ No Elgato Ring Light found in setup mode"
    echo ""
    echo "To put your Elgato Ring Light in setup mode:"
    echo "1. Hold the button on the back for 10 seconds"
    echo "2. Wait for the light to blink"
    echo "3. Run this script again"
fi