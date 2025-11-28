//! Configuration management for Pleb Signer

use crate::error::{Result, SignerError};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use directories::ProjectDirs;
use tokio::fs;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Path to the configuration file
    #[serde(skip)]
    config_path: PathBuf,

    /// General settings
    #[serde(default)]
    pub general: GeneralConfig,

    /// Security settings
    #[serde(default)]
    pub security: SecurityConfig,

    /// UI settings
    #[serde(default)]
    pub ui: UiConfig,

    /// List of authorized applications
    #[serde(default)]
    pub authorized_apps: Vec<AuthorizedApp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Start minimized to tray
    #[serde(default = "default_true")]
    pub start_minimized: bool,

    /// Auto-start on login
    #[serde(default)]
    pub auto_start: bool,

    /// Show notifications for signing requests
    #[serde(default = "default_true")]
    pub show_notifications: bool,

    /// Default timeout for signing requests (seconds)
    #[serde(default = "default_timeout")]
    pub request_timeout_secs: u64,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            start_minimized: true,
            auto_start: false,
            show_notifications: true,
            request_timeout_secs: 60,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Require password on startup
    #[serde(default = "default_true")]
    pub require_password_on_start: bool,

    /// Lock after inactivity (minutes, 0 = never)
    #[serde(default = "default_lock_timeout")]
    pub lock_timeout_mins: u64,

    /// Require confirmation for all signing requests
    #[serde(default = "default_true")]
    pub always_confirm: bool,

    /// Allow auto-approval for trusted apps and specific event kinds
    #[serde(default)]
    pub allow_auto_approve: bool,

    /// Maximum number of auto-approvals per minute (rate limiting)
    #[serde(default = "default_rate_limit")]
    pub max_auto_approvals_per_min: u32,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            require_password_on_start: true,
            lock_timeout_mins: 15,
            always_confirm: true,
            allow_auto_approve: false,
            max_auto_approvals_per_min: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Theme (light, dark, system)
    #[serde(default = "default_theme")]
    pub theme: String,

    /// Show event content in approval dialog
    #[serde(default = "default_true")]
    pub show_event_content: bool,

    /// Compact mode for approval dialogs
    #[serde(default)]
    pub compact_mode: bool,

    /// Window opacity (0.0-1.0)
    #[serde(default = "default_opacity")]
    pub window_opacity: f32,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            show_event_content: true,
            compact_mode: false,
            window_opacity: 1.0,
        }
    }
}

/// Represents an authorized application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizedApp {
    /// Application identifier (process name or D-Bus sender)
    pub app_id: String,

    /// Human-readable name
    pub name: String,

    /// When the app was first authorized
    pub authorized_at: chrono::DateTime<chrono::Utc>,

    /// Permissions granted to this app
    pub permissions: AppPermissions,

    /// Whether auto-approval is enabled for this app
    pub auto_approve: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppPermissions {
    /// Can request public key
    pub get_public_key: bool,

    /// Can sign events (optionally restricted to specific kinds)
    pub sign_event: Option<Vec<u16>>,  // None = all kinds, Some([]) = none, Some([1,4]) = specific

    /// Can use NIP-04 encryption
    pub nip04_encrypt: bool,
    pub nip04_decrypt: bool,

    /// Can use NIP-44 encryption
    pub nip44_encrypt: bool,
    pub nip44_decrypt: bool,

    /// Can decrypt zap events
    pub decrypt_zap_event: bool,
}

impl Config {
    /// Create a default configuration (for use before async loading)
    pub fn default_config() -> Self {
        Self {
            config_path: PathBuf::new(),
            general: GeneralConfig::default(),
            security: SecurityConfig::default(),
            ui: UiConfig::default(),
            authorized_apps: Vec::new(),
        }
    }

    /// Load configuration from disk, creating default if not exists
    pub async fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;

        if config_path.exists() {
            let content = fs::read_to_string(&config_path).await?;
            let mut config: Config = toml::from_str(&content)
                .map_err(|e| SignerError::ConfigError(e.to_string()))?;
            config.config_path = config_path;
            Ok(config)
        } else {
            // Create default configuration
            let config = Config {
                config_path: config_path.clone(),
                general: GeneralConfig::default(),
                security: SecurityConfig::default(),
                ui: UiConfig::default(),
                authorized_apps: Vec::new(),
            };
            config.save().await?;
            Ok(config)
        }
    }

    /// Save configuration to disk
    pub async fn save(&self) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| SignerError::ConfigError(e.to_string()))?;
        fs::write(&self.config_path, content).await?;
        Ok(())
    }

    /// Get the configuration file path
    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    /// Get the data directory path
    pub fn data_dir() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "plebsigner", "PlebSigner")
            .ok_or_else(|| SignerError::ConfigError("Could not determine config directory".into()))?;
        Ok(proj_dirs.data_dir().to_path_buf())
    }

    /// Get the keys file path
    pub fn keys_path() -> Result<PathBuf> {
        Ok(Self::data_dir()?.join("keys.enc"))
    }

    fn get_config_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "plebsigner", "PlebSigner")
            .ok_or_else(|| SignerError::ConfigError("Could not determine config directory".into()))?;
        Ok(proj_dirs.config_dir().join("config.toml"))
    }

    /// Add or update an authorized application
    pub fn authorize_app(&mut self, app: AuthorizedApp) {
        if let Some(existing) = self.authorized_apps.iter_mut().find(|a| a.app_id == app.app_id) {
            *existing = app;
        } else {
            self.authorized_apps.push(app);
        }
    }

    /// Remove an authorized application
    pub fn revoke_app(&mut self, app_id: &str) {
        self.authorized_apps.retain(|a| a.app_id != app_id);
    }

    /// Check if an app is authorized
    pub fn is_app_authorized(&self, app_id: &str) -> bool {
        self.authorized_apps.iter().any(|a| a.app_id == app_id)
    }

    /// Get an authorized app by ID
    pub fn get_authorized_app(&self, app_id: &str) -> Option<&AuthorizedApp> {
        self.authorized_apps.iter().find(|a| a.app_id == app_id)
    }
}

// Default value helpers
fn default_true() -> bool { true }
fn default_timeout() -> u64 { 60 }
fn default_lock_timeout() -> u64 { 15 }
fn default_rate_limit() -> u32 { 10 }
fn default_theme() -> String { "system".to_string() }
fn default_opacity() -> f32 { 1.0 }
