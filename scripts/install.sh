#!/bin/bash
# Install script for Pleb Signer

set -e

echo "Building Pleb Signer..."
cargo build --release

echo "Installing binary..."
sudo install -Dm755 target/release/pleb-signer /usr/local/bin/pleb-signer

echo "Installing desktop file..."
sudo install -Dm644 assets/pleb-signer.desktop /usr/share/applications/pleb-signer.desktop

echo "Installing D-Bus service file..."
mkdir -p ~/.local/share/dbus-1/services/
cat > ~/.local/share/dbus-1/services/com.plebsigner.Signer.service << EOF
[D-BUS Service]
Name=com.plebsigner.Signer
Exec=/usr/local/bin/pleb-signer --dbus-activated
EOF

echo ""
echo "Installation complete!"
echo ""
echo "To start Pleb Signer, run: pleb-signer"
echo "To enable autostart, run: ./scripts/enable-autostart.sh"
