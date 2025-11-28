//! Example: NIP-04 encryption/decryption using Pleb Signer

use std::error::Error;
use zbus::Connection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("NIP-04 Encryption Example");
    println!("=========================\n");

    let connection = Connection::session().await?;

    let proxy = zbus::Proxy::new(
        &connection,
        "com.plebsigner.Signer",
        "/com/plebsigner/Signer",
        "com.plebsigner.Signer1",
    )
    .await?;

    // Check if signer is ready
    let ready: bool = proxy.call("IsReady", &()).await?;
    if !ready {
        println!("Signer is locked. Please unlock it first.");
        return Ok(());
    }

    // Get our public key first
    let pubkey_result: String = proxy.call("GetPublicKey", &("",)).await?;
    let pubkey_response: serde_json::Value = serde_json::from_str(&pubkey_result)?;
    
    if !pubkey_response["success"].as_bool().unwrap_or(false) {
        println!("Failed to get public key");
        return Ok(());
    }

    let pubkey_data: serde_json::Value = serde_json::from_str(
        pubkey_response["result"].as_str().unwrap_or("{}")
    )?;
    let our_pubkey = pubkey_data["pubkey_hex"].as_str().unwrap_or("");
    
    println!("Our public key: {}", our_pubkey);

    // For this example, we'll encrypt a message to ourselves
    let plaintext = "This is a secret message! 🤫";
    let app_id = "nip04-example";

    println!("\nEncrypting message: {}", plaintext);

    // Encrypt
    let encrypt_result: String = proxy
        .call("Nip04Encrypt", &(plaintext, our_pubkey, "", app_id))
        .await?;

    let encrypt_response: serde_json::Value = serde_json::from_str(&encrypt_result)?;

    if encrypt_response["success"].as_bool().unwrap_or(false) {
        let result_data: serde_json::Value = serde_json::from_str(
            encrypt_response["result"].as_str().unwrap_or("{}")
        )?;
        let ciphertext = result_data["ciphertext"].as_str().unwrap_or("");
        
        println!("✅ Encrypted: {}", ciphertext);

        // Now decrypt it
        println!("\nDecrypting...");
        
        let decrypt_result: String = proxy
            .call("Nip04Decrypt", &(ciphertext, our_pubkey, "", app_id))
            .await?;

        let decrypt_response: serde_json::Value = serde_json::from_str(&decrypt_result)?;

        if decrypt_response["success"].as_bool().unwrap_or(false) {
            let decrypt_data: serde_json::Value = serde_json::from_str(
                decrypt_response["result"].as_str().unwrap_or("{}")
            )?;
            let decrypted = decrypt_data["plaintext"].as_str().unwrap_or("");
            
            println!("✅ Decrypted: {}", decrypted);

            if decrypted == plaintext {
                println!("\n✅ Round-trip successful! Messages match.");
            }
        } else {
            println!("❌ Decryption failed!");
        }
    } else {
        println!("❌ Encryption failed!");
        if let Some(error) = encrypt_response["error"].as_str() {
            println!("Error: {}", error);
        }
    }

    Ok(())
}
