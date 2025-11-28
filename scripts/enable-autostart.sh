#!/bin/bash
# Enable autostart for Pleb Signer

mkdir -p ~/.config/autostart

cat > ~/.config/autostart/pleb-signer.desktop << EOF
[Desktop Entry]
Type=Application
Name=Pleb Signer
Exec=/usr/local/bin/pleb-signer --minimized
Icon=security-high
Comment=Nostr Signer
Categories=Utility;Security;
X-GNOME-Autostart-enabled=true
StartupNotify=false
EOF

echo "Autostart enabled! Pleb Signer will start automatically on login."
