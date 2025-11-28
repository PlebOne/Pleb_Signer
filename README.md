# Pleb Signer

A NIP-55 compatible Nostr signer for Linux, similar to Amber for Android.

## Features

- 🔐 **Secure Key Management**: Keys are encrypted at rest using ChaCha20-Poly1305 with Argon2 key derivation
- 🖥️ **System Tray Integration**: Runs quietly in the background, always ready to sign
- 📝 **Event Signing**: Sign Nostr events with user approval
- 🔒 **NIP-04 & NIP-44 Encryption**: Support for both encryption standards
- ⚡ **Auto-Approve**: Optional auto-approval for trusted applications
- 🎨 **Modern UI**: Clean, dark-themed interface built with Iced

## Installation

### From Source

```bash
# Install dependencies (Debian/Ubuntu)
sudo apt install libdbus-1-dev libssl-dev pkg-config

# Clone and build
git clone https://github.com/example/pleb-signer.git
cd pleb-signer
cargo build --release

# Install
sudo cp target/release/pleb-signer /usr/local/bin/
```

### Requirements

- Linux with D-Bus session bus
- A system tray (KDE, GNOME with extensions, etc.)
- Rust 1.70+ (for building)

## Usage

### Starting the Signer

```bash
# Start the signer
pleb-signer

# Start minimized to tray
pleb-signer --minimized
```

### First-Time Setup

1. Launch Pleb Signer
2. Create a strong password (8+ characters, letters and numbers)
3. Generate a new key or import an existing one (nsec/hex)

### D-Bus API

Other applications can interact with Pleb Signer via D-Bus:

**Service**: `com.plebsigner.Signer`  
**Object Path**: `/com/plebsigner/Signer`  
**Interface**: `com.plebsigner.Signer1`

#### Methods

| Method | Parameters | Returns | Description |
|--------|------------|---------|-------------|
| `Version` | - | String | Get signer version |
| `IsReady` | - | Boolean | Check if signer is unlocked |
| `ListKeys` | - | JSON Array | List all keys (public info) |
| `GetPublicKey` | `key_id: String` | JSON | Get public key |
| `SignEvent` | `event_json, key_id, app_id` | JSON | Sign a Nostr event |
| `Nip04Encrypt` | `plaintext, recipient, key_id, app_id` | JSON | NIP-04 encrypt |
| `Nip04Decrypt` | `ciphertext, sender, key_id, app_id` | JSON | NIP-04 decrypt |
| `Nip44Encrypt` | `plaintext, recipient, key_id, app_id` | JSON | NIP-44 encrypt |
| `Nip44Decrypt` | `ciphertext, sender, key_id, app_id` | JSON | NIP-44 decrypt |

#### Example (using dbus-send)

```bash
# Check if signer is ready
dbus-send --session --dest=com.plebsigner.Signer \
  --type=method_call --print-reply \
  /com/plebsigner/Signer \
  com.plebsigner.Signer1.IsReady

# Get public key
dbus-send --session --dest=com.plebsigner.Signer \
  --type=method_call --print-reply \
  /com/plebsigner/Signer \
  com.plebsigner.Signer1.GetPublicKey string:""
```

#### Example (Rust client)

```rust
use pleb_signer::client::PlebSignerClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = PlebSignerClient::new("my-nostr-app").await?;
    
    // Check if signer is ready
    if !client.is_ready().await? {
        println!("Please unlock Pleb Signer first");
        return Ok(());
    }
    
    // Get public key
    let pubkey = client.get_public_key(None).await?;
    println!("Public Key: {}", pubkey.npub);
    
    // Sign an event
    let event_json = r#"{"kind": 1, "content": "Hello Nostr!", "tags": []}"#;
    let signed = client.sign_event(event_json, None).await?;
    println!("Signed Event: {}", signed.event_json);
    
    Ok(())
}
```

## Configuration

Configuration is stored in `~/.config/plebsigner/PlebSigner/config.toml`

```toml
[general]
start_minimized = true
auto_start = false
show_notifications = true
request_timeout_secs = 60

[security]
require_password_on_start = true
lock_timeout_mins = 15
always_confirm = true
allow_auto_approve = false
max_auto_approvals_per_min = 10

[ui]
theme = "dark"
show_event_content = true
compact_mode = false
```

## Security

### Key Storage

Keys are stored encrypted at `~/.local/share/plebsigner/PlebSigner/keys.enc`

- Password-based key derivation using Argon2
- Encryption using ChaCha20-Poly1305
- Keys are zeroized in memory when locked

### Permissions

Each application must be authorized before it can request signatures. You can:

- Authorize specific event kinds
- Enable/disable NIP-04/NIP-44 operations
- Allow auto-approval for trusted apps
- Set rate limits for auto-approved requests

## NIP-55 Compatibility

This signer implements the NIP-55 protocol adapted for Linux:

- **Android Intent → D-Bus Method Call**
- **Content Resolver → D-Bus Properties/Methods**
- Supports all NIP-55 operations:
  - `get_public_key`
  - `sign_event`
  - `nip04_encrypt` / `nip04_decrypt`
  - `nip44_encrypt` / `nip44_decrypt`
  - `decrypt_zap_event`

## Autostart

To start Pleb Signer automatically on login:

```bash
# Create autostart entry
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
EOF
```

## Development

### Building

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

### Project Structure

```
src/
├── main.rs           # Entry point
├── app.rs            # Application state
├── config.rs         # Configuration management
├── crypto.rs         # Encryption utilities
├── dbus.rs           # D-Bus service
├── error.rs          # Error types
├── keys.rs           # Key management
├── permissions.rs    # Permission handling
├── signing.rs        # Signing operations
├── tray.rs           # System tray
├── client.rs         # Client library
└── ui/
    ├── mod.rs            # Main UI
    ├── approval_dialog.rs
    ├── components.rs
    ├── key_management.rs
    ├── settings.rs
    ├── styles.rs
    └── unlock.rs
```

## License

MIT License - see LICENSE file

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

- [Amber](https://github.com/greenart7c3/Amber) - Android Nostr Signer (inspiration)
- [nostr-rust](https://github.com/rust-nostr/nostr) - Nostr protocol implementation
- [iced](https://github.com/iced-rs/iced) - Cross-platform GUI library
- [ksni](https://github.com/ksni) - Linux system tray implementation
