#!/bin/bash

echo "Elgato Ring Light Provisioning Helper"
echo "======================================"
echo ""

# Check current network
CURRENT_NETWORK=$(nmcli -t -f NAME connection show --active | head -1)
echo "Current network: $CURRENT_NETWORK"
echo ""

# Check if Elgato network is visible
ELGATO_SSID=$(nmcli -t -f SSID dev wifi list | grep "Elgato Ring Light" | head -1)

if [ -n "$ELGATO_SSID" ]; then
    echo "✅ Found Elgato device: $ELGATO_SSID"
    echo ""
    
    if [[ "$CURRENT_NETWORK" == *"Elgato"* ]]; then
        echo "✅ Already connected to Elgato device!"
        echo ""
        echo "Testing Elgato API endpoints..."
        echo ""
        
        # Test various possible Elgato endpoints
        for ip in "192.168.4.1" "10.123.45.1" "172.16.0.1"; do
            echo "Testing $ip..."
            if ping -c 1 -W 1 $ip >/dev/null 2>&1; then
                echo "  ✅ $ip is reachable"
                
                # Test common ports
                for port in 80 9123 9090 8080; do
                    if curl -s --max-time 1 http://$ip:$port/ >/dev/null 2>&1; then
                        echo "    Port $port is open"
                        
                        # Test Elgato endpoints
                        for endpoint in "/elgato/accessory-info" "/elgato/lights" "/info" "/api/info"; do
                            response=$(curl -s --max-time 1 http://$ip:$port$endpoint 2>/dev/null)
                            if [ -n "$response" ] && [[ "$response" != *"404"* ]] && [[ "$response" != *"html"* ]]; then
                                echo "    ✅ Found API at http://$ip:$port$endpoint"
                                echo "    Response: $response" | head -c 200
                                echo ""
                            fi
                        done
                    fi
                done
            else
                echo "  ❌ $ip not reachable"
            fi
        done
        
    else
        echo "❌ Not connected to Elgato device"
        echo ""
        echo "To connect to the Elgato device:"
        echo "1. Run: nmcli device wifi connect '$ELGATO_SSID'"
        echo "2. Run this script again"
        echo ""
        echo "Note: The Elgato network is typically open (no password)"
    fi
else
    echo "❌ No Elgato Ring Light found in setup mode"
    echo ""
    echo "To put your Elgato Ring Light in setup mode:"
    echo "1. Press and hold the control button on the back for 10 seconds"
    echo "2. The light will blink to indicate setup mode"
    echo "3. Wait a few seconds for the WiFi network to appear"
    echo "4. Run this script again"
fi

echo ""
echo "Alternative: Manual connection"
echo "1. Open WiFi settings"
echo "2. Connect to 'Elgato Ring Light ADD0'"
echo "3. Run this script again to test the API"