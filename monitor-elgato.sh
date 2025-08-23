#!/bin/bash

echo "Monitoring for Elgato Ring Light setup network..."
echo "================================================"
echo ""
echo "Instructions:"
echo "1. Press and hold the button on your Elgato Ring Light for 10-15 seconds"
echo "2. The light should blink/flash to indicate setup mode"
echo "3. This script will detect when the WiFi network appears"
echo ""
echo "Press Ctrl+C to stop monitoring"
echo ""

while true; do
    # Rescan WiFi networks
    nmcli dev wifi rescan 2>/dev/null
    
    # Check for Elgato network
    ELGATO=$(nmcli -t -f SSID,SIGNAL dev wifi list 2>/dev/null | grep "Elgato Ring Light")
    
    if [ -n "$ELGATO" ]; then
        echo ""
        echo "🎉 FOUND ELGATO DEVICE!"
        echo "$ELGATO"
        echo ""
        echo "To connect:"
        SSID=$(echo "$ELGATO" | cut -d: -f1)
        echo "nmcli device wifi connect '$SSID'"
        echo ""
        echo "Then run: ./provision-elgato.sh"
        break
    else
        echo -n "."
    fi
    
    sleep 2
done