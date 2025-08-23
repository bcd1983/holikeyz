#!/bin/bash

# Enhanced Elgato Ring Light Controller Test Script

BINARY="./target/release/elgato-enhanced"

# Build if not exists
if [ ! -f "$BINARY" ]; then
    echo "Building enhanced binary..."
    cargo build --release --bin elgato-enhanced
fi

echo "🚀 Elgato Enhanced Controller Test Suite"
echo "========================================"
echo ""

# Function to pause between tests
pause() {
    echo ""
    read -p "Press Enter to continue..."
    echo ""
}

# 1. Discovery test
echo "Test 1: Discovering devices"
echo "----------------------------"
$BINARY discover --all
pause

# 2. Credential management
echo "Test 2: Credential Management"
echo "-----------------------------"
echo "Listing saved networks:"
$BINARY credentials list
pause

# 3. Interactive setup
echo "Test 3: Interactive Setup (with saved credentials)"
echo "---------------------------------------------------"
echo "This will attempt to use saved credentials, but prompt if not found"
$BINARY setup --use-saved
pause

# 4. Quick actions
echo "Test 4: Quick Actions"
echo "----------------------"
echo "Available quick actions:"
echo "  - all-on: Turn all lights on"
echo "  - all-off: Turn all lights off"
echo "  - all-scene <scene>: Apply scene to all lights"
echo "  - flash: Flash all lights for notification"
echo ""
read -p "Enter quick action (or skip): " action
if [ ! -z "$action" ]; then
    $BINARY quick $action
fi
pause

# 5. Control test
echo "Test 5: Device Control"
echo "----------------------"
echo "Starting interactive control mode..."
$BINARY control interactive

echo ""
echo "✅ Test suite complete!"