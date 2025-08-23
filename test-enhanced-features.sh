#!/bin/bash

set -e

echo "Testing Enhanced Ring Light Features"
echo "====================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# D-Bus service name
DBUS_NAME="com.holikeyz.RingLight"
DBUS_PATH="/com/holikeyz/RingLight"
DBUS_INTERFACE="com.holikeyz.RingLight.Control"

# Helper function to call D-Bus methods
call_dbus() {
    dbus-send --session --print-reply --dest=$DBUS_NAME $DBUS_PATH $DBUS_INTERFACE.$1 "${@:2}" 2>/dev/null
}

# Test function with result checking
test_feature() {
    local test_name="$1"
    local command="$2"
    
    echo -n "Testing $test_name... "
    if eval "$command" > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
        return 0
    else
        echo -e "${RED}✗${NC}"
        return 1
    fi
}

echo "1. Testing Basic Controls"
echo "--------------------------"
test_feature "Turn On" "call_dbus TurnOn"
sleep 1
test_feature "Turn Off" "call_dbus TurnOff"
sleep 1
test_feature "Toggle" "call_dbus Toggle"
sleep 1

echo ""
echo "2. Testing Brightness & Temperature"
echo "------------------------------------"
test_feature "Set Brightness 75%" "call_dbus SetBrightness byte:75"
sleep 1
test_feature "Set Temperature 5000K" "call_dbus SetTemperature uint32:5000"
sleep 1

echo ""
echo "3. Testing Status Queries"
echo "-------------------------"
echo -n "Getting status... "
if STATUS=$(call_dbus GetStatus); then
    echo -e "${GREEN}✓${NC}"
    echo "  Status: $STATUS"
else
    echo -e "${RED}✗${NC}"
fi

echo ""
echo "4. Testing Multi-Light Support"
echo "------------------------------"
echo -n "Getting number of lights... "
if NUM_LIGHTS=$(call_dbus GetNumLights); then
    echo -e "${GREEN}✓${NC}"
    # Extract the number from the response
    NUM=$(echo "$NUM_LIGHTS" | grep -oP 'byte \K\d+' | head -1)
    echo "  Number of lights: ${NUM:-1}"
    
    if [ "${NUM:-1}" -gt 1 ]; then
        echo "  Testing individual light control..."
        test_feature "  Turn on light 0" "call_dbus TurnOnLight byte:0"
        sleep 1
        test_feature "  Turn off light 0" "call_dbus TurnOffLight byte:0"
        sleep 1
        test_feature "  Set brightness light 0" "call_dbus SetBrightnessLight byte:80 byte:0"
        sleep 1
    fi
else
    echo -e "${RED}✗${NC}"
fi

echo ""
echo "5. Testing Accessory Information"
echo "--------------------------------"
echo -n "Getting accessory info... "
if INFO=$(call_dbus GetAccessoryInfo); then
    echo -e "${GREEN}✓${NC}"
    # Parse and display info
    PRODUCT=$(echo "$INFO" | grep -oP 'string "\K[^"]+' | sed -n '1p')
    FIRMWARE=$(echo "$INFO" | grep -oP 'string "\K[^"]+' | sed -n '2p')
    SERIAL=$(echo "$INFO" | grep -oP 'string "\K[^"]+' | sed -n '3p')
    echo "  Product: ${PRODUCT:-Unknown}"
    echo "  Firmware: ${FIRMWARE:-Unknown}"
    echo "  Serial: ${SERIAL:-Unknown}"
else
    echo -e "${RED}✗${NC}"
fi

echo ""
echo "6. Testing Settings Management"
echo "------------------------------"
echo -n "Getting settings... "
if SETTINGS=$(call_dbus GetSettings); then
    echo -e "${GREEN}✓${NC}"
    echo "  Current settings retrieved"
    
    # Test setting power-on behavior
    test_feature "Set power-on settings" "call_dbus SetPowerOnSettings byte:1 byte:50 uint32:4500"
else
    echo -e "${RED}✗${NC}"
fi

echo ""
echo "7. Testing Scene Presets"
echo "------------------------"
SCENES=("daylight" "warm" "cool" "reading" "video" "relax")
for scene in "${SCENES[@]}"; do
    test_feature "Apply '$scene' scene" "call_dbus ApplyScene string:$scene"
    sleep 1
done

echo ""
echo "8. Testing Identify Function"
echo "----------------------------"
test_feature "Identify light (flash)" "call_dbus Identify"

echo ""
echo "====================================="
echo "Enhanced Features Test Complete!"
echo ""

# Count successes and failures
TOTAL_TESTS=20  # Approximate
echo "Note: Some features may not be available depending on your light model."
echo "Check the D-Bus service logs for more details:"
echo "  journalctl --user -u holikeyz-ring-light.service -f"