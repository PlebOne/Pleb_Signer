#!/bin/bash
set -e

# Build Pleb Signer packages (.deb and .rpm)
# Run from the project root directory

VERSION="0.1.1"
PACKAGE_NAME="pleb-signer"
ARCH="amd64"

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_DIR="$PROJECT_ROOT/build"
RELEASE_DIR="$PROJECT_ROOT/release"

echo "=== Building Pleb Signer v${VERSION} packages ==="

# Clean previous builds
rm -rf "$BUILD_DIR" "$RELEASE_DIR"
mkdir -p "$BUILD_DIR" "$RELEASE_DIR"

# Ensure release binary exists
if [ ! -f "$PROJECT_ROOT/target/release/pleb-signer" ]; then
    echo "Building release binary..."
    cd "$PROJECT_ROOT"
    cargo build --release
fi

BINARY="$PROJECT_ROOT/target/release/pleb-signer"
BINARY_SIZE=$(du -h "$BINARY" | cut -f1)
echo "Binary size: $BINARY_SIZE"

# ============================================
# Build .deb package
# ============================================
echo ""
echo "=== Building .deb package ==="

DEB_ROOT="$BUILD_DIR/deb/${PACKAGE_NAME}_${VERSION}_${ARCH}"
mkdir -p "$DEB_ROOT/DEBIAN"
mkdir -p "$DEB_ROOT/usr/bin"
mkdir -p "$DEB_ROOT/usr/share/applications"
mkdir -p "$DEB_ROOT/usr/share/dbus-1/services"
mkdir -p "$DEB_ROOT/usr/share/doc/${PACKAGE_NAME}"

# Copy binary
cp "$BINARY" "$DEB_ROOT/usr/bin/pleb-signer"
chmod 755 "$DEB_ROOT/usr/bin/pleb-signer"

# Copy desktop file
cp "$PROJECT_ROOT/assets/pleb-signer.desktop" "$DEB_ROOT/usr/share/applications/"

# Copy D-Bus service file
cp "$PROJECT_ROOT/assets/com.plebsigner.Signer.service" "$DEB_ROOT/usr/share/dbus-1/services/"

# Copy documentation
cp "$PROJECT_ROOT/README.md" "$DEB_ROOT/usr/share/doc/${PACKAGE_NAME}/"
cp "$PROJECT_ROOT/LICENSE" "$DEB_ROOT/usr/share/doc/${PACKAGE_NAME}/"
cp "$PROJECT_ROOT/docs/CLIENT_INTEGRATION.md" "$DEB_ROOT/usr/share/doc/${PACKAGE_NAME}/"

# Calculate installed size
INSTALLED_SIZE=$(du -s "$DEB_ROOT" | cut -f1)

# Create control file
cat > "$DEB_ROOT/DEBIAN/control" << EOF
Package: ${PACKAGE_NAME}
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: ${ARCH}
Installed-Size: ${INSTALLED_SIZE}
Maintainer: PlebOne <plebone@protonmail.com>
Depends: libc6, libdbus-1-3
Homepage: https://github.com/PlebOne/Pleb_Signer
Description: Linux desktop Nostr signer with D-Bus interface
 Pleb Signer is a NIP-55 compatible Nostr signer for Linux,
 similar to Amber for Android. It stores your private keys
 securely in the OS keyring and provides a D-Bus interface
 for other applications to request signatures.
 .
 Features:
  - Secure key storage in OS keyring (GNOME Keyring/KWallet)
  - System tray integration
  - D-Bus API for application integration
  - NIP-04 and NIP-44 encryption support
  - NIP-46 Bunker mode for remote signing
EOF

# Copy maintainer scripts
cp "$PROJECT_ROOT/packaging/debian/postinst" "$DEB_ROOT/DEBIAN/"
cp "$PROJECT_ROOT/packaging/debian/postrm" "$DEB_ROOT/DEBIAN/"
chmod 755 "$DEB_ROOT/DEBIAN/postinst"
chmod 755 "$DEB_ROOT/DEBIAN/postrm"

# Build the .deb
DEB_FILE="${PACKAGE_NAME}_${VERSION}_${ARCH}.deb"
dpkg-deb --build --root-owner-group "$DEB_ROOT" "$RELEASE_DIR/$DEB_FILE"
echo "Created: $RELEASE_DIR/$DEB_FILE"

# ============================================
# Build .rpm package using alien (if available)
# ============================================
echo ""
echo "=== Building .rpm package ==="

if command -v alien &> /dev/null; then
    cd "$RELEASE_DIR"
    # Convert .deb to .rpm
    sudo alien --to-rpm --scripts "$DEB_FILE"
    # Rename to consistent format
    RPM_FILE=$(ls *.rpm 2>/dev/null | head -1)
    if [ -n "$RPM_FILE" ]; then
        NEW_RPM="${PACKAGE_NAME}-${VERSION}-1.x86_64.rpm"
        if [ "$RPM_FILE" != "$NEW_RPM" ]; then
            mv "$RPM_FILE" "$NEW_RPM"
        fi
        echo "Created: $RELEASE_DIR/$NEW_RPM"
    fi
else
    echo "Warning: 'alien' not found. Skipping .rpm generation."
    echo "Install with: sudo apt install alien"
    
    # Create a tarball instead for manual RPM building
    echo "Creating tarball for manual RPM building..."
    TARBALL="${PACKAGE_NAME}-${VERSION}.tar.gz"
    mkdir -p "$BUILD_DIR/tarball/${PACKAGE_NAME}-${VERSION}"
    cp "$BINARY" "$BUILD_DIR/tarball/${PACKAGE_NAME}-${VERSION}/"
    cp "$PROJECT_ROOT/assets/pleb-signer.desktop" "$BUILD_DIR/tarball/${PACKAGE_NAME}-${VERSION}/"
    cp "$PROJECT_ROOT/assets/com.plebsigner.Signer.service" "$BUILD_DIR/tarball/${PACKAGE_NAME}-${VERSION}/"
    cp "$PROJECT_ROOT/README.md" "$BUILD_DIR/tarball/${PACKAGE_NAME}-${VERSION}/"
    cp "$PROJECT_ROOT/LICENSE" "$BUILD_DIR/tarball/${PACKAGE_NAME}-${VERSION}/"
    cp "$PROJECT_ROOT/packaging/pleb-signer.spec" "$BUILD_DIR/tarball/${PACKAGE_NAME}-${VERSION}/"
    
    cd "$BUILD_DIR/tarball"
    tar czf "$RELEASE_DIR/$TARBALL" "${PACKAGE_NAME}-${VERSION}"
    echo "Created: $RELEASE_DIR/$TARBALL"
fi

# ============================================
# Create checksums
# ============================================
echo ""
echo "=== Creating checksums ==="
cd "$RELEASE_DIR"
sha256sum * > SHA256SUMS.txt
cat SHA256SUMS.txt

echo ""
echo "=== Build complete ==="
echo "Packages are in: $RELEASE_DIR"
ls -lh "$RELEASE_DIR"
