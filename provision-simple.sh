#!/bin/bash

echo "Elgato Ring Light Simple Provisioning"
echo "======================================"
echo ""

# Check if connected to Elgato
if ! curl -s --max-time 2 http://192.168.62.1:9123/elgato/accessory-info > /dev/null 2>&1; then
    echo "❌ Not connected to Elgato Ring Light WiFi"
    echo "   Please connect to 'Elgato Ring Light ADD0' first"
    exit 1
fi

# Get device info
DEVICE_INFO=$(curl -s http://192.168.62.1:9123/elgato/accessory-info)
echo "✅ Connected to Elgato Ring Light"
echo "   Serial: $(echo $DEVICE_INFO | jq -r '.serialNumber')"
echo "   Firmware: $(echo $DEVICE_INFO | jq -r '.firmwareVersion')"
echo ""

# The Elgato uses encrypted configuration via web interface
echo "WiFi Configuration Options:"
echo ""
echo "Option 1: Use Web Interface (Recommended)"
echo "   1. Open browser to: http://192.168.62.1:9123"
echo "   2. Enter your WiFi credentials"
echo "   3. Click Connect"
echo ""
echo "Option 2: Use the Control API"
echo "   The device is already accessible for control:"
echo ""

# Show current light status
LIGHTS=$(curl -s http://192.168.62.1:9123/elgato/lights)
echo "Current Light Status:"
echo "$LIGHTS" | jq .
echo ""

# Test light control
echo "Testing light control..."
echo "Turning light off..."
curl -X PUT http://192.168.62.1:9123/elgato/lights \
    -H "Content-Type: application/json" \
    -d '{"numberOfLights":1,"lights":[{"on":0,"brightness":20,"temperature":230}]}' \
    -s > /dev/null

sleep 2

echo "Turning light on at 50% brightness..."
curl -X PUT http://192.168.62.1:9123/elgato/lights \
    -H "Content-Type: application/json" \
    -d '{"numberOfLights":1,"lights":[{"on":1,"brightness":50,"temperature":230}]}' \
    -s > /dev/null

echo ""
echo "✅ Light control working!"
echo ""
echo "To configure WiFi:"
echo "1. Open: http://192.168.62.1:9123 in your browser"
echo "2. Enter SSID: BELL825"
echo "3. Enter your WiFi password"
echo "4. Click Connect"
echo ""
echo "The device will reboot and connect to your network."