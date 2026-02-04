//! Pleb Signer Client Library
//!
//! This module provides a simple client library that other applications
//! can use to communicate with the Pleb Signer D-Bus service.

use serde::{Deserialize, Serialize};
use zbus::{Connection, Proxy};

/// Response from the signer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignerResponse {
    pub success: bool,
    pub id: String,
    pub result: Option<String>,
    pub error: Option<String>,
}

/// Public key response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKeyResult {
    pub pubkey_hex: String,
    pub npub: String,
}

/// Signed event response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedEventResult {
    pub event_json: String,
    pub signature: String,
    pub event_id: String,
}

/// Encryption result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptResult {
    pub ciphertext: String,
}

/// Decryption result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecryptResult {
    pub plaintext: String,
}

/// Key info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyInfo {
    pub id: String,
    pub name: String,
    pub pubkey_hex: String,
    pub npub: String,
    pub is_default: bool,
}

/// Client error type that is Send + Sync
#[derive(Debug, Clone)]
pub struct ClientError(pub String);

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ClientError {}

impl From<zbus::Error> for ClientError {
    fn from(e: zbus::Error) -> Self {
        ClientError(e.to_string())
    }
}

impl From<serde_json::Error> for ClientError {
    fn from(e: serde_json::Error) -> Self {
        ClientError(e.to_string())
    }
}

impl From<String> for ClientError {
    fn from(s: String) -> Self {
        ClientError(s)
    }
}

impl From<&str> for ClientError {
    fn from(s: &str) -> Self {
        ClientError(s.to_string())
    }
}

/// Pleb Signer client
pub struct PlebSignerClient {
    connection: Connection,
    app_id: String,
}

impl PlebSignerClient {
    /// Create a new client with the given application ID
    pub async fn new(app_id: &str) -> Result<Self, ClientError> {
        let connection = Connection::session().await?;
        Ok(Self {
            connection,
            app_id: app_id.to_string(),
        })
    }

    /// Check if the signer is running
    pub async fn is_available(&self) -> bool {
        let proxy = Proxy::new(
            &self.connection,
            "com.plebsigner.Signer",
            "/com/plebsigner/Signer",
            "com.plebsigner.Signer1",
        )
        .await;

        proxy.is_ok()
    }

    /// Check if the signer is unlocked and ready
    pub async fn is_ready(&self) -> Result<bool, ClientError> {
        let proxy = Proxy::new(
            &self.connection,
            "com.plebsigner.Signer",
            "/com/plebsigner/Signer",
            "com.plebsigner.Signer1",
        )
        .await?;

        let result: bool = proxy.call("IsReady", &()).await?;
        Ok(result)
    }

    /// Get the signer version
    pub async fn version(&self) -> Result<String, ClientError> {
        let proxy = Proxy::new(
            &self.connection,
            "com.plebsigner.Signer",
            "/com/plebsigner/Signer",
            "com.plebsigner.Signer1",
        )
        .await?;

        let result: String = proxy.call("Version", &()).await?;
        Ok(result)
    }

    /// List all available keys
    pub async fn list_keys(&self) -> Result<Vec<KeyInfo>, ClientError> {
        let proxy = Proxy::new(
            &self.connection,
            "com.plebsigner.Signer",
            "/com/plebsigner/Signer",
            "com.plebsigner.Signer1",
        )
        .await?;

        let result: String = proxy.call("ListKeys", &()).await?;
        let keys: Vec<KeyInfo> = serde_json::from_str(&result)?;
        Ok(keys)
    }

    /// Get the public key
    pub async fn get_public_key(
        &self,
        key_id: Option<&str>,
    ) -> Result<PublicKeyResult, ClientError> {
        let proxy = Proxy::new(
            &self.connection,
            "com.plebsigner.Signer",
            "/com/plebsigner/Signer",
            "com.plebsigner.Signer1",
        )
        .await?;

        let key_id_str = key_id.unwrap_or("");
        let result: String = proxy.call("GetPublicKey", &(key_id_str,)).await?;

        let response: SignerResponse = serde_json::from_str(&result)?;
        if response.success {
            let pubkey: PublicKeyResult =
                serde_json::from_str(&response.result.unwrap_or_default())?;
            Ok(pubkey)
        } else {
            Err(ClientError(response.error.unwrap_or_else(|| "Unknown error".into())))
        }
    }

    /// Sign an event
    pub async fn sign_event(
        &self,
        event_json: &str,
        key_id: Option<&str>,
    ) -> Result<SignedEventResult, ClientError> {
        let proxy = Proxy::new(
            &self.connection,
            "com.plebsigner.Signer",
            "/com/plebsigner/Signer",
            "com.plebsigner.Signer1",
        )
        .await?;

        let key_id_str = key_id.unwrap_or("");
        let result: String = proxy
            .call("SignEvent", &(event_json, key_id_str, &self.app_id))
            .await?;

        let response: SignerResponse = serde_json::from_str(&result)?;
        if response.success {
            let signed: SignedEventResult =
                serde_json::from_str(&response.result.unwrap_or_default())?;
            Ok(signed)
        } else {
            Err(ClientError(response.error.unwrap_or_else(|| "Unknown error".into())))
        }
    }

    /// NIP-04 encrypt
    pub async fn nip04_encrypt(
        &self,
        plaintext: &str,
        recipient_pubkey: &str,
        key_id: Option<&str>,
    ) -> Result<String, ClientError> {
        let proxy = Proxy::new(
            &self.connection,
            "com.plebsigner.Signer",
            "/com/plebsigner/Signer",
            "com.plebsigner.Signer1",
        )
        .await?;

        let key_id_str = key_id.unwrap_or("");
        let result: String = proxy
            .call(
                "Nip04Encrypt",
                &(plaintext, recipient_pubkey, key_id_str, &self.app_id),
            )
            .await?;

        let response: SignerResponse = serde_json::from_str(&result)?;
        if response.success {
            let encrypted: EncryptResult =
                serde_json::from_str(&response.result.unwrap_or_default())?;
            Ok(encrypted.ciphertext)
        } else {
            Err(ClientError(response.error.unwrap_or_else(|| "Unknown error".into())))
        }
    }

    /// NIP-04 decrypt
    pub async fn nip04_decrypt(
        &self,
        ciphertext: &str,
        sender_pubkey: &str,
        key_id: Option<&str>,
    ) -> Result<String, ClientError> {
        let proxy = Proxy::new(
            &self.connection,
            "com.plebsigner.Signer",
            "/com/plebsigner/Signer",
            "com.plebsigner.Signer1",
        )
        .await?;

        let key_id_str = key_id.unwrap_or("");
        let result: String = proxy
            .call(
                "Nip04Decrypt",
                &(ciphertext, sender_pubkey, key_id_str, &self.app_id),
            )
            .await?;

        let response: SignerResponse = serde_json::from_str(&result)?;
        if response.success {
            let decrypted: DecryptResult =
                serde_json::from_str(&response.result.unwrap_or_default())?;
            Ok(decrypted.plaintext)
        } else {
            Err(ClientError(response.error.unwrap_or_else(|| "Unknown error".into())))
        }
    }

    /// NIP-44 encrypt
    pub async fn nip44_encrypt(
        &self,
        plaintext: &str,
        recipient_pubkey: &str,
        key_id: Option<&str>,
    ) -> Result<String, ClientError> {
        let proxy = Proxy::new(
            &self.connection,
            "com.plebsigner.Signer",
            "/com/plebsigner/Signer",
            "com.plebsigner.Signer1",
        )
        .await?;

        let key_id_str = key_id.unwrap_or("");
        let result: String = proxy
            .call(
                "Nip44Encrypt",
                &(plaintext, recipient_pubkey, key_id_str, &self.app_id),
            )
            .await?;

        let response: SignerResponse = serde_json::from_str(&result)?;
        if response.success {
            let encrypted: EncryptResult =
                serde_json::from_str(&response.result.unwrap_or_default())?;
            Ok(encrypted.ciphertext)
        } else {
            Err(ClientError(response.error.unwrap_or_else(|| "Unknown error".into())))
        }
    }

    /// NIP-44 decrypt
    pub async fn nip44_decrypt(
        &self,
        ciphertext: &str,
        sender_pubkey: &str,
        key_id: Option<&str>,
    ) -> Result<String, ClientError> {
        let proxy = Proxy::new(
            &self.connection,
            "com.plebsigner.Signer",
            "/com/plebsigner/Signer",
            "com.plebsigner.Signer1",
        )
        .await?;

        let key_id_str = key_id.unwrap_or("");
        let result: String = proxy
            .call(
                "Nip44Decrypt",
                &(ciphertext, sender_pubkey, key_id_str, &self.app_id),
            )
            .await?;

        let response: SignerResponse = serde_json::from_str(&result)?;
        if response.success {
            let decrypted: DecryptResult =
                serde_json::from_str(&response.result.unwrap_or_default())?;
            Ok(decrypted.plaintext)
        } else {
            Err(ClientError(response.error.unwrap_or_else(|| "Unknown error".into())))
        }
    }

    /// Start the bunker listener and get the connection URI
    pub async fn start_bunker(&self) -> Result<String, ClientError> {
        let proxy = Proxy::new(
            &self.connection,
            "com.plebsigner.Signer",
            "/com/plebsigner/Signer",
            "com.plebsigner.Signer1",
        )
        .await?;

        let result: String = proxy.call("StartBunker", &()).await?;
        let response: SignerResponse = serde_json::from_str(&result)?;
        if response.success {
            // The result is JSON-escaped, need to unescape
            let uri = response.result.unwrap_or_default();
            // Remove surrounding quotes if present
            let uri = uri.trim_matches('"').to_string();
            Ok(uri)
        } else {
            Err(ClientError(response.error.unwrap_or_else(|| "Unknown error".into())))
        }
    }

    /// Stop the bunker listener
    pub async fn stop_bunker(&self) -> Result<(), ClientError> {
        let proxy = Proxy::new(
            &self.connection,
            "com.plebsigner.Signer",
            "/com/plebsigner/Signer",
            "com.plebsigner.Signer1",
        )
        .await?;

        let result: String = proxy.call("StopBunker", &()).await?;
        let response: SignerResponse = serde_json::from_str(&result)?;
        if response.success {
            Ok(())
        } else {
            Err(ClientError(response.error.unwrap_or_else(|| "Unknown error".into())))
        }
    }

    /// Get the current bunker state
    pub async fn get_bunker_state(&self) -> Result<String, ClientError> {
        let proxy = Proxy::new(
            &self.connection,
            "com.plebsigner.Signer",
            "/com/plebsigner/Signer",
            "com.plebsigner.Signer1",
        )
        .await?;

        let result: String = proxy.call("GetBunkerState", &()).await?;
        let response: SignerResponse = serde_json::from_str(&result)?;
        if response.success {
            let state = response.result.unwrap_or_default();
            let state = state.trim_matches('"').to_string();
            Ok(state)
        } else {
            Err(ClientError(response.error.unwrap_or_else(|| "Unknown error".into())))
        }
    }

    /// Get the bunker URI (without starting)
    pub async fn get_bunker_uri(&self) -> Result<String, ClientError> {
        let proxy = Proxy::new(
            &self.connection,
            "com.plebsigner.Signer",
            "/com/plebsigner/Signer",
            "com.plebsigner.Signer1",
        )
        .await?;

        let result: String = proxy.call("GetBunkerUri", &()).await?;
        let response: SignerResponse = serde_json::from_str(&result)?;
        if response.success {
            let uri = response.result.unwrap_or_default();
            let uri = uri.trim_matches('"').to_string();
            Ok(uri)
        } else {
            Err(ClientError(response.error.unwrap_or_else(|| "Unknown error".into())))
        }
    }
}
