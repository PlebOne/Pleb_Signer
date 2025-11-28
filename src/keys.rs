//! Key management for Pleb Signer using nostr-keyring
//!
//! Uses the OS keyring (Secret Service on Linux) for secure key storage.

use crate::error::{Result, SignerError};
use nostr::prelude::*;
use nostr_keyring::NostrKeyring;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

const KEYRING_SERVICE: &str = "pleb-signer";
const METADATA_FILE: &str = "keys_metadata.json";

/// Metadata about a stored key (public info only)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetadata {
    /// Unique name/label for this key
    pub name: String,
    /// Public key in npub format
    pub npub: String,
    /// Public key in hex format
    pub pubkey_hex: String,
    /// When this key was added
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Whether this is the active/default key
    pub is_active: bool,
}

/// Stored key metadata (persisted to disk)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeysMetadata {
    /// Map of key name to metadata
    pub keys: HashMap<String, KeyMetadata>,
    /// Currently active key name
    pub active_key: Option<String>,
}

impl KeysMetadata {
    fn path() -> Result<PathBuf> {
        let proj_dirs = directories::ProjectDirs::from("com", "plebsigner", "PlebSigner")
            .ok_or_else(|| SignerError::ConfigError("Could not determine data directory".into()))?;
        Ok(proj_dirs.data_dir().join(METADATA_FILE))
    }

    pub async fn load() -> Result<Self> {
        let path = Self::path()?;
        if path.exists() {
            let content = fs::read_to_string(&path).await?;
            let metadata: KeysMetadata = serde_json::from_str(&content)?;
            Ok(metadata)
        } else {
            Ok(KeysMetadata::default())
        }
    }

    pub async fn save(&self) -> Result<()> {
        let path = Self::path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content).await?;
        Ok(())
    }
}

/// Key manager using nostr-keyring for secure storage
pub struct KeyManager {
    keyring: NostrKeyring,
    metadata: KeysMetadata,
    /// Cached active keys (loaded from keyring when unlocked)
    cached_keys: Option<Keys>,
}

impl KeyManager {
    /// Create a new key manager
    pub fn new() -> Self {
        Self {
            keyring: NostrKeyring::new(KEYRING_SERVICE),
            metadata: KeysMetadata::default(),
            cached_keys: None,
        }
    }

    /// Load metadata from disk
    pub async fn load(&mut self) -> Result<()> {
        self.metadata = KeysMetadata::load().await?;
        Ok(())
    }

    /// Check if any keys exist
    pub fn has_keys(&self) -> bool {
        !self.metadata.keys.is_empty()
    }

    /// Get list of all keys (metadata only)
    pub fn list_keys(&self) -> Vec<&KeyMetadata> {
        self.metadata.keys.values().collect()
    }

    /// Get the active key's public key
    pub fn get_active_pubkey(&self) -> Option<&str> {
        self.metadata.active_key.as_ref()
            .and_then(|name| self.metadata.keys.get(name))
            .map(|m| m.npub.as_str())
    }

    /// Get the active key name
    pub fn get_active_key_name(&self) -> Option<&str> {
        self.metadata.active_key.as_deref()
    }

    /// Set the active key by name
    pub async fn set_active_key(&mut self, name: &str) -> Result<()> {
        if !self.metadata.keys.contains_key(name) {
            return Err(SignerError::KeyNotFound(name.to_string()));
        }

        // Update is_active flags
        for (key_name, meta) in &mut self.metadata.keys {
            meta.is_active = key_name == name;
        }
        self.metadata.active_key = Some(name.to_string());
        
        // Clear cached keys to force reload
        self.cached_keys = None;
        
        self.metadata.save().await?;
        Ok(())
    }

    /// Generate a new key and store it
    pub async fn generate_key(&mut self, name: &str) -> Result<KeyMetadata> {
        if self.metadata.keys.contains_key(name) {
            return Err(SignerError::KeyAlreadyExists(name.to_string()));
        }

        let keys = Keys::generate();
        self.store_key(name, &keys).await
    }

    /// Import a key from nsec or hex
    pub async fn import_key(&mut self, name: &str, secret: &str) -> Result<KeyMetadata> {
        if self.metadata.keys.contains_key(name) {
            return Err(SignerError::KeyAlreadyExists(name.to_string()));
        }

        let keys = Keys::parse(secret)
            .map_err(|e| SignerError::InvalidKeyFormat(e.to_string()))?;
        
        self.store_key(name, &keys).await
    }

    /// Import a key from mnemonic (NIP-06)
    pub async fn import_from_mnemonic(
        &mut self,
        name: &str,
        mnemonic: &str,
        passphrase: Option<&str>,
    ) -> Result<KeyMetadata> {
        if self.metadata.keys.contains_key(name) {
            return Err(SignerError::KeyAlreadyExists(name.to_string()));
        }

        let keys = Keys::from_mnemonic(mnemonic, passphrase)
            .map_err(|e| SignerError::InvalidKeyFormat(e.to_string()))?;
        
        self.store_key(name, &keys).await
    }

    /// Store a key in the keyring
    async fn store_key(&mut self, name: &str, keys: &Keys) -> Result<KeyMetadata> {
        // Store in OS keyring
        self.keyring.set_async(name, keys).await
            .map_err(|e| SignerError::EncryptionError(e.to_string()))?;

        let public_key = keys.public_key();
        let metadata = KeyMetadata {
            name: name.to_string(),
            npub: public_key.to_bech32().unwrap_or_default(),
            pubkey_hex: public_key.to_hex(),
            created_at: chrono::Utc::now(),
            is_active: self.metadata.keys.is_empty(),
        };

        // Set as active if first key
        if self.metadata.keys.is_empty() {
            self.metadata.active_key = Some(name.to_string());
        }

        self.metadata.keys.insert(name.to_string(), metadata.clone());
        self.metadata.save().await?;

        Ok(metadata)
    }

    /// Delete a key
    pub async fn delete_key(&mut self, name: &str) -> Result<()> {
        if !self.metadata.keys.contains_key(name) {
            return Err(SignerError::KeyNotFound(name.to_string()));
        }

        // Remove from keyring
        self.keyring.delete_async(name).await
            .map_err(|e| SignerError::DecryptionError(e.to_string()))?;

        self.metadata.keys.remove(name);
        
        // Update active key if needed
        if self.metadata.active_key.as_deref() == Some(name) {
            self.metadata.active_key = self.metadata.keys.keys().next().cloned();
            self.cached_keys = None;
        }

        self.metadata.save().await?;
        Ok(())
    }

    /// Get the active signing keys
    pub async fn get_signing_keys(&mut self) -> Result<&Keys> {
        if self.cached_keys.is_some() {
            return Ok(self.cached_keys.as_ref().unwrap());
        }

        let name = self.metadata.active_key.as_ref()
            .ok_or(SignerError::NoKeysConfigured)?;

        let keys = self.keyring.get_async(name).await
            .map_err(|e| SignerError::DecryptionError(e.to_string()))?;
        
        self.cached_keys = Some(keys);
        Ok(self.cached_keys.as_ref().unwrap())
    }

    /// Get keys by name
    pub async fn get_keys_by_name(&self, name: &str) -> Result<Keys> {
        if !self.metadata.keys.contains_key(name) {
            return Err(SignerError::KeyNotFound(name.to_string()));
        }

        self.keyring.get_async(name).await
            .map_err(|e| SignerError::DecryptionError(e.to_string()))
    }

    /// Export key as nsec (bech32)
    pub async fn export_nsec(&self, name: &str) -> Result<String> {
        let keys = self.get_keys_by_name(name).await?;
        keys.secret_key().to_bech32()
            .map_err(|e| SignerError::NostrError(e.to_string()))
    }

    /// Export key as NIP-49 encrypted format (ncryptsec)
    pub async fn export_encrypted(&self, name: &str, password: &str) -> Result<String> {
        let keys = self.get_keys_by_name(name).await?;
        let encrypted = EncryptedSecretKey::new(
            keys.secret_key(),
            password,
            16, // log_n for scrypt
            KeySecurity::Medium,
        ).map_err(|e| SignerError::EncryptionError(e.to_string()))?;
        
        encrypted.to_bech32()
            .map_err(|e| SignerError::NostrError(e.to_string()))
    }

    /// Import from NIP-49 encrypted format
    pub async fn import_encrypted(&mut self, name: &str, ncryptsec: &str, password: &str) -> Result<KeyMetadata> {
        if self.metadata.keys.contains_key(name) {
            return Err(SignerError::KeyAlreadyExists(name.to_string()));
        }

        let encrypted = EncryptedSecretKey::from_bech32(ncryptsec)
            .map_err(|e| SignerError::InvalidKeyFormat(e.to_string()))?;
        
        let secret_key = encrypted.decrypt(password)
            .map_err(|_| SignerError::InvalidPassword)?;
        
        let keys = Keys::new(secret_key);
        self.store_key(name, &keys).await
    }

    /// Clear cached keys (for locking)
    pub fn lock(&mut self) {
        self.cached_keys = None;
    }

    /// Check if keys are cached (unlocked)
    pub fn is_unlocked(&self) -> bool {
        self.cached_keys.is_some()
    }
}

impl Default for KeyManager {
    fn default() -> Self {
        Self::new()
    }
}
