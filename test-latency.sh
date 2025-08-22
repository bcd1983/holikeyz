#!/bin/bash

echo "Testing Elgato Ring Light latency..."
IP="192.168.7.80"
PORT="9123"

# Test 1: Direct HTTP request
echo "1. Direct HTTP request to set brightness to 50%:"
time curl -X PUT "http://$IP:$PORT/elgato/lights" \
  -H "Content-Type: application/json" \
  -d '{"numberOfLights":1,"lights":[{"on":1,"brightness":50,"temperature":213}]}' \
  --max-time 2 --silent --output /dev/null

echo ""
echo "2. Direct HTTP request to set brightness to 75%:"
time curl -X PUT "http://$IP:$PORT/elgato/lights" \
  -H "Content-Type: application/json" \
  -d '{"numberOfLights":1,"lights":[{"on":1,"brightness":75,"temperature":213}]}' \
  --max-time 2 --silent --output /dev/null

echo ""
echo "3. Testing rapid successive requests (5 requests):"
for i in {30..50..5}; do
  echo -n "  Setting brightness to $i%... "
  start=$(date +%s%N)
  curl -X PUT "http://$IP:$PORT/elgato/lights" \
    -H "Content-Type: application/json" \
    -d "{\"numberOfLights\":1,\"lights\":[{\"on\":1,\"brightness\":$i,\"temperature\":213}]}" \
    --max-time 1 --silent --output /dev/null
  end=$(date +%s%N)
  elapsed=$((($end - $start) / 1000000))
  echo "${elapsed}ms"
done