//! Example: Sign a Nostr event using Pleb Signer

use serde_json::json;
use std::error::Error;
use zbus::Connection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Sign Event Example");
    println!("==================\n");

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

    // Create an unsigned event
    let event = json!({
        "kind": 1,
        "content": "Hello from Pleb Signer! 🔑",
        "tags": [
            ["t", "nostr"],
            ["t", "plebsigner"]
        ]
    });

    let event_json = serde_json::to_string(&event)?;
    let app_id = "sign-event-example";

    println!("Requesting signature for event:");
    println!("{}\n", serde_json::to_string_pretty(&event)?);

    // Request signature (this will show approval dialog)
    let result: String = proxy
        .call("SignEvent", &(&event_json, "", app_id))
        .await?;

    // Parse result
    let response: serde_json::Value = serde_json::from_str(&result)?;

    if response["success"].as_bool().unwrap_or(false) {
        println!("✅ Event signed successfully!");
        if let Some(result_data) = response["result"].as_str() {
            let signed: serde_json::Value = serde_json::from_str(result_data)?;
            println!("\nSignature: {}", signed["signature"]);
            println!("Event ID: {}", signed["event_id"]);
            println!("\nFull signed event:");
            if let Some(event_json) = signed["event_json"].as_str() {
                let event: serde_json::Value = serde_json::from_str(event_json)?;
                println!("{}", serde_json::to_string_pretty(&event)?);
            }
        }
    } else {
        println!("❌ Signing failed!");
        if let Some(error) = response["error"].as_str() {
            println!("Error: {}", error);
        }
    }

    Ok(())
}
