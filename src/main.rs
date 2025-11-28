//! Pleb Signer - A Linux NIP-55 Nostr Signer Application
//! 
//! This is the main entry point for the Pleb Signer application.
//! It provides secure key management and event signing for Nostr clients.

mod app;
mod bunker;
pub mod client;
mod config;
mod dbus;
mod error;
mod keys;
mod permissions;
mod signing;
mod tray;
mod ui;

use anyhow::Result;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tokio::sync::{Mutex, RwLock};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::app::AppState;
use crate::config::Config;
use crate::dbus::SignerService;
use crate::keys::KeyManager;

fn main() -> Result<()> {
    // Check if we're being run in UI-only mode (spawned by tray)
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--ui-only" {
        return run_ui_only();
    }

    // Initialize logging
    FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .compact()
        .init();

    info!("Starting Pleb Signer v{}", env!("CARGO_PKG_VERSION"));

    // Create a tokio runtime for async operations (D-Bus, keyring)
    let runtime = tokio::runtime::Runtime::new()?;
    
    // Load configuration and initialize state in the runtime
    let (_config, _key_manager, app_state) = runtime.block_on(async {
        let config = Config::load().await?;
        info!("Configuration loaded");

        // Create shared key manager
        let key_manager = Arc::new(Mutex::new(KeyManager::new()));
        
        // Load key metadata
        {
            let mut km = key_manager.lock().await;
            if let Err(e) = km.load().await {
                tracing::warn!("Failed to load key metadata: {}", e);
            }
        }

        // Initialize application state
        let app_state = Arc::new(RwLock::new(AppState::new(config.clone()).await?));
        
        Ok::<_, anyhow::Error>((config, key_manager, app_state))
    })?;

    // Clone for D-Bus service - IMPORTANT: load keys for D-Bus too
    let dbus_state = Arc::clone(&app_state);
    let dbus_km = Arc::new(Mutex::new(KeyManager::new()));
    
    // Load keys for D-Bus KeyManager
    let dbus_km_init = Arc::clone(&dbus_km);
    runtime.block_on(async {
        let mut km = dbus_km_init.lock().await;
        if let Err(e) = km.load().await {
            tracing::warn!("Failed to load keys for D-Bus service: {}", e);
        }
    });

    // Start D-Bus service in background on the runtime
    runtime.spawn(async move {
        if let Err(e) = SignerService::run(dbus_state, dbus_km).await {
            tracing::error!("D-Bus service error: {}", e);
        }
    });

    // Start system tray (runs in its own thread)
    let tray_state = tray::start_tray();
    info!("System tray initialized");

    // Show the UI window initially (spawn as subprocess)
    spawn_ui_window();

    // Main loop - tray controls the lifecycle
    loop {
        // Check if quit was requested
        if tray_state.quit_requested.load(Ordering::Relaxed) {
            info!("Quit requested, shutting down...");
            break;
        }

        // Check if window should be shown
        if tray_state.show_requested.swap(false, Ordering::Relaxed) {
            info!("Spawning UI window...");
            spawn_ui_window();
        }

        // Sleep a bit before checking again
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    info!("Pleb Signer shutting down");
    Ok(())
}

/// Spawn the UI window as a separate process
fn spawn_ui_window() {
    let exe = std::env::current_exe().unwrap_or_else(|_| "pleb-signer".into());
    
    match std::process::Command::new(&exe)
        .arg("--ui-only")
        .spawn()
    {
        Ok(_child) => {
            info!("UI window process spawned");
        }
        Err(e) => {
            tracing::error!("Failed to spawn UI window: {}", e);
        }
    }
}

/// Run only the UI (called when spawned with --ui-only)
fn run_ui_only() -> Result<()> {
    // Minimal logging for UI subprocess
    FmtSubscriber::builder()
        .with_max_level(Level::WARN)
        .with_target(false)
        .compact()
        .init();

    // Create runtime just for loading config/keys
    let runtime = tokio::runtime::Runtime::new()?;
    
    let (config, key_manager) = runtime.block_on(async {
        let config = Config::load().await?;
        let key_manager = Arc::new(tokio::sync::Mutex::new(KeyManager::new()));
        
        {
            let mut km = key_manager.lock().await;
            if let Err(e) = km.load().await {
                tracing::warn!("Failed to load key metadata: {}", e);
            }
        }
        
        Ok::<_, anyhow::Error>((config, key_manager))
    })?;

    // Drop the runtime before starting iced (iced creates its own)
    drop(runtime);

    // Run the UI - when window closes, this process exits
    ui::run_ui(key_manager, config)?;
    
    Ok(())
}
