//! Signing operations for Pleb Signer
//!
//! Uses the NostrSigner trait from the nostr crate.

use crate::error::{Result, SignerError};
use crate::keys::KeyManager;
use crate::permissions::RequestType;
use nostr::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Data for an unsigned event (simplified for serialization)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsignedEventData {
    pub kind: u16,
    pub content: String,
    #[serde(default)]
    pub tags: Vec<Vec<String>>,
    #[serde(default)]
    pub created_at: Option<u64>,
}

/// Payload for signing requests
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SigningPayload {
    /// Empty payload (for get_public_key)
    Empty,
    /// Event to sign
    Event(UnsignedEventData),
    /// Data to encrypt
    Encrypt {
        plaintext: String,
        recipient_pubkey: String,
    },
    /// Data to decrypt
    Decrypt {
        ciphertext: String,
        sender_pubkey: String,
    },
    /// Zap event to decrypt
    ZapEvent(String),
}

/// A signing request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningRequest {
    /// Unique request ID
    pub id: String,
    /// Type of request
    pub request_type: RequestType,
    /// Requesting application ID
    pub app_id: String,
    /// Requesting application name (if known)
    #[serde(default)]
    pub app_name: Option<String>,
    /// Specific key to use (if any)
    #[serde(default)]
    pub key_id: Option<String>,
    /// Request payload
    pub payload: SigningPayload,
    /// When the request was made
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Result data from signing operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SigningResultData {
    /// Public key result
    PublicKey { npub: String, hex: String },
    /// Signed event
    Event { event_json: String, signature: String },
    /// Encrypted data
    Encrypted { ciphertext: String },
    /// Decrypted data
    Decrypted { plaintext: String },
}

/// Result of a signing operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningResult {
    pub request_id: String,
    pub approved: bool,
    #[serde(default)]
    pub result: Option<SigningResultData>,
    #[serde(default)]
    pub error: Option<String>,
}

/// Signing engine that wraps key management with signing operations
pub struct SigningEngine {
    key_manager: Arc<Mutex<KeyManager>>,
}

impl SigningEngine {
    /// Create a new signing engine
    pub fn new(key_manager: Arc<Mutex<KeyManager>>) -> Self {
        Self { key_manager }
    }

    /// Get the public key
    pub async fn get_public_key(&self) -> Result<SigningResultData> {
        let mut km = self.key_manager.lock().await;
        let keys = km.get_signing_keys().await?;
        let pubkey = keys.public_key();
        
        Ok(SigningResultData::PublicKey {
            npub: pubkey.to_bech32().unwrap_or_default(),
            hex: pubkey.to_hex(),
        })
    }

    /// Sign an unsigned event from data
    pub async fn sign_event(&self, event_data: &UnsignedEventData) -> Result<SigningResultData> {
        let mut km = self.key_manager.lock().await;
        let keys = km.get_signing_keys().await?;
        
        // Build the event
        let kind = Kind::from(event_data.kind);
        let created_at = event_data.created_at
            .map(Timestamp::from)
            .unwrap_or_else(Timestamp::now);
        
        let mut builder = EventBuilder::new(kind, &event_data.content);
        
        // Add tags
        for tag_data in &event_data.tags {
            if !tag_data.is_empty() {
                let tag = Tag::parse(tag_data)
                    .map_err(|e| SignerError::InvalidRequest(e.to_string()))?;
                builder = builder.tag(tag);
            }
        }
        
        let event = builder
            .custom_created_at(created_at)
            .sign_with_keys(keys)
            .map_err(|e| SignerError::NostrError(e.to_string()))?;
        
        Ok(SigningResultData::Event {
            event_json: event.as_json(),
            signature: event.sig.to_string(),
        })
    }

    /// NIP-04 encrypt
    pub async fn nip04_encrypt(&self, recipient_pubkey: &str, plaintext: &str) -> Result<SigningResultData> {
        let mut km = self.key_manager.lock().await;
        let keys = km.get_signing_keys().await?;
        
        let pubkey = PublicKey::parse(recipient_pubkey)
            .map_err(|e| SignerError::InvalidRequest(e.to_string()))?;
        
        let ciphertext = nip04::encrypt(keys.secret_key(), &pubkey, plaintext)
            .map_err(|e| SignerError::EncryptionError(e.to_string()))?;
        
        Ok(SigningResultData::Encrypted { ciphertext })
    }

    /// NIP-04 decrypt
    pub async fn nip04_decrypt(&self, sender_pubkey: &str, ciphertext: &str) -> Result<SigningResultData> {
        let mut km = self.key_manager.lock().await;
        let keys = km.get_signing_keys().await?;
        
        let pubkey = PublicKey::parse(sender_pubkey)
            .map_err(|e| SignerError::InvalidRequest(e.to_string()))?;
        
        let plaintext = nip04::decrypt(keys.secret_key(), &pubkey, ciphertext)
            .map_err(|e| SignerError::DecryptionError(e.to_string()))?;
        
        Ok(SigningResultData::Decrypted { plaintext })
    }

    /// NIP-44 encrypt
    pub async fn nip44_encrypt(&self, recipient_pubkey: &str, plaintext: &str) -> Result<SigningResultData> {
        let mut km = self.key_manager.lock().await;
        let keys = km.get_signing_keys().await?;
        
        let pubkey = PublicKey::parse(recipient_pubkey)
            .map_err(|e| SignerError::InvalidRequest(e.to_string()))?;
        
        let ciphertext = nip44::encrypt(keys.secret_key(), &pubkey, plaintext, nip44::Version::default())
            .map_err(|e| SignerError::EncryptionError(e.to_string()))?;
        
        Ok(SigningResultData::Encrypted { ciphertext })
    }

    /// NIP-44 decrypt
    pub async fn nip44_decrypt(&self, sender_pubkey: &str, ciphertext: &str) -> Result<SigningResultData> {
        let mut km = self.key_manager.lock().await;
        let keys = km.get_signing_keys().await?;
        
        let pubkey = PublicKey::parse(sender_pubkey)
            .map_err(|e| SignerError::InvalidRequest(e.to_string()))?;
        
        let plaintext = nip44::decrypt(keys.secret_key(), &pubkey, ciphertext)
            .map_err(|e| SignerError::DecryptionError(e.to_string()))?;
        
        Ok(SigningResultData::Decrypted { plaintext })
    }

    /// Decrypt a zap event (NIP-57)
    pub async fn decrypt_zap_event(&self, event_json: &str) -> Result<SigningResultData> {
        let event: Event = Event::from_json(event_json)
            .map_err(|e| SignerError::InvalidRequest(e.to_string()))?;
        
        // Get the sender's public key from p tag
        let sender_pubkey = event.tags.public_keys()
            .next()
            .ok_or_else(|| SignerError::InvalidRequest("No sender pubkey in zap event".into()))?;
        
        let mut km = self.key_manager.lock().await;
        let keys = km.get_signing_keys().await?;
        
        // Decrypt the content
        let plaintext = nip04::decrypt(keys.secret_key(), sender_pubkey, &event.content)
            .map_err(|e| SignerError::DecryptionError(e.to_string()))?;
        
        Ok(SigningResultData::Decrypted { plaintext })
    }
}
