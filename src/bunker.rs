//! NIP-46 Nostr Connect (Bunker) support
//!
//! This module allows Pleb Signer to act as a remote signer via NIP-46,
//! enabling signing from any device that can connect to Nostr relays.

use crate::error::{Result, SignerError};
use crate::keys::KeyManager;
use nostr::prelude::*;
use nostr_sdk::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Mutex;
use tracing::{error, info, warn};

/// Bunker connection state
#[derive(Debug, Clone)]
pub enum BunkerState {
    /// Not connected
    Disconnected,
    /// Waiting for client connection
    WaitingForConnection { connection_string: String },
    /// Connected to a client
    Connected { client_pubkey: String, app_name: Option<String> },
    /// Error state
    Error(String),
}

/// NIP-46 Bunker signer that allows remote signing
pub struct BunkerSigner {
    key_manager: Arc<Mutex<KeyManager>>,
    state: Arc<Mutex<BunkerState>>,
    relays: Vec<String>,
    secret: Option<String>,
    /// Flag to signal the listener thread to stop
    stop_flag: Arc<AtomicBool>,
    /// Handle to the listener thread
    listener_handle: std::sync::Mutex<Option<std::thread::JoinHandle<()>>>,
}

impl BunkerSigner {
    /// Create a new bunker signer
    pub fn new(key_manager: Arc<Mutex<KeyManager>>) -> Self {
        Self {
            key_manager,
            state: Arc::new(Mutex::new(BunkerState::Disconnected)),
            relays: vec![
                "wss://relay.nsec.app".to_string(),
                "wss://relay.damus.io".to_string(),
            ],
            secret: None,
            stop_flag: Arc::new(AtomicBool::new(false)),
            listener_handle: std::sync::Mutex::new(None),
        }
    }

    /// Set custom relays for bunker connection
    pub fn with_relays(mut self, relays: Vec<String>) -> Self {
        self.relays = relays;
        self
    }

    /// Set a secret for the connection (optional additional security)
    pub fn with_secret(mut self, secret: String) -> Self {
        self.secret = Some(secret);
        self
    }

    /// Get current state
    pub async fn state(&self) -> BunkerState {
        self.state.lock().await.clone()
    }

    /// Generate a bunker:// URI for clients that support it
    pub async fn generate_bunker_uri(&self) -> Result<String> {
        let km = self.key_manager.lock().await;
        let pubkey = km.get_active_pubkey()
            .ok_or_else(|| SignerError::KeyNotFound("No active key".into()))?;
        
        let mut uri = format!("bunker://{}", pubkey);
        
        let mut params = Vec::new();
        for relay in &self.relays {
            params.push(format!("relay={}", urlencoding::encode(relay)));
        }
        
        if let Some(ref secret) = self.secret {
            params.push(format!("secret={}", urlencoding::encode(secret)));
        }
        
        if !params.is_empty() {
            uri.push('?');
            uri.push_str(&params.join("&"));
        }
        
        Ok(uri)
    }

    /// Start listening for bunker connections
    /// 
    /// This spawns a background THREAD (not tokio task) that handles incoming NIP-46 requests
    pub async fn start_listening(&self) -> Result<()> {
        info!("Starting bunker listener...");
        
        // Check if already running
        {
            let handle = self.listener_handle.lock().unwrap();
            if handle.is_some() {
                info!("Bunker listener already running");
                return Ok(());
            }
        }
        
        // Get the keys we need
        let mut km = self.key_manager.lock().await;
        let keys = km.get_signing_keys().await
            .map_err(|e| SignerError::NostrError(e.to_string()))?
            .clone();
        drop(km);
        
        // Update state
        {
            let mut state = self.state.lock().await;
            let pubkey = keys.public_key().to_hex();
            *state = BunkerState::WaitingForConnection {
                connection_string: format!("bunker://{}", pubkey),
            };
        }
        
        // Reset stop flag
        self.stop_flag.store(false, Ordering::SeqCst);
        
        // Clone what we need for the thread
        let state = Arc::clone(&self.state);
        let key_manager = Arc::clone(&self.key_manager);
        let relays = self.relays.clone();
        let stop_flag = Arc::clone(&self.stop_flag);
        
        // Spawn a real OS thread with its own tokio runtime
        let handle = std::thread::spawn(move || {
            info!("Bunker listener thread started");
            
            // Create a new tokio runtime for this thread
            let rt = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    error!("Failed to create tokio runtime: {}", e);
                    return;
                }
            };
            
            // Run the listener
            rt.block_on(async {
                if let Err(e) = run_bunker_listener(keys, relays, stop_flag, state, key_manager).await {
                    error!("Bunker listener error: {}", e);
                }
            });
            
            info!("Bunker listener thread exiting");
        });
        
        // Store the handle
        {
            let mut guard = self.listener_handle.lock().unwrap();
            *guard = Some(handle);
        }
        
        info!("Bunker signer started listening on {} relays", self.relays.len());
        Ok(())
    }

    /// Stop the bunker listener
    pub async fn stop(&self) {
        info!("Stopping bunker listener...");
        
        // Signal the thread to stop
        self.stop_flag.store(true, Ordering::SeqCst);
        
        // Update state
        {
            let mut state = self.state.lock().await;
            *state = BunkerState::Disconnected;
        }
        
        // Wait for thread to finish (with timeout)
        let handle = {
            let mut guard = self.listener_handle.lock().unwrap();
            guard.take()
        };
        
        if let Some(h) = handle {
            // Give it a moment to finish
            std::thread::sleep(std::time::Duration::from_millis(500));
            // We don't join because it might block, just let it finish
            drop(h);
        }
        
        info!("Bunker listener stopped");
    }
}

/// URL encoding helper
mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut result = String::new();
        for c in s.chars() {
            match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                    result.push(c);
                }
                _ => {
                    for byte in c.to_string().as_bytes() {
                        result.push_str(&format!("%{:02X}", byte));
                    }
                }
            }
        }
        result
    }
}

/// Background task that handles NIP-46 requests
async fn run_bunker_listener(
    keys: Keys,
    relays: Vec<String>,
    stop_flag: Arc<AtomicBool>,
    state: Arc<Mutex<BunkerState>>,
    key_manager: Arc<Mutex<KeyManager>>,
) -> Result<()> {
    info!("Bunker listener initializing...");
    
    // Create a Nostr client
    let client = Client::new(keys.clone());
    
    // Add relays
    for relay in &relays {
        info!("Adding relay: {}", relay);
        if let Err(e) = client.add_relay(relay).await {
            warn!("Failed to add relay {}: {}", relay, e);
        }
    }
    
    // Connect
    info!("Connecting to relays...");
    client.connect().await;
    info!("Connected to relays");
    
    // Subscribe to NIP-46 requests addressed to our pubkey
    let pubkey = keys.public_key();
    let filter = Filter::new()
        .kind(Kind::NostrConnect)
        .pubkey(pubkey)
        .since(Timestamp::now());
    
    info!("Subscribing to NIP-46 events for pubkey: {}", pubkey.to_bech32().unwrap_or_default());
    client.subscribe(filter, None).await
        .map_err(|e| SignerError::DbusError(e.to_string()))?;
    
    info!("Bunker listener ready and waiting for connections...");
    
    // Main event loop using handle_notifications with periodic checks
    loop {
        // Check stop flag first
        if stop_flag.load(Ordering::SeqCst) {
            info!("Stop flag set, exiting bunker listener");
            break;
        }
        
        // Clone state for closure
        let state_clone = Arc::clone(&state);
        let key_manager_clone = Arc::clone(&key_manager);
        let keys_clone = keys.clone();
        let client_clone = client.clone();
        let stop_flag_clone = Arc::clone(&stop_flag);
        
        // Handle notifications for a short period, then check stop flag
        let handle_result = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            client.handle_notifications(|notification| {
                let state = Arc::clone(&state_clone);
                let key_manager = Arc::clone(&key_manager_clone);
                let keys = keys_clone.clone();
                let client_send = client_clone.clone();
                let stop_flag = Arc::clone(&stop_flag_clone);
                
                async move {
                    // Check stop flag
                    if stop_flag.load(Ordering::SeqCst) {
                        return Ok(true); // Signal to stop
                    }
                    
                    if let RelayPoolNotification::Event { event, .. } = notification {
                        if event.kind == Kind::NostrConnect {
                            // Check if this is for us
                            let our_pubkey = keys.public_key();
                            let p_tags: Vec<_> = event.tags.public_keys().collect();
                            
                            if p_tags.contains(&&our_pubkey) {
                                info!("Received NIP-46 request from {}", event.pubkey.to_bech32().unwrap_or_default());
                                
                                match handle_nip46_request(&event, &keys, &key_manager, &state).await {
                                    Ok(Some(response)) => {
                                        info!("Sending NIP-46 response");
                                        if let Err(e) = client_send.send_event(&response).await {
                                            error!("Failed to send response: {}", e);
                                        }
                                    }
                                    Ok(None) => {}
                                    Err(e) => {
                                        error!("Error handling request: {}", e);
                                    }
                                }
                            }
                        }
                    }
                    
                    Ok(false) // Continue listening
                }
            })
        ).await;
        
        // Check if we should exit
        if stop_flag.load(Ordering::SeqCst) {
            break;
        }
        
        match handle_result {
            Ok(Ok(())) => {
                // Handler returned normally (signaled to stop)
            }
            Ok(Err(e)) => {
                warn!("Notification handler error: {}", e);
            }
            Err(_) => {
                // Timeout - this is expected, just continue loop
            }
        }
    }
    
    // Disconnect
    client.disconnect().await;
    info!("Bunker listener disconnected");
    
    Ok(())
}

/// Handle a NIP-46 request event
async fn handle_nip46_request(
    event: &Event,
    keys: &Keys,
    key_manager: &Arc<Mutex<KeyManager>>,
    state: &Arc<Mutex<BunkerState>>,
) -> Result<Option<Event>> {
    // Decrypt the request content using NIP-04
    let sender_pubkey = event.pubkey;
    let decrypted = nip04::decrypt(keys.secret_key(), &sender_pubkey, &event.content)
        .map_err(|e| SignerError::DecryptionError(e.to_string()))?;
    
    // Parse the request
    let request: serde_json::Value = serde_json::from_str(&decrypted)?;
    
    let method = request["method"].as_str().unwrap_or("");
    let id = request["id"].as_str().unwrap_or("");
    let params = &request["params"];
    
    info!("Received NIP-46 request: {} (id: {})", method, id);
    
    // Update state to show connected client
    {
        let mut s = state.lock().await;
        *s = BunkerState::Connected {
            client_pubkey: sender_pubkey.to_hex(),
            app_name: None,
        };
    }
    
    // Handle the request
    let result: serde_json::Value = match method {
        "connect" => {
            // Client is connecting
            let app_pubkey = params.get(0).and_then(|v| v.as_str()).unwrap_or("");
            info!("Client connecting: {}", app_pubkey);
            serde_json::json!("ack")
        }
        
        "get_public_key" => {
            let km = key_manager.lock().await;
            let pubkey = km.get_active_pubkey()
                .ok_or_else(|| SignerError::KeyNotFound("No active key".into()))?;
            serde_json::json!(pubkey)
        }
        
        "sign_event" => {
            let event_json = params.get(0).and_then(|v| v.as_str())
                .ok_or_else(|| SignerError::InvalidRequest("Missing event".into()))?;
            
            // Parse the unsigned event data
            let event_data: serde_json::Value = serde_json::from_str(event_json)?;
            let kind = event_data["kind"].as_u64().unwrap_or(1) as u16;
            let content = event_data["content"].as_str().unwrap_or("");
            let created_at = event_data["created_at"].as_u64()
                .map(Timestamp::from)
                .unwrap_or_else(Timestamp::now);
            
            let mut km = key_manager.lock().await;
            let active_keys = km.get_signing_keys().await
                .map_err(|e| SignerError::NostrError(e.to_string()))?;
            
            // Build and sign the event
            let signed = EventBuilder::new(Kind::from(kind), content)
                .custom_created_at(created_at)
                .sign_with_keys(active_keys)
                .map_err(|e| SignerError::NostrError(e.to_string()))?;
            
            serde_json::to_value(&signed)?
        }
        
        "nip04_encrypt" => {
            let third_party_pubkey = params.get(0).and_then(|v| v.as_str())
                .ok_or_else(|| SignerError::InvalidRequest("Missing pubkey".into()))?;
            let plaintext = params.get(1).and_then(|v| v.as_str())
                .ok_or_else(|| SignerError::InvalidRequest("Missing plaintext".into()))?;
            
            let pubkey = PublicKey::from_hex(third_party_pubkey)
                .map_err(|e| SignerError::NostrError(e.to_string()))?;
            
            let ciphertext = nip04::encrypt(keys.secret_key(), &pubkey, plaintext)
                .map_err(|e| SignerError::EncryptionError(e.to_string()))?;
            
            serde_json::json!(ciphertext)
        }
        
        "nip04_decrypt" => {
            let third_party_pubkey = params.get(0).and_then(|v| v.as_str())
                .ok_or_else(|| SignerError::InvalidRequest("Missing pubkey".into()))?;
            let ciphertext = params.get(1).and_then(|v| v.as_str())
                .ok_or_else(|| SignerError::InvalidRequest("Missing ciphertext".into()))?;
            
            let pubkey = PublicKey::from_hex(third_party_pubkey)
                .map_err(|e| SignerError::NostrError(e.to_string()))?;
            
            let plaintext = nip04::decrypt(keys.secret_key(), &pubkey, ciphertext)
                .map_err(|e| SignerError::DecryptionError(e.to_string()))?;
            
            serde_json::json!(plaintext)
        }
        
        "nip44_encrypt" => {
            let third_party_pubkey = params.get(0).and_then(|v| v.as_str())
                .ok_or_else(|| SignerError::InvalidRequest("Missing pubkey".into()))?;
            let plaintext = params.get(1).and_then(|v| v.as_str())
                .ok_or_else(|| SignerError::InvalidRequest("Missing plaintext".into()))?;
            
            let pubkey = PublicKey::from_hex(third_party_pubkey)
                .map_err(|e| SignerError::NostrError(e.to_string()))?;
            
            let ciphertext = nip44::encrypt(keys.secret_key(), &pubkey, plaintext, nip44::Version::default())
                .map_err(|e| SignerError::EncryptionError(e.to_string()))?;
            
            serde_json::json!(ciphertext)
        }
        
        "nip44_decrypt" => {
            let third_party_pubkey = params.get(0).and_then(|v| v.as_str())
                .ok_or_else(|| SignerError::InvalidRequest("Missing pubkey".into()))?;
            let ciphertext = params.get(1).and_then(|v| v.as_str())
                .ok_or_else(|| SignerError::InvalidRequest("Missing ciphertext".into()))?;
            
            let pubkey = PublicKey::from_hex(third_party_pubkey)
                .map_err(|e| SignerError::NostrError(e.to_string()))?;
            
            let plaintext = nip44::decrypt(keys.secret_key(), &pubkey, ciphertext)
                .map_err(|e| SignerError::DecryptionError(e.to_string()))?;
            
            serde_json::json!(plaintext)
        }
        
        "ping" => {
            serde_json::json!("pong")
        }
        
        _ => {
            warn!("Unknown NIP-46 method: {}", method);
            return Err(SignerError::InvalidRequest(format!("Unknown method: {}", method)));
        }
    };
    
    // Build response
    let response = serde_json::json!({
        "id": id,
        "result": result,
    });
    
    // Encrypt response
    let encrypted = nip04::encrypt(keys.secret_key(), &sender_pubkey, &response.to_string())
        .map_err(|e| SignerError::EncryptionError(e.to_string()))?;
    
    // Create response event
    let response_event = EventBuilder::new(Kind::NostrConnect, encrypted)
        .tag(Tag::public_key(sender_pubkey))
        .sign_with_keys(keys)
        .map_err(|e| SignerError::NostrError(e.to_string()))?;
    
    Ok(Some(response_event))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_encoding() {
        assert_eq!(urlencoding::encode("hello world"), "hello%20world");
        assert_eq!(urlencoding::encode("wss://relay.damus.io"), "wss%3A%2F%2Frelay.damus.io");
    }
}
