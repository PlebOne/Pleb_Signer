//! Application state management

use crate::config::Config;
use crate::error::Result;
use crate::keys::KeyManager;
use crate::permissions::RateLimiter;
use async_channel::{Receiver, Sender};

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
        })
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
