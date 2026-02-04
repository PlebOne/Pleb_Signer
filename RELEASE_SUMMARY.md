# Release v0.1.1 - Complete! 🎉

## Summary

Successfully implemented bunker/NIP-46 remote signing functionality and released version 0.1.1 with full package support for all major Linux distributions.

## Release URL
**https://github.com/PlebOne/Pleb_Signer/releases/tag/v0.1.1**

## What Was Done

### 1. Bunker/NIP-46 Implementation ✅
- **Fixed**: Bunker feature was documented but completely non-functional
- **Added**: Full BunkerSigner integration into AppState
- **Added**: 4 D-Bus methods for bunker control:
  - `StartBunker()` - Start NIP-46 listener, returns bunker URI
  - `GetBunkerUri()` - Get connection URI
  - `StopBunker()` - Stop listener
  - `GetBunkerState()` - Check connection status
- **Added**: Default relay configuration (relay.nsec.app, relay.damus.io)
- **Added**: Test script (`test_bunker.sh`)
- **Added**: Implementation documentation (`BUNKER_IMPLEMENTATION.md`)

### 2. Version Bump ✅
- Updated from 0.1.0 → 0.1.1
- Updated Cargo.toml
- Updated build scripts

### 3. Code Changes ✅
**Files Modified:**
- `src/app.rs` (+56 lines) - BunkerSigner integration
- `src/dbus.rs` (+50 lines) - D-Bus method exposure
- `src/main.rs` (refactored) - Initialize bunker on startup
- `BUNKER_IMPLEMENTATION.md` (new) - Documentation
- `test_bunker.sh` (new) - Testing script

### 4. Build & Packaging ✅
**Built Successfully:**
- Release binary (20MB, optimized)
- Source tarball (428MB with full source)
- Binary tarball (9.1MB, ready to install)
- Arch Linux package (8.9MB .pkg.tar.zst)
- DEB package structure (9.1MB tarball)
- RPM spec file
- PKGBUILD for Arch
- SHA256 checksums

### 5. Git & GitHub ✅
- Committed changes with descriptive messages
- Pushed to origin/main
- Created release tag v0.1.1
- Published GitHub release with full release notes
- Uploaded all 7 release assets

## Release Assets (7 files)

1. **PKGBUILD** - Arch Linux package build file
2. **pleb-signer-0.1.1-1-x86_64.pkg.tar.zst** - Ready-to-install Arch package
3. **pleb-signer_0.1.1_amd64-deb-structure.tar.gz** - Debian/Ubuntu package structure
4. **pleb-signer-0.1.1-linux-x86_64.tar.gz** - Binary tarball (universal Linux)
5. **pleb-signer-0.1.1.tar.gz** - Source tarball
6. **pleb-signer.spec** - RPM spec file for Fedora/RHEL
7. **SHA256SUMS.txt** - Checksums for verification

## Installation Quick Start

### Arch Linux
```bash
# Download and install
curl -LO https://github.com/PlebOne/Pleb_Signer/releases/download/v0.1.1/pleb-signer-0.1.1-1-x86_64.pkg.tar.zst
sudo pacman -U pleb-signer-0.1.1-1-x86_64.pkg.tar.zst

# Or build from PKGBUILD
curl -LO https://github.com/PlebOne/Pleb_Signer/releases/download/v0.1.1/PKGBUILD
makepkg -si
```

### Debian/Ubuntu
```bash
# Download and extract DEB structure
curl -LO https://github.com/PlebOne/Pleb_Signer/releases/download/v0.1.1/pleb-signer_0.1.1_amd64-deb-structure.tar.gz
tar xzf pleb-signer_0.1.1_amd64-deb-structure.tar.gz

# Build and install
dpkg-deb --build --root-owner-group pleb-signer_0.1.1_amd64
sudo dpkg -i pleb-signer_0.1.1_amd64.deb
```

### Fedora/RHEL/CentOS
```bash
# Download source and spec
curl -LO https://github.com/PlebOne/Pleb_Signer/releases/download/v0.1.1/pleb-signer-0.1.1.tar.gz
curl -LO https://github.com/PlebOne/Pleb_Signer/releases/download/v0.1.1/pleb-signer.spec

# Build RPM
rpmbuild -tb pleb-signer-0.1.1.tar.gz
sudo rpm -i ~/rpmbuild/RPMS/x86_64/pleb-signer-*.rpm
```

### Generic Linux (any distro)
```bash
# Download and extract
curl -LO https://github.com/PlebOne/Pleb_Signer/releases/download/v0.1.1/pleb-signer-0.1.1-linux-x86_64.tar.gz
tar xzf pleb-signer-0.1.1-linux-x86_64.tar.gz
cd pleb-signer-0.1.1-linux-x86_64

# Install
./install.sh
```

## Testing Bunker Functionality

After installation:

```bash
# 1. Start Pleb Signer
pleb-signer

# 2. In another terminal, start bunker
dbus-send --session --print-reply --dest=com.plebsigner.Signer \
  /com/plebsigner/Signer com.plebsigner.Signer1.StartBunker

# 3. Use the returned bunker:// URI with any NIP-46 compatible client

# 4. Or run the test script
./test_bunker.sh
```

## Checksums

All release assets verified with SHA256:
```
0fa45b6f429fe54555922e6c4727b5137fab5f9b34990664477621ba1f672dd8  pleb-signer_0.1.1_amd64-deb-structure.tar.gz
546590f5f1ade8c3d63116227659133d60956ae9106f6424c2b3bd16d8b31181  pleb-signer-0.1.1-linux-x86_64.tar.gz
7e5116b929e863114bd15609f7126249a83211a6bd94cccb3d74c19e177c1fed  pleb-signer-0.1.1.tar.gz
7f53b828e8f91751146ccdfe14fcb239aca8a5ab10cd5c4f0f3ec667d97de713  PKGBUILD
7bd0b0266dad43e5fbfa1fc4deeb69b32325bc0ebe61c426d1b2bb5a7cb08bad  pleb-signer.spec
48246f91b665a1794c19baf07b732745ddaf01283855b45c7a4975f0bf88ac37  pleb-signer-0.1.1-1-x86_64.pkg.tar.zst
```

## Build Information

- **Compiler**: Rust stable (cargo 1.x)
- **Target**: x86_64-unknown-linux-gnu
- **Profile**: release (optimized)
- **Binary Size**: 20MB (stripped)
- **Package Sizes**: 8.9MB - 9.1MB (compressed)
- **Build Time**: ~3 minutes
- **Warnings**: 26 cosmetic warnings (lifetime elision)
- **Errors**: 0

## Git Commits

1. **feat: implement bunker/NIP-46 remote signing functionality** (2edc8b5)
   - Full implementation of bunker integration
   - D-Bus methods
   - Documentation

2. **chore: bump version to 0.1.1** (540ee20)
   - Version update in Cargo.toml
   - Build script version update

## What's Next (Future Enhancements)

Potential improvements for future releases:
- [ ] User approval prompts for remote connections
- [ ] UI integration for bunker status/management
- [ ] Connected clients list in UI
- [ ] Custom relay configuration
- [ ] QR code generation for easier mobile scanning
- [ ] Session persistence across restarts
- [ ] Per-client permissions management

## Support & Documentation

- **Repository**: https://github.com/PlebOne/Pleb_Signer
- **Release**: https://github.com/PlebOne/Pleb_Signer/releases/tag/v0.1.1
- **Issues**: https://github.com/PlebOne/Pleb_Signer/issues
- **Docs**: See `docs/CLIENT_INTEGRATION.md` in release assets

## Credits

- Implementation: @PlebOne
- NIP-46 Protocol: Nostr community
- Dependencies: rust-nostr ecosystem

---

**Release completed successfully on 2026-02-03** 🚀
