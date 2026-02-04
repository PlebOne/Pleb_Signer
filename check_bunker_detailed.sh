#!/bin/bash

echo "=== Detailed Bunker Status Check ==="
echo

# 1. Check if pleb-signer is running
echo "1. Process status:"
pgrep -a pleb-signer
echo

# 2. Get bunker state
echo "2. Bunker state:"
dbus-send --session --print-reply --dest=com.plebsigner.Signer \
  /com/plebsigner/Signer com.plebsigner.Signer1.GetBunkerState 2>&1 | grep "string"
echo

# 3. Try starting bunker
echo "3. Starting bunker..."
RESPONSE=$(dbus-send --session --print-reply --dest=com.plebsigner.Signer \
  /com/plebsigner/Signer com.plebsigner.Signer1.StartBunker 2>&1)
echo "$RESPONSE" | grep "string"
echo

# 4. Wait a bit
echo "4. Waiting 3 seconds..."
sleep 3

# 5. Check state again
echo "5. Bunker state after start:"
dbus-send --session --print-reply --dest=com.plebsigner.Signer \
  /com/plebsigner/Signer com.plebsigner.Signer1.GetBunkerState 2>&1 | grep "string"
echo

# 6. Extract and show bunker URI
echo "6. Bunker URI from response:"
echo "$RESPONSE" | grep -o 'bunker://[^"]*' | head -1
echo

echo "=== End Status Check ==="
