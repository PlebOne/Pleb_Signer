#!/bin/bash
set -e

# Build Pleb Signer packages for all distros
# Works on any Linux distro

VERSION="0.1.1"
PACKAGE_NAME="pleb-signer"

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
# Create source tarball for manual packaging
# ============================================
echo ""
echo "=== Creating source tarball ==="

TARBALL_DIR="$BUILD_DIR/tarball/${PACKAGE_NAME}-${VERSION}"
mkdir -p "$TARBALL_DIR"

# Copy files needed for all package types
cp "$BINARY" "$TARBALL_DIR/"
cp "$PROJECT_ROOT/README.md" "$TARBALL_DIR/"
cp "$PROJECT_ROOT/LICENSE" "$TARBALL_DIR/"
cp "$PROJECT_ROOT/BUNKER_IMPLEMENTATION.md" "$TARBALL_DIR/"
cp "$PROJECT_ROOT/test_bunker.sh" "$TARBALL_DIR/"
cp -r "$PROJECT_ROOT/docs" "$TARBALL_DIR/"
cp -r "$PROJECT_ROOT/assets" "$TARBALL_DIR/"
cp -r "$PROJECT_ROOT/packaging" "$TARBALL_DIR/"
cp -r "$PROJECT_ROOT/scripts" "$TARBALL_DIR/"

# Create the tarball
cd "$BUILD_DIR/tarball"
TARBALL="${PACKAGE_NAME}-${VERSION}.tar.gz"
tar czf "$RELEASE_DIR/$TARBALL" "${PACKAGE_NAME}-${VERSION}"
echo "✓ Created: $RELEASE_DIR/$TARBALL"

# ============================================
# Create binary tarball (for generic install)
# ============================================
echo ""
echo "=== Creating binary tarball ==="

BINARY_TARBALL_DIR="$BUILD_DIR/binary/${PACKAGE_NAME}-${VERSION}-linux-x86_64"
mkdir -p "$BINARY_TARBALL_DIR"

cp "$BINARY" "$BINARY_TARBALL_DIR/"
cp "$PROJECT_ROOT/README.md" "$BINARY_TARBALL_DIR/"
cp "$PROJECT_ROOT/LICENSE" "$BINARY_TARBALL_DIR/"
cp "$PROJECT_ROOT/scripts/install.sh" "$BINARY_TARBALL_DIR/"
cp "$PROJECT_ROOT/assets/pleb-signer.desktop" "$BINARY_TARBALL_DIR/"
cp "$PROJECT_ROOT/assets/com.plebsigner.Signer.service" "$BINARY_TARBALL_DIR/"

# Create install instructions
cat > "$BINARY_TARBALL_DIR/INSTALL.txt" << 'EOF'
Pleb Signer - Installation Instructions
========================================

1. Extract the archive:
   tar xzf pleb-signer-*.tar.gz
   cd pleb-signer-*

2. Install manually:
   sudo cp pleb-signer /usr/local/bin/
   sudo chmod +x /usr/local/bin/pleb-signer
   
   mkdir -p ~/.local/share/applications
   cp pleb-signer.desktop ~/.local/share/applications/
   
   mkdir -p ~/.local/share/dbus-1/services
   cp com.plebsigner.Signer.service ~/.local/share/dbus-1/services/

3. Run:
   pleb-signer

Or use the automated script:
   ./install.sh

For package manager installations, see packaging/ directory.
EOF

cd "$BUILD_DIR/binary"
BINARY_TARBALL="${PACKAGE_NAME}-${VERSION}-linux-x86_64.tar.gz"
tar czf "$RELEASE_DIR/$BINARY_TARBALL" "${PACKAGE_NAME}-${VERSION}-linux-x86_64"
echo "✓ Created: $RELEASE_DIR/$BINARY_TARBALL"

# ============================================
# Create Arch Linux PKGBUILD
# ============================================
echo ""
echo "=== Creating Arch Linux PKGBUILD ==="

ARCH_DIR="$BUILD_DIR/arch"
mkdir -p "$ARCH_DIR"

# Calculate SHA256 of the binary tarball
TARBALL_SHA256=$(sha256sum "$RELEASE_DIR/$BINARY_TARBALL" | cut -d' ' -f1)

cat > "$ARCH_DIR/PKGBUILD" << EOF
# Maintainer: PlebOne <plebone@protonmail.com>
pkgname=pleb-signer
pkgver=${VERSION}
pkgrel=1
pkgdesc="Linux desktop Nostr signer with NIP-46 bunker support"
arch=('x86_64')
url="https://github.com/PlebOne/Pleb_Signer"
license=('MIT')
depends=('dbus' 'gcc-libs')
source=("\${pkgname}-\${pkgver}.tar.gz::https://github.com/PlebOne/Pleb_Signer/releases/download/v\${pkgver}/\${pkgname}-\${pkgver}-linux-x86_64.tar.gz")
sha256sums=('${TARBALL_SHA256}')

package() {
    cd "\${srcdir}/\${pkgname}-\${pkgver}-linux-x86_64"
    
    # Install binary
    install -Dm755 pleb-signer "\${pkgdir}/usr/bin/pleb-signer"
    
    # Install desktop file
    install -Dm644 pleb-signer.desktop "\${pkgdir}/usr/share/applications/pleb-signer.desktop"
    
    # Install D-Bus service
    install -Dm644 com.plebsigner.Signer.service "\${pkgdir}/usr/share/dbus-1/services/com.plebsigner.Signer.service"
    
    # Install documentation
    install -Dm644 README.md "\${pkgdir}/usr/share/doc/\${pkgname}/README.md"
    install -Dm644 LICENSE "\${pkgdir}/usr/share/licenses/\${pkgname}/LICENSE"
}
EOF

cp "$ARCH_DIR/PKGBUILD" "$RELEASE_DIR/"
echo "✓ Created: $RELEASE_DIR/PKGBUILD"

# ============================================
# Create DEB package structure (for manual dpkg-deb)
# ============================================
echo ""
echo "=== Creating DEB package structure ==="

DEB_ROOT="$BUILD_DIR/deb/${PACKAGE_NAME}_${VERSION}_amd64"
mkdir -p "$DEB_ROOT/DEBIAN"
mkdir -p "$DEB_ROOT/usr/bin"
mkdir -p "$DEB_ROOT/usr/share/applications"
mkdir -p "$DEB_ROOT/usr/share/dbus-1/services"
mkdir -p "$DEB_ROOT/usr/share/doc/${PACKAGE_NAME}"

# Copy binary
cp "$BINARY" "$DEB_ROOT/usr/bin/pleb-signer"
chmod 755 "$DEB_ROOT/usr/bin/pleb-signer"

# Copy files
cp "$PROJECT_ROOT/assets/pleb-signer.desktop" "$DEB_ROOT/usr/share/applications/"
cp "$PROJECT_ROOT/assets/com.plebsigner.Signer.service" "$DEB_ROOT/usr/share/dbus-1/services/"
cp "$PROJECT_ROOT/README.md" "$DEB_ROOT/usr/share/doc/${PACKAGE_NAME}/"
cp "$PROJECT_ROOT/LICENSE" "$DEB_ROOT/usr/share/doc/${PACKAGE_NAME}/"
cp "$PROJECT_ROOT/BUNKER_IMPLEMENTATION.md" "$DEB_ROOT/usr/share/doc/${PACKAGE_NAME}/"

# Calculate installed size
INSTALLED_SIZE=$(du -s "$DEB_ROOT" | cut -f1)

# Create control file
cat > "$DEB_ROOT/DEBIAN/control" << EOF
Package: ${PACKAGE_NAME}
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: amd64
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

if [ -f "$PROJECT_ROOT/packaging/debian/postinst" ]; then
    cp "$PROJECT_ROOT/packaging/debian/postinst" "$DEB_ROOT/DEBIAN/"
    chmod 755 "$DEB_ROOT/DEBIAN/postinst"
fi

if [ -f "$PROJECT_ROOT/packaging/debian/postrm" ]; then
    cp "$PROJECT_ROOT/packaging/debian/postrm" "$DEB_ROOT/DEBIAN/"
    chmod 755 "$DEB_ROOT/DEBIAN/postrm"
fi

# Create a tarball of the DEB structure
cd "$BUILD_DIR/deb"
DEB_STRUCTURE="${PACKAGE_NAME}_${VERSION}_amd64-deb-structure.tar.gz"
tar czf "$RELEASE_DIR/$DEB_STRUCTURE" "${PACKAGE_NAME}_${VERSION}_amd64"
echo "✓ Created: $RELEASE_DIR/$DEB_STRUCTURE"
echo "  (Extract and run: dpkg-deb --build --root-owner-group <dir>)"

# ============================================
# Create RPM spec file
# ============================================
echo ""
echo "=== Creating RPM spec file ==="

cp "$PROJECT_ROOT/packaging/pleb-signer.spec" "$RELEASE_DIR/"
# Update version in spec file
sed -i "s/^Version:.*/Version:        ${VERSION}/" "$RELEASE_DIR/pleb-signer.spec"
echo "✓ Created: $RELEASE_DIR/pleb-signer.spec"

# ============================================
# Create checksums
# ============================================
echo ""
echo "=== Creating checksums ==="
cd "$RELEASE_DIR"
sha256sum *.tar.gz PKGBUILD pleb-signer.spec 2>/dev/null > SHA256SUMS.txt || true
echo "✓ Created: SHA256SUMS.txt"
echo ""
cat SHA256SUMS.txt

# ============================================
# Create release notes
# ============================================
cat > "$RELEASE_DIR/RELEASE_NOTES.md" << 'EOF'
# Pleb Signer v0.1.1 Release Notes

## What's New

### 🌐 NIP-46 Bunker Remote Signing
The headline feature of this release is **full NIP-46 bunker support** for remote signing!

**What does this mean?**
- Sign Nostr events from any device (phone, web browser, CLI) 
- Your private keys **never leave** your desktop
- Works over Nostr relays - no direct network connection needed
- Perfect for using mobile clients while keeping keys secure at home

### Features Implemented
- ✅ BunkerSigner integration into application state
- ✅ D-Bus methods: `StartBunker`, `GetBunkerUri`, `StopBunker`, `GetBunkerState`
- ✅ Auto-connect to default relays (relay.nsec.app, relay.damus.io)
- ✅ Support for all NIP-46 methods: `sign_event`, `nip04/nip44_encrypt/decrypt`, `ping`
- ✅ Test script for verifying bunker functionality

### How to Use

1. **Start Pleb Signer**
   ```bash
   pleb-signer
   ```

2. **Enable bunker mode** (via D-Bus)
   ```bash
   dbus-send --session --print-reply --dest=com.plebsigner.Signer \
     /com/plebsigner/Signer com.plebsigner.Signer1.StartBunker
   ```

3. **Get your bunker URI** (from the response above)
   ```
   bunker://your-pubkey-here?relay=wss://relay.nsec.app&relay=wss://relay.damus.io
   ```

4. **Scan the QR code** (or paste URI) in your remote Nostr client

5. **Sign events remotely!** Your desktop handles all signing securely

### Testing
Run the included test script:
```bash
./test_bunker.sh
```

## Installation

### Arch Linux
```bash
# Using PKGBUILD
wget https://github.com/PlebOne/Pleb_Signer/releases/download/v0.1.1/PKGBUILD
makepkg -si

# Or from binary tarball
wget https://github.com/PlebOne/Pleb_Signer/releases/download/v0.1.1/pleb-signer-0.1.1-linux-x86_64.tar.gz
tar xzf pleb-signer-*.tar.gz
cd pleb-signer-*
./install.sh
```

### Debian/Ubuntu
```bash
# Download DEB structure and build
wget https://github.com/PlebOne/Pleb_Signer/releases/download/v0.1.1/pleb-signer_0.1.1_amd64-deb-structure.tar.gz
tar xzf pleb-signer_*-deb-structure.tar.gz
dpkg-deb --build --root-owner-group pleb-signer_0.1.1_amd64
sudo dpkg -i pleb-signer_0.1.1_amd64.deb

# Or from binary tarball
wget https://github.com/PlebOne/Pleb_Signer/releases/download/v0.1.1/pleb-signer-0.1.1-linux-x86_64.tar.gz
tar xzf pleb-signer-*.tar.gz
cd pleb-signer-*
./install.sh
```

### Fedora/RHEL/CentOS
```bash
# Download and build RPM
wget https://github.com/PlebOne/Pleb_Signer/releases/download/v0.1.1/pleb-signer-0.1.1.tar.gz
wget https://github.com/PlebOne/Pleb_Signer/releases/download/v0.1.1/pleb-signer.spec
rpmbuild -tb pleb-signer-0.1.1.tar.gz

# Or from binary tarball
wget https://github.com/PlebOne/Pleb_Signer/releases/download/v0.1.1/pleb-signer-0.1.1-linux-x86_64.tar.gz
tar xzf pleb-signer-*.tar.gz
cd pleb-signer-*
./install.sh
```

### Generic Linux
```bash
wget https://github.com/PlebOne/Pleb_Signer/releases/download/v0.1.1/pleb-signer-0.1.1-linux-x86_64.tar.gz
tar xzf pleb-signer-*.tar.gz
cd pleb-signer-*
./install.sh
```

## Documentation
- [Bunker Implementation Details](BUNKER_IMPLEMENTATION.md)
- [Client Integration Guide](docs/CLIENT_INTEGRATION.md)
- [Test Bunker Script](test_bunker.sh)

## Checksums
See [SHA256SUMS.txt](SHA256SUMS.txt) for file checksums.

## Bug Fixes
- Fixed: Bunker/NIP-46 feature was documented but non-functional

## Known Issues
- None at this time

## Contributors
- @PlebOne

---
Full Changelog: [v0.1.0...v0.1.1](https://github.com/PlebOne/Pleb_Signer/compare/v0.1.0...v0.1.1)
EOF

echo "✓ Created: $RELEASE_DIR/RELEASE_NOTES.md"

echo ""
echo "=== Build complete ==="
echo "Release artifacts in: $RELEASE_DIR"
echo ""
ls -lh "$RELEASE_DIR"
echo ""
echo "To create packages:"
echo "  DEB: cd to extracted structure and run 'dpkg-deb --build --root-owner-group <dir>'"
echo "  RPM: rpmbuild -tb pleb-signer-${VERSION}.tar.gz"
echo "  Arch: makepkg -si (in directory with PKGBUILD)"
