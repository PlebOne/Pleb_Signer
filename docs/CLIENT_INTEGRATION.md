# Pleb Signer - Client Integration Guide

Pleb Signer is a Linux desktop Nostr signer that runs in the system tray and provides secure key management and signing services via D-Bus. This guide explains how to integrate your Nostr client with Pleb Signer.

## Overview

Instead of managing private keys directly, your application communicates with Pleb Signer over D-Bus to:
- Get the user's public key
- Sign Nostr events  
- Encrypt/decrypt messages (NIP-04 and NIP-44)

This keeps private keys secure in the OS keyring, never exposing them to client applications.

## D-Bus Connection Details

| Property | Value |
|----------|-------|
| **Bus** | Session Bus |
| **Service Name** | `com.plebsigner.Signer` |
| **Object Path** | `/com/plebsigner/Signer` |
| **Interface** | `com.plebsigner.Signer1` |

## Available Methods

### `Version() → String`
Returns the signer version.

### `IsReady() → Boolean`
Returns `true` if the signer is unlocked and ready to sign.

### `GetPublicKey() → String`
Returns a JSON response with the user's active public key (hex format).

### `ListKeys() → String`
Returns a JSON array of available keys with their public info.

### `SignEvent(event_json: String, app_id: String) → String`
Signs a Nostr event. The `event_json` should contain:
```json
{
  "kind": 1,
  "content": "Hello, Nostr!",
  "tags": [],
  "created_at": 1234567890
}
```

### `Nip04Encrypt(plaintext: String, recipient_pubkey: String, app_id: String) → String`
Encrypts a message using NIP-04 (deprecated but still widely used).

### `Nip04Decrypt(ciphertext: String, sender_pubkey: String, app_id: String) → String`
Decrypts a NIP-04 encrypted message.

### `Nip44Encrypt(plaintext: String, recipient_pubkey: String, app_id: String) → String`
Encrypts a message using NIP-44 (recommended).

### `Nip44Decrypt(ciphertext: String, sender_pubkey: String, app_id: String) → String`
Decrypts a NIP-44 encrypted message.

### `DecryptZapEvent(event_json: String, app_id: String) → String`
Decrypts a zap request event.

## Response Format

All methods return a JSON string:

**Success:**
```json
{
  "success": true,
  "id": "req_1a2b3c4d",
  "result": "\"<result_data>\"",
  "error": null
}
```

**Error:**
```json
{
  "success": false,
  "id": "req_1a2b3c4d",
  "result": null,
  "error": "Error description"
}
```

---

## Integration Examples

### Command Line (dbus-send)

```bash
# Check if signer is ready
dbus-send --session --print-reply --dest=com.plebsigner.Signer \
  /com/plebsigner/Signer com.plebsigner.Signer1.IsReady

# Get public key
dbus-send --session --print-reply --dest=com.plebsigner.Signer \
  /com/plebsigner/Signer com.plebsigner.Signer1.GetPublicKey

# List available keys
dbus-send --session --print-reply --dest=com.plebsigner.Signer \
  /com/plebsigner/Signer com.plebsigner.Signer1.ListKeys

# Sign an event
dbus-send --session --print-reply --dest=com.plebsigner.Signer \
  /com/plebsigner/Signer com.plebsigner.Signer1.SignEvent \
  string:'{"kind":1,"content":"Hello Nostr!","tags":[],"created_at":1732800000}' \
  string:'my-app-id'

# NIP-04 encrypt
dbus-send --session --print-reply --dest=com.plebsigner.Signer \
  /com/plebsigner/Signer com.plebsigner.Signer1.Nip04Encrypt \
  string:'Secret message' \
  string:'recipient_pubkey_hex' \
  string:'my-app-id'
```

### Python

```python
#!/usr/bin/env python3
"""Example Nostr client using Pleb Signer for key management."""

import json
import time
from pydbus import SessionBus

class PlebSignerClient:
    SERVICE = "com.plebsigner.Signer"
    PATH = "/com/plebsigner/Signer"
    
    def __init__(self, app_id: str = "python-client"):
        self.bus = SessionBus()
        self.signer = self.bus.get(self.SERVICE, self.PATH)
        self.app_id = app_id
    
    def _parse_response(self, response: str) -> dict:
        data = json.loads(response)
        if not data.get("success"):
            raise Exception(data.get("error", "Unknown error"))
        # Result is double-encoded JSON
        return json.loads(data["result"]) if data.get("result") else None
    
    def is_ready(self) -> bool:
        return self.signer.IsReady()
    
    def get_public_key(self) -> str:
        response = self.signer.GetPublicKey()
        return self._parse_response(response)
    
    def list_keys(self) -> list:
        response = self.signer.ListKeys()
        return json.loads(response)
    
    def sign_event(self, kind: int, content: str, tags: list = None) -> dict:
        event = {
            "kind": kind,
            "content": content,
            "tags": tags or [],
            "created_at": int(time.time())
        }
        response = self.signer.SignEvent(json.dumps(event), self.app_id)
        return self._parse_response(response)
    
    def nip04_encrypt(self, plaintext: str, recipient_pubkey: str) -> str:
        response = self.signer.Nip04Encrypt(plaintext, recipient_pubkey, self.app_id)
        return self._parse_response(response)
    
    def nip04_decrypt(self, ciphertext: str, sender_pubkey: str) -> str:
        response = self.signer.Nip04Decrypt(ciphertext, sender_pubkey, self.app_id)
        return self._parse_response(response)
    
    def nip44_encrypt(self, plaintext: str, recipient_pubkey: str) -> str:
        response = self.signer.Nip44Encrypt(plaintext, recipient_pubkey, self.app_id)
        return self._parse_response(response)
    
    def nip44_decrypt(self, ciphertext: str, sender_pubkey: str) -> str:
        response = self.signer.Nip44Decrypt(ciphertext, sender_pubkey, self.app_id)
        return self._parse_response(response)


# Usage example
if __name__ == "__main__":
    client = PlebSignerClient("my-nostr-app")
    
    # Check if signer is available and unlocked
    if not client.is_ready():
        print("Pleb Signer is locked or not running")
        exit(1)
    
    # Get our public key
    pubkey = client.get_public_key()
    print(f"My public key: {pubkey}")
    
    # Sign a note
    signed_event = client.sign_event(
        kind=1,
        content="Hello from Python!"
    )
    print(f"Signed event: {json.dumps(signed_event, indent=2)}")
```

### Rust

```rust
//! Example Nostr client using Pleb Signer

use serde::{Deserialize, Serialize};
use zbus::{Connection, Result, proxy};

#[derive(Debug, Deserialize)]
struct SignerResponse {
    success: bool,
    id: String,
    result: Option<String>,
    error: Option<String>,
}

#[proxy(
    interface = "com.plebsigner.Signer1",
    default_service = "com.plebsigner.Signer",
    default_path = "/com/plebsigner/Signer"
)]
trait Signer {
    async fn version(&self) -> Result<String>;
    async fn is_ready(&self) -> Result<bool>;
    async fn get_public_key(&self) -> Result<String>;
    async fn list_keys(&self) -> Result<String>;
    async fn sign_event(&self, event_json: &str, app_id: &str) -> Result<String>;
    async fn nip04_encrypt(&self, plaintext: &str, recipient: &str, app_id: &str) -> Result<String>;
    async fn nip04_decrypt(&self, ciphertext: &str, sender: &str, app_id: &str) -> Result<String>;
    async fn nip44_encrypt(&self, plaintext: &str, recipient: &str, app_id: &str) -> Result<String>;
    async fn nip44_decrypt(&self, ciphertext: &str, sender: &str, app_id: &str) -> Result<String>;
}

#[derive(Serialize)]
struct UnsignedEvent {
    kind: u32,
    content: String,
    tags: Vec<Vec<String>>,
    created_at: u64,
}

pub struct PlebSignerClient<'a> {
    proxy: SignerProxy<'a>,
    app_id: String,
}

impl<'a> PlebSignerClient<'a> {
    pub async fn new(connection: &'a Connection, app_id: &str) -> Result<Self> {
        let proxy = SignerProxy::new(connection).await?;
        Ok(Self {
            proxy,
            app_id: app_id.to_string(),
        })
    }

    fn parse_response(response: &str) -> std::result::Result<String, String> {
        let parsed: SignerResponse = serde_json::from_str(response)
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        
        if parsed.success {
            Ok(parsed.result.unwrap_or_default())
        } else {
            Err(parsed.error.unwrap_or_else(|| "Unknown error".to_string()))
        }
    }

    pub async fn is_ready(&self) -> Result<bool> {
        self.proxy.is_ready().await
    }

    pub async fn get_public_key(&self) -> std::result::Result<String, String> {
        let response = self.proxy.get_public_key().await
            .map_err(|e| e.to_string())?;
        Self::parse_response(&response)
    }

    pub async fn sign_event(
        &self,
        kind: u32,
        content: &str,
        tags: Vec<Vec<String>>,
    ) -> std::result::Result<String, String> {
        let event = UnsignedEvent {
            kind,
            content: content.to_string(),
            tags,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        let event_json = serde_json::to_string(&event)
            .map_err(|e| e.to_string())?;
        
        let response = self.proxy.sign_event(&event_json, &self.app_id).await
            .map_err(|e| e.to_string())?;
        
        Self::parse_response(&response)
    }

    pub async fn nip44_encrypt(
        &self,
        plaintext: &str,
        recipient_pubkey: &str,
    ) -> std::result::Result<String, String> {
        let response = self.proxy
            .nip44_encrypt(plaintext, recipient_pubkey, &self.app_id)
            .await
            .map_err(|e| e.to_string())?;
        Self::parse_response(&response)
    }

    pub async fn nip44_decrypt(
        &self,
        ciphertext: &str,
        sender_pubkey: &str,
    ) -> std::result::Result<String, String> {
        let response = self.proxy
            .nip44_decrypt(ciphertext, sender_pubkey, &self.app_id)
            .await
            .map_err(|e| e.to_string())?;
        Self::parse_response(&response)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let connection = Connection::session().await?;
    let client = PlebSignerClient::new(&connection, "rust-example").await?;
    
    // Check availability
    if !client.is_ready().await? {
        eprintln!("Pleb Signer is locked or not running");
        return Ok(());
    }
    
    // Get public key
    match client.get_public_key().await {
        Ok(pubkey) => println!("Public key: {}", pubkey),
        Err(e) => eprintln!("Error: {}", e),
    }
    
    // Sign a note
    match client.sign_event(1, "Hello from Rust!", vec![]).await {
        Ok(signed) => println!("Signed event: {}", signed),
        Err(e) => eprintln!("Error signing: {}", e),
    }
    
    Ok(())
}
```

### JavaScript/TypeScript (Node.js)

```typescript
import DBus from 'dbus-next';

interface SignerResponse {
  success: boolean;
  id: string;
  result?: string;
  error?: string;
}

class PlebSignerClient {
  private proxy: any;
  private appId: string;

  constructor(appId: string = 'js-client') {
    this.appId = appId;
  }

  async connect(): Promise<void> {
    const bus = DBus.sessionBus();
    const obj = await bus.getProxyObject(
      'com.plebsigner.Signer',
      '/com/plebsigner/Signer'
    );
    this.proxy = obj.getInterface('com.plebsigner.Signer1');
  }

  private parseResponse(response: string): any {
    const data: SignerResponse = JSON.parse(response);
    if (!data.success) {
      throw new Error(data.error || 'Unknown error');
    }
    return data.result ? JSON.parse(data.result) : null;
  }

  async isReady(): Promise<boolean> {
    return await this.proxy.IsReady();
  }

  async getPublicKey(): Promise<string> {
    const response = await this.proxy.GetPublicKey();
    return this.parseResponse(response);
  }

  async listKeys(): Promise<any[]> {
    const response = await this.proxy.ListKeys();
    return JSON.parse(response);
  }

  async signEvent(kind: number, content: string, tags: string[][] = []): Promise<any> {
    const event = {
      kind,
      content,
      tags,
      created_at: Math.floor(Date.now() / 1000)
    };
    const response = await this.proxy.SignEvent(JSON.stringify(event), this.appId);
    return this.parseResponse(response);
  }

  async nip04Encrypt(plaintext: string, recipientPubkey: string): Promise<string> {
    const response = await this.proxy.Nip04Encrypt(plaintext, recipientPubkey, this.appId);
    return this.parseResponse(response);
  }

  async nip04Decrypt(ciphertext: string, senderPubkey: string): Promise<string> {
    const response = await this.proxy.Nip04Decrypt(ciphertext, senderPubkey, this.appId);
    return this.parseResponse(response);
  }

  async nip44Encrypt(plaintext: string, recipientPubkey: string): Promise<string> {
    const response = await this.proxy.Nip44Encrypt(plaintext, recipientPubkey, this.appId);
    return this.parseResponse(response);
  }

  async nip44Decrypt(ciphertext: string, senderPubkey: string): Promise<string> {
    const response = await this.proxy.Nip44Decrypt(ciphertext, senderPubkey, this.appId);
    return this.parseResponse(response);
  }
}

// Usage
async function main() {
  const client = new PlebSignerClient('my-nostr-app');
  await client.connect();

  if (!await client.isReady()) {
    console.error('Pleb Signer is locked or not running');
    process.exit(1);
  }

  const pubkey = await client.getPublicKey();
  console.log('Public key:', pubkey);

  const signedEvent = await client.signEvent(1, 'Hello from JavaScript!');
  console.log('Signed event:', signedEvent);
}

main().catch(console.error);
```

---

## Login Flow for Nostr Clients

Here's a typical "login" flow using Pleb Signer:

```
┌─────────────────┐     ┌─────────────────┐
│  Your Nostr     │     │  Pleb Signer    │
│  Client App     │     │  (System Tray)  │
└────────┬────────┘     └────────┬────────┘
         │                       │
         │  1. IsReady()         │
         │──────────────────────>│
         │                       │
         │  true/false           │
         │<──────────────────────│
         │                       │
         │  2. GetPublicKey()    │
         │──────────────────────>│
         │                       │
         │  { pubkey: "abc..." } │
         │<──────────────────────│
         │                       │
         │  3. Display pubkey    │
         │  to user as their     │
         │  "logged in" identity │
         │                       │
         │  4. When posting...   │
         │  SignEvent(event)     │
         │──────────────────────>│
         │                       │
         │  { signed_event }     │
         │<──────────────────────│
         │                       │
         │  5. Publish to relays │
         │                       │
```

### Step-by-Step:

1. **Check Availability**: Call `IsReady()` to verify Pleb Signer is running and unlocked
2. **Get Identity**: Call `GetPublicKey()` to get the user's active public key
3. **Display Identity**: Show the user their npub/pubkey as confirmation they're "logged in"
4. **Sign When Needed**: Call `SignEvent()` whenever the user creates content
5. **Encrypt DMs**: Use `Nip44Encrypt()`/`Nip44Decrypt()` for private messages

### Handling Multiple Keys

Users can have multiple keys in Pleb Signer. Use `ListKeys()` to show available identities:

```python
keys = client.list_keys()
for key in keys:
    status = "✓ ACTIVE" if key["is_active"] else ""
    print(f"{key['name']}: {key['npub'][:20]}... {status}")
```

Users switch their active key in the Pleb Signer UI.

---

## Error Handling

Always handle these cases:

1. **Signer not running**: D-Bus connection fails
2. **Signer is locked**: `IsReady()` returns `false`
3. **No active key**: `GetPublicKey()` returns error
4. **User rejection**: Future versions may prompt user for approval

```python
try:
    if not client.is_ready():
        show_message("Please unlock Pleb Signer")
        return
    
    pubkey = client.get_public_key()
except Exception as e:
    if "org.freedesktop.DBus.Error.ServiceUnknown" in str(e):
        show_message("Please start Pleb Signer")
    else:
        show_message(f"Signer error: {e}")
```

---

## Security Considerations

- **Private keys never leave Pleb Signer** - your app only sees public keys and signatures
- **Keys are stored in the OS keyring** (GNOME Keyring, KWallet, etc.)
- **App ID tracking** - Pass a unique `app_id` for future permission/audit features
- **Session bus only** - D-Bus session bus provides per-user isolation

---

## Troubleshooting

### "Service not found" error
```bash
# Check if Pleb Signer is running
pgrep -a pleb-signer

# Check D-Bus service registration  
dbus-send --session --print-reply --dest=org.freedesktop.DBus \
  /org/freedesktop/DBus org.freedesktop.DBus.ListNames | grep plebsigner
```

### "Signer is locked" error
Open Pleb Signer from the system tray and unlock it, or ensure a key is active.

### D-Bus introspection
```bash
# View available methods
dbus-send --session --print-reply --dest=com.plebsigner.Signer \
  /com/plebsigner/Signer org.freedesktop.DBus.Introspectable.Introspect
```

---

---

## NIP-46 Bunker Mode (Remote Signing)

Pleb Signer also supports **NIP-46 (Nostr Connect)**, allowing you to sign events remotely from any device - even mobile phones or web apps - without exposing your private key.

### How It Works

```
┌─────────────────┐                    ┌─────────────────┐
│  Remote Client  │                    │  Pleb Signer    │
│  (Phone/Web)    │                    │  (Your Desktop) │
└────────┬────────┘                    └────────┬────────┘
         │                                      │
         │  1. Scan bunker:// QR code           │
         │─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─>│
         │                                      │
         │  2. Connect via Nostr relays         │
         │<─────────────────────────────────────>│
         │                                      │
         │  3. Request: sign_event(...)         │
         │─────────────────────────────────────>│
         │                                      │
         │  4. Response: signed event           │
         │<─────────────────────────────────────│
         │                                      │
```

### Connection URIs

Pleb Signer generates two types of connection URIs:

#### `bunker://` URI (Recommended)
```
bunker://<signer-pubkey>?relay=wss://relay.nsec.app&relay=wss://relay.damus.io
```

#### `nostrconnect://` URI (Alternative)
```
nostrconnect://<signer-pubkey>?relay=wss://relay.nsec.app&metadata={"name":"Pleb Signer"}
```

### Enabling Bunker Mode

#### Via D-Bus

```bash
# Start bunker listener
dbus-send --session --print-reply --dest=com.plebsigner.Signer \
  /com/plebsigner/Signer com.plebsigner.Signer1.StartBunker

# Get connection URI (to show as QR code)
dbus-send --session --print-reply --dest=com.plebsigner.Signer \
  /com/plebsigner/Signer com.plebsigner.Signer1.GetBunkerUri

# Stop bunker listener
dbus-send --session --print-reply --dest=com.plebsigner.Signer \
  /com/plebsigner/Signer com.plebsigner.Signer1.StopBunker
```

#### Via UI

1. Open Pleb Signer from system tray
2. Go to Settings tab
3. Enable "Bunker Mode"
4. Scan the QR code with your remote client

### Supported NIP-46 Methods

| Method | Description |
|--------|-------------|
| `connect` | Establish connection |
| `get_public_key` | Get signer's public key |
| `sign_event` | Sign an unsigned event |
| `nip04_encrypt` | Encrypt with NIP-04 |
| `nip04_decrypt` | Decrypt with NIP-04 |
| `nip44_encrypt` | Encrypt with NIP-44 |
| `nip44_decrypt` | Decrypt with NIP-44 |
| `ping` | Test connection |

### Client Integration (NIP-46)

#### Python (using nostr-sdk)

```python
from nostr_sdk import Client, NostrConnectUri, Keys

# Parse the bunker URI from Pleb Signer
bunker_uri = "bunker://abc123...?relay=wss://relay.nsec.app"

# Create a client with remote signer
uri = NostrConnectUri.parse(bunker_uri)
client = Client.with_remote_signer(uri)

await client.connect()

# Now you can sign events remotely
event = EventBuilder.text_note("Hello from remote!")
signed = await client.sign_event(event)
```

#### JavaScript (using nostr-tools)

```javascript
import { nip46 } from 'nostr-tools';

const bunkerUri = 'bunker://abc123...?relay=wss://relay.nsec.app';

// Create NIP-46 remote signer
const remoteSigner = new nip46.BunkerSigner(
  clientSecretKey,  // Your client's ephemeral key
  bunkerUri
);

await remoteSigner.connect();

// Get public key from remote signer
const pubkey = await remoteSigner.getPublicKey();

// Sign event remotely
const signedEvent = await remoteSigner.signEvent(unsignedEvent);
```

#### Rust (using nostr-sdk)

```rust
use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let bunker_uri = NostrConnectUri::parse(
        "bunker://abc123...?relay=wss://relay.nsec.app"
    )?;
    
    // Create client with remote signer
    let client = Client::with_remote_signer(bunker_uri, None).await?;
    client.connect().await;
    
    // Sign events remotely
    let event = EventBuilder::text_note("Hello from remote!");
    let signed = client.sign_event(event).await?;
    
    Ok(())
}
```

### Security Considerations for Bunker Mode

- **Private key stays local**: Your key never leaves Pleb Signer
- **Relay-based communication**: Uses Nostr relays for transport (encrypted)
- **Optional secret**: Add a secret to the URI for additional authentication
- **Connection approval**: Future versions will prompt before accepting connections
- **Relay selection**: Use trusted relays for lower latency and better privacy

### Use Cases

1. **Mobile Signing**: Use your phone's Nostr client while keys stay on desktop
2. **Web Apps**: Sign from web browsers without browser extensions
3. **Shared Computers**: Sign on untrusted machines without exposing keys
4. **Multi-Device**: Use the same identity across all your devices

---

## Future Enhancements

- **Permission prompts**: Ask user before signing (like browser extension popups)
- **Browser bridge**: Native messaging for NIP-07 compatibility in browsers
- **Standardized interface**: Working toward `org.nostr.Signer` standard
- **Bunker approval UI**: Prompt before accepting remote connections

---

## Need Help?

- Open an issue on the Pleb Signer repository
- Check the system tray icon - hover for status
- Ensure your OS keyring service is running (e.g., `gnome-keyring-daemon`)
