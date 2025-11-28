//! D-Bus interface for NIP-55 compatible signing on Linux
//!
//! This module provides a D-Bus service that allows other applications
//! to request signing operations, similar to how Android apps use intents.

use crate::app::AppState;
use crate::error::{Result, SignerError};
use crate::keys::KeyManager;
use crate::signing::{SigningEngine, UnsignedEventData};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::info;
use zbus::{interface, ConnectionBuilder};

/// D-Bus service name
pub const DBUS_NAME: &str = "com.plebsigner.Signer";

/// D-Bus object path
pub const DBUS_PATH: &str = "/com/plebsigner/Signer";

/// Response structure for D-Bus calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbusResponse {
    pub success: bool,
    pub id: String,
    #[serde(default)]
    pub result: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
}

impl DbusResponse {
    fn success(id: String, result: impl Serialize) -> String {
        serde_json::to_string(&DbusResponse {
            success: true,
            id,
            result: Some(serde_json::to_string(&result).unwrap_or_default()),
            error: None,
        }).unwrap_or_default()
    }

    fn error(id: String, error: impl ToString) -> String {
        serde_json::to_string(&DbusResponse {
            success: false,
            id,
            result: None,
            error: Some(error.to_string()),
        }).unwrap_or_default()
    }
}

/// The D-Bus interface implementation
pub struct SignerInterface {
    app_state: Arc<RwLock<AppState>>,
    signing_engine: Arc<SigningEngine>,
}

impl SignerInterface {
    pub fn new(app_state: Arc<RwLock<AppState>>, key_manager: Arc<Mutex<KeyManager>>) -> Self {
        Self {
            app_state,
            signing_engine: Arc::new(SigningEngine::new(key_manager)),
        }
    }

    fn generate_request_id() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        format!("req_{:x}", ts)
    }

    async fn check_ready(&self) -> std::result::Result<(), String> {
        let state = self.app_state.read().await;
        if state.is_locked {
            Err("Signer is locked".into())
        } else {
            Ok(())
        }
    }
}

#[interface(name = "com.plebsigner.Signer1")]
impl SignerInterface {
    /// Get the version of the signer
    async fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    /// Check if the signer is unlocked and ready
    async fn is_ready(&self) -> bool {
        let state = self.app_state.read().await;
        !state.is_locked
    }

    /// Get the active public key
    async fn get_public_key(&self) -> String {
        let id = Self::generate_request_id();
        
        if let Err(e) = self.check_ready().await {
            return DbusResponse::error(id, e);
        }

        match self.signing_engine.get_public_key().await {
            Ok(result) => DbusResponse::success(id, result),
            Err(e) => DbusResponse::error(id, e),
        }
    }

    /// List all available keys (returns public info only)
    async fn list_keys(&self) -> String {
        let state = self.app_state.read().await;
        let keys: Vec<_> = state.key_manager.list_keys()
            .iter()
            .map(|k| serde_json::json!({
                "name": k.name,
                "npub": k.npub,
                "pubkey_hex": k.pubkey_hex,
                "is_active": k.is_active,
            }))
            .collect();
        serde_json::to_string(&keys).unwrap_or_default()
    }

    /// Sign a Nostr event
    async fn sign_event(&self, event_json: &str, _app_id: &str) -> String {
        let id = Self::generate_request_id();
        
        if let Err(e) = self.check_ready().await {
            return DbusResponse::error(id, e);
        }

        let event_data: UnsignedEventData = match serde_json::from_str(event_json) {
            Ok(e) => e,
            Err(e) => return DbusResponse::error(id, format!("Invalid event: {}", e)),
        };

        match self.signing_engine.sign_event(&event_data).await {
            Ok(result) => DbusResponse::success(id, result),
            Err(e) => DbusResponse::error(id, e),
        }
    }

    /// NIP-04 encrypt
    async fn nip04_encrypt(&self, plaintext: &str, recipient_pubkey: &str, _app_id: &str) -> String {
        let id = Self::generate_request_id();
        
        if let Err(e) = self.check_ready().await {
            return DbusResponse::error(id, e);
        }

        match self.signing_engine.nip04_encrypt(recipient_pubkey, plaintext).await {
            Ok(result) => DbusResponse::success(id, result),
            Err(e) => DbusResponse::error(id, e),
        }
    }

    /// NIP-04 decrypt
    async fn nip04_decrypt(&self, ciphertext: &str, sender_pubkey: &str, _app_id: &str) -> String {
        let id = Self::generate_request_id();
        
        if let Err(e) = self.check_ready().await {
            return DbusResponse::error(id, e);
        }

        match self.signing_engine.nip04_decrypt(sender_pubkey, ciphertext).await {
            Ok(result) => DbusResponse::success(id, result),
            Err(e) => DbusResponse::error(id, e),
        }
    }

    /// NIP-44 encrypt
    async fn nip44_encrypt(&self, plaintext: &str, recipient_pubkey: &str, _app_id: &str) -> String {
        let id = Self::generate_request_id();
        
        if let Err(e) = self.check_ready().await {
            return DbusResponse::error(id, e);
        }

        match self.signing_engine.nip44_encrypt(recipient_pubkey, plaintext).await {
            Ok(result) => DbusResponse::success(id, result),
            Err(e) => DbusResponse::error(id, e),
        }
    }

    /// NIP-44 decrypt
    async fn nip44_decrypt(&self, ciphertext: &str, sender_pubkey: &str, _app_id: &str) -> String {
        let id = Self::generate_request_id();
        
        if let Err(e) = self.check_ready().await {
            return DbusResponse::error(id, e);
        }

        match self.signing_engine.nip44_decrypt(sender_pubkey, ciphertext).await {
            Ok(result) => DbusResponse::success(id, result),
            Err(e) => DbusResponse::error(id, e),
        }
    }

    /// Decrypt a zap event
    async fn decrypt_zap_event(&self, event_json: &str, _app_id: &str) -> String {
        let id = Self::generate_request_id();
        
        if let Err(e) = self.check_ready().await {
            return DbusResponse::error(id, e);
        }

        match self.signing_engine.decrypt_zap_event(event_json).await {
            Ok(result) => DbusResponse::success(id, result),
            Err(e) => DbusResponse::error(id, e),
        }
    }
}

/// D-Bus service runner
pub struct SignerService;

impl SignerService {
    pub async fn run(app_state: Arc<RwLock<AppState>>, key_manager: Arc<Mutex<KeyManager>>) -> Result<()> {
        let interface = SignerInterface::new(app_state, key_manager);

        let _connection = ConnectionBuilder::session()
            .map_err(|e| SignerError::DbusError(e.to_string()))?
            .name(DBUS_NAME)
            .map_err(|e| SignerError::DbusError(e.to_string()))?
            .serve_at(DBUS_PATH, interface)
            .map_err(|e| SignerError::DbusError(e.to_string()))?
            .build()
            .await
            .map_err(|e| SignerError::DbusError(e.to_string()))?;

        info!("D-Bus service started at {} on {}", DBUS_PATH, DBUS_NAME);

        // Keep the connection alive
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        }
    }
}
