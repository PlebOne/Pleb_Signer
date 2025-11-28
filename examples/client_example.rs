//! Example client usage for Pleb Signer

use std::error::Error;

// This would use the client module from pleb_signer
// For now, we demonstrate the D-Bus interaction directly

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Pleb Signer Client Example");
    println!("==========================\n");

    // In a real application, you would use:
    // use pleb_signer::client::PlebSignerClient;
    // let client = PlebSignerClient::new("my-app").await?;

    // For this example, we'll use zbus directly
    use zbus::Connection;

    let connection = Connection::session().await?;

    // Create a proxy to the signer service
    let proxy = zbus::Proxy::new(
        &connection,
        "com.plebsigner.Signer",
        "/com/plebsigner/Signer",
        "com.plebsigner.Signer1",
    )
    .await;

    match proxy {
        Ok(proxy) => {
            // Check version
            match proxy.call::<_, String>("Version", &()).await {
                Ok(version) => println!("Signer version: {}", version),
                Err(e) => println!("Failed to get version: {}", e),
            }

            // Check if ready
            match proxy.call::<_, bool>("IsReady", &()).await {
                Ok(ready) => {
                    if ready {
                        println!("Signer is ready!");

                        // List keys
                        match proxy.call::<_, String>("ListKeys", &()).await {
                            Ok(keys_json) => println!("Keys: {}", keys_json),
                            Err(e) => println!("Failed to list keys: {}", e),
                        }

                        // Get public key
                        match proxy.call::<_, String>("GetPublicKey", &("",)).await {
                            Ok(result) => println!("Public key: {}", result),
                            Err(e) => println!("Failed to get public key: {}", e),
                        }
                    } else {
                        println!("Signer is locked. Please unlock it first.");
                    }
                }
                Err(e) => println!("Failed to check if ready: {}", e),
            }
        }
        Err(e) => {
            println!("Could not connect to Pleb Signer: {}", e);
            println!("\nMake sure Pleb Signer is running:");
            println!("  pleb-signer");
        }
    }

    Ok(())
}
