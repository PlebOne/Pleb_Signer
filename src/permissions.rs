//! Permission management for Pleb Signer

use crate::config::AppPermissions;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Types of requests that can be made to the signer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestType {
    GetPublicKey,
    SignEvent,
    Nip04Encrypt,
    Nip04Decrypt,
    Nip44Encrypt,
    Nip44Decrypt,
    DecryptZapEvent,
}

impl RequestType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RequestType::GetPublicKey => "get_public_key",
            RequestType::SignEvent => "sign_event",
            RequestType::Nip04Encrypt => "nip04_encrypt",
            RequestType::Nip04Decrypt => "nip04_decrypt",
            RequestType::Nip44Encrypt => "nip44_encrypt",
            RequestType::Nip44Decrypt => "nip44_decrypt",
            RequestType::DecryptZapEvent => "decrypt_zap_event",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            RequestType::GetPublicKey => "Get Public Key",
            RequestType::SignEvent => "Sign Event",
            RequestType::Nip04Encrypt => "NIP-04 Encrypt",
            RequestType::Nip04Decrypt => "NIP-04 Decrypt",
            RequestType::Nip44Encrypt => "NIP-44 Encrypt",
            RequestType::Nip44Decrypt => "NIP-44 Decrypt",
            RequestType::DecryptZapEvent => "Decrypt Zap Event",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            RequestType::GetPublicKey => "Access your public key (npub)",
            RequestType::SignEvent => "Sign a Nostr event with your private key",
            RequestType::Nip04Encrypt => "Encrypt a message using NIP-04",
            RequestType::Nip04Decrypt => "Decrypt a message using NIP-04",
            RequestType::Nip44Encrypt => "Encrypt a message using NIP-44",
            RequestType::Nip44Decrypt => "Decrypt a message using NIP-44",
            RequestType::DecryptZapEvent => "Decrypt a zap event",
        }
    }

    pub fn is_sensitive(&self) -> bool {
        match self {
            RequestType::GetPublicKey => false,
            _ => true,
        }
    }
}

impl std::str::FromStr for RequestType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "get_public_key" => Ok(RequestType::GetPublicKey),
            "sign_event" => Ok(RequestType::SignEvent),
            "nip04_encrypt" => Ok(RequestType::Nip04Encrypt),
            "nip04_decrypt" => Ok(RequestType::Nip04Decrypt),
            "nip44_encrypt" => Ok(RequestType::Nip44Encrypt),
            "nip44_decrypt" => Ok(RequestType::Nip44Decrypt),
            "decrypt_zap_event" => Ok(RequestType::DecryptZapEvent),
            _ => Err(format!("Unknown request type: {}", s)),
        }
    }
}

/// Permission checker for applications
pub struct PermissionChecker;

impl PermissionChecker {
    /// Check if an app has permission for a specific request type
    pub fn check_permission(
        permissions: &AppPermissions,
        request_type: RequestType,
        event_kind: Option<u16>,
    ) -> bool {
        match request_type {
            RequestType::GetPublicKey => permissions.get_public_key,
            RequestType::SignEvent => {
                match &permissions.sign_event {
                    None => true, // All kinds allowed
                    Some(kinds) => {
                        if kinds.is_empty() {
                            false // No kinds allowed
                        } else {
                            // Check if specific kind is allowed
                            event_kind.map(|k| kinds.contains(&k)).unwrap_or(false)
                        }
                    }
                }
            }
            RequestType::Nip04Encrypt => permissions.nip04_encrypt,
            RequestType::Nip04Decrypt => permissions.nip04_decrypt,
            RequestType::Nip44Encrypt => permissions.nip44_encrypt,
            RequestType::Nip44Decrypt => permissions.nip44_decrypt,
            RequestType::DecryptZapEvent => permissions.decrypt_zap_event,
        }
    }
}

/// Rate limiter for auto-approved requests
pub struct RateLimiter {
    /// Map of app_id to (request_type -> timestamps of recent requests)
    requests: HashMap<String, HashMap<RequestType, Vec<std::time::Instant>>>,
    /// Maximum requests per minute
    max_per_minute: u32,
}

impl RateLimiter {
    pub fn new(max_per_minute: u32) -> Self {
        Self {
            requests: HashMap::new(),
            max_per_minute,
        }
    }

    /// Check if a request is allowed and record it
    pub fn check_and_record(&mut self, app_id: &str, request_type: RequestType) -> bool {
        let now = std::time::Instant::now();
        let one_minute_ago = now - std::time::Duration::from_secs(60);

        let app_requests = self.requests.entry(app_id.to_string()).or_default();
        let type_requests = app_requests.entry(request_type).or_default();

        // Remove old requests
        type_requests.retain(|t| *t > one_minute_ago);

        // Check if under limit
        if type_requests.len() < self.max_per_minute as usize {
            type_requests.push(now);
            true
        } else {
            false
        }
    }

    /// Clear old entries periodically
    pub fn cleanup(&mut self) {
        let one_minute_ago = std::time::Instant::now() - std::time::Duration::from_secs(60);

        for app_requests in self.requests.values_mut() {
            for type_requests in app_requests.values_mut() {
                type_requests.retain(|t| *t > one_minute_ago);
            }
        }

        // Remove empty entries
        self.requests.retain(|_, v| {
            v.retain(|_, r| !r.is_empty());
            !v.is_empty()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_check() {
        let mut permissions = AppPermissions::default();
        permissions.get_public_key = true;
        permissions.sign_event = Some(vec![1, 4]);

        assert!(PermissionChecker::check_permission(
            &permissions,
            RequestType::GetPublicKey,
            None
        ));

        assert!(PermissionChecker::check_permission(
            &permissions,
            RequestType::SignEvent,
            Some(1)
        ));

        assert!(!PermissionChecker::check_permission(
            &permissions,
            RequestType::SignEvent,
            Some(0)
        ));

        assert!(!PermissionChecker::check_permission(
            &permissions,
            RequestType::Nip04Decrypt,
            None
        ));
    }

    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(3);

        assert!(limiter.check_and_record("app1", RequestType::SignEvent));
        assert!(limiter.check_and_record("app1", RequestType::SignEvent));
        assert!(limiter.check_and_record("app1", RequestType::SignEvent));
        assert!(!limiter.check_and_record("app1", RequestType::SignEvent)); // Over limit

        // Different app should work
        assert!(limiter.check_and_record("app2", RequestType::SignEvent));
    }
}
