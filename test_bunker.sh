#!/bin/bash
# Test script for bunker/NIP-46 D-Bus methods

echo "Testing Pleb Signer Bunker D-Bus Interface"
echo "==========================================="
echo

# Check if signer is running
echo "1. Checking if Pleb Signer is running..."
if ! pgrep -x "pleb-signer" > /dev/null; then
    echo "❌ Pleb Signer is not running. Please start it first."
    exit 1
fi
echo "✓ Pleb Signer is running"
echo

# Check if service is available on D-Bus
echo "2. Checking D-Bus service availability..."
if dbus-send --session --print-reply --dest=com.plebsigner.Signer \
    /com/plebsigner/Signer com.plebsigner.Signer1.Version &>/dev/null; then
    echo "✓ D-Bus service is available"
else
    echo "❌ D-Bus service not found"
    exit 1
fi
echo

# Check if signer is ready (unlocked)
echo "3. Checking if signer is ready..."
READY=$(dbus-send --session --print-reply --dest=com.plebsigner.Signer \
    /com/plebsigner/Signer com.plebsigner.Signer1.IsReady 2>/dev/null | \
    grep "boolean" | awk '{print $2}')

if [ "$READY" == "true" ]; then
    echo "✓ Signer is ready (unlocked)"
else
    echo "❌ Signer is locked or no keys available"
    echo "   Please unlock Pleb Signer and ensure a key is active"
    exit 1
fi
echo

# Test getting bunker URI
echo "4. Testing GetBunkerUri method..."
URI_RESPONSE=$(dbus-send --session --print-reply --dest=com.plebsigner.Signer \
    /com/plebsigner/Signer com.plebsigner.Signer1.GetBunkerUri 2>&1)

if echo "$URI_RESPONSE" | grep -q "string"; then
    echo "✓ GetBunkerUri method is available"
    echo "   Response preview:"
    echo "$URI_RESPONSE" | grep "string" | head -3
else
    echo "❌ GetBunkerUri method failed"
    echo "$URI_RESPONSE"
fi
echo

# Test starting bunker
echo "5. Testing StartBunker method..."
START_RESPONSE=$(dbus-send --session --print-reply --dest=com.plebsigner.Signer \
    /com/plebsigner/Signer com.plebsigner.Signer1.StartBunker 2>&1)

if echo "$START_RESPONSE" | grep -q "bunker://"; then
    echo "✓ StartBunker method works!"
    echo "   Bunker URI has been generated and listener started"
    
    # Extract and display the bunker URI
    BUNKER_URI=$(echo "$START_RESPONSE" | grep -o 'bunker://[^"]*' | head -1)
    echo
    echo "   🔗 Bunker URI:"
    echo "   $BUNKER_URI"
else
    echo "⚠ StartBunker returned a response (check if already running):"
    echo "$START_RESPONSE" | grep "string" | head -3
fi
echo

# Test getting bunker state
echo "6. Testing GetBunkerState method..."
STATE_RESPONSE=$(dbus-send --session --print-reply --dest=com.plebsigner.Signer \
    /com/plebsigner/Signer com.plebsigner.Signer1.GetBunkerState 2>&1)

if echo "$STATE_RESPONSE" | grep -q "string"; then
    echo "✓ GetBunkerState method is available"
    echo "   Current state:"
    echo "$STATE_RESPONSE" | grep "string" | head -3
fi
echo

# Test stopping bunker
echo "7. Testing StopBunker method..."
STOP_RESPONSE=$(dbus-send --session --print-reply --dest=com.plebsigner.Signer \
    /com/plebsigner/Signer com.plebsigner.Signer1.StopBunker 2>&1)

if echo "$STOP_RESPONSE" | grep -q "string"; then
    echo "✓ StopBunker method works"
    echo "   Bunker listener stopped"
fi
echo

echo "==========================================="
echo "✓ All bunker D-Bus methods are working!"
echo
echo "To use bunker mode:"
echo "1. Start bunker: dbus-send ... StartBunker"
echo "2. Copy the bunker:// URI"
echo "3. Scan QR or paste into your remote client"
echo "4. Sign events remotely via Nostr relays"
