#!/bin/bash

echo "Testing D-Bus service connection..."

# Test if service is running
dbus-send --session --print-reply --dest=com.elgato.RingLight \
    /com/elgato/RingLight \
    com.elgato.RingLight.Control.GetStatus

if [ $? -eq 0 ]; then
    echo "D-Bus service is working!"
else
    echo "D-Bus service is not responding"
fi