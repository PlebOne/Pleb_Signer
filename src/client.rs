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

/// Pleb Signer client
pub struct PlebSignerClient {
    connection: Connection,
    app_id: String,
}

impl PlebSignerClient {
    /// Create a new client with the given application ID
    pub async fn new(app_id: &str) -> Result<Self, Box<dyn std::error::Error>> {
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
    pub async fn is_ready(&self) -> Result<bool, Box<dyn std::error::Error>> {
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
    pub async fn version(&self) -> Result<String, Box<dyn std::error::Error>> {
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
    pub async fn list_keys(&self) -> Result<Vec<KeyInfo>, Box<dyn std::error::Error>> {
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
    ) -> Result<PublicKeyResult, Box<dyn std::error::Error>> {
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
            Err(response.error.unwrap_or("Unknown error".into()).into())
        }
    }

    /// Sign an event
    pub async fn sign_event(
        &self,
        event_json: &str,
        key_id: Option<&str>,
    ) -> Result<SignedEventResult, Box<dyn std::error::Error>> {
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
            Err(response.error.unwrap_or("Unknown error".into()).into())
        }
    }

    /// NIP-04 encrypt
    pub async fn nip04_encrypt(
        &self,
        plaintext: &str,
        recipient_pubkey: &str,
        key_id: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error>> {
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
            Err(response.error.unwrap_or("Unknown error".into()).into())
        }
    }

    /// NIP-04 decrypt
    pub async fn nip04_decrypt(
        &self,
        ciphertext: &str,
        sender_pubkey: &str,
        key_id: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error>> {
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
            Err(response.error.unwrap_or("Unknown error".into()).into())
        }
    }

    /// NIP-44 encrypt
    pub async fn nip44_encrypt(
        &self,
        plaintext: &str,
        recipient_pubkey: &str,
        key_id: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error>> {
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
            Err(response.error.unwrap_or("Unknown error".into()).into())
        }
    }

    /// NIP-44 decrypt
    pub async fn nip44_decrypt(
        &self,
        ciphertext: &str,
        sender_pubkey: &str,
        key_id: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error>> {
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
            Err(response.error.unwrap_or("Unknown error".into()).into())
        }
    }
}
