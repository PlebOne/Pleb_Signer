//! Application state management

use crate::bunker::{BunkerSigner, BunkerState};
use crate::config::Config;
use crate::error::Result;
use crate::keys::KeyManager;
use crate::permissions::RateLimiter;
use async_channel::{Receiver, Sender};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Message types for communication between components
#[derive(Debug, Clone)]
pub enum AppMessage {
    /// Lock the application
    Lock,
    /// Unlock the application
    Unlock,
    /// Show the main window
    ShowWindow,
    /// Hide to tray
    HideToTray,
    /// Quit the application
    Quit,
}

/// Main application state
pub struct AppState {
    /// Application configuration
    pub config: Config,
    /// Key manager
    pub key_manager: KeyManager,
    /// Rate limiter for auto-approved requests
    pub rate_limiter: RateLimiter,
    /// Whether the application is currently locked
    pub is_locked: bool,
    /// Whether the main window is visible
    pub window_visible: bool,
    /// Channel for internal messages
    pub message_sender: Sender<AppMessage>,
    pub message_receiver: Receiver<AppMessage>,
    /// Bunker signer for NIP-46 remote signing
    pub bunker_signer: Option<Arc<BunkerSigner>>,
}

impl AppState {
    /// Create a new application state
    pub async fn new(config: Config) -> Result<Self> {
        let (message_sender, message_receiver) = async_channel::unbounded();
        let key_manager = KeyManager::new();
        let rate_limiter = RateLimiter::new(config.security.max_auto_approvals_per_min);

        Ok(Self {
            config,
            key_manager,
            rate_limiter,
            is_locked: false, // Start unlocked since we use OS keyring
            window_visible: true,
            message_sender,
            message_receiver,
            bunker_signer: None,
        })
    }
    
    /// Initialize bunker signer with key manager
    pub fn init_bunker(&mut self, key_manager: Arc<Mutex<KeyManager>>) {
        let bunker = BunkerSigner::new(key_manager)
            .with_relays(vec![
                "wss://relay.nsec.app".to_string(),
                "wss://relay.damus.io".to_string(),
            ]);
        self.bunker_signer = Some(Arc::new(bunker));
    }
    
    /// Start bunker listener and return connection URI
    pub async fn start_bunker(&self) -> Result<String> {
        if let Some(ref bunker) = self.bunker_signer {
            // Generate connection URI first
            let uri = bunker.generate_bunker_uri().await?;
            
            // Start listening for connections
            bunker.start_listening().await?;
            
            Ok(uri)
        } else {
            Err(crate::error::SignerError::NostrError("Bunker not initialized".into()))
        }
    }
    
    /// Stop bunker listener
    pub async fn stop_bunker(&self) {
        if let Some(ref bunker) = self.bunker_signer {
            bunker.stop().await;
        }
    }
    
    /// Get bunker connection URI
    pub async fn get_bunker_uri(&self) -> Result<String> {
        if let Some(ref bunker) = self.bunker_signer {
            bunker.generate_bunker_uri().await
        } else {
            Err(crate::error::SignerError::NostrError("Bunker not initialized".into()))
        }
    }
    
    /// Get bunker state
    pub async fn get_bunker_state(&self) -> BunkerState {
        if let Some(ref bunker) = self.bunker_signer {
            bunker.state().await
        } else {
            BunkerState::Disconnected
        }
    }

    /// Check if application is ready
    pub fn is_ready(&self) -> bool {
        !self.is_locked
    }

    /// Get the message sender for cloning
    pub fn get_message_sender(&self) -> Sender<AppMessage> {
        self.message_sender.clone()
    }
}
