//! Homeserver validation example
//!
//! This example demonstrates how to use the Matrix SDK-based homeserver validation
//! functionality implemented in the clitrix application.
//!
//! Run with: cargo run --example homeserver_validation

use clitrix::data::MatrixService;
use color_eyre::Result;
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    println!("🔍 Matrix Homeserver Validation Demo");
    println!("=====================================\n");

    let matrix_service = MatrixService::new();

    // Test various homeserver URLs
    let test_homeservers = vec![
        // Valid homeservers
        ("karibooru.love", "Public Matrix.org homeserver"),
        ("chat.mozilla.org", "Mozilla's homeserver"),
        ("matrix.kde.org", "KDE's homeserver"),
        ("localhost:8008", "Local development server"),
        ("127.0.0.1:8448", "Local IP with Matrix port"),
        ("https://matrix.example.com", "HTTPS URL format"),
        // Invalid homeservers
        ("", "Empty homeserver"),
        ("not-a-domain", "Invalid domain format"),
        ("ftp://example.com", "Wrong protocol"),
        ("invalid..domain.com", "Invalid domain syntax"),
        ("example.com:99999", "Invalid port number"),
        ("javascript:alert('xss')", "Potential XSS attempt"),
    ];

    for (homeserver, description) in test_homeservers {
        println!("Testing: {} ({})", homeserver, description);
        print!("  Result: ");

        match matrix_service.check_homeserver(homeserver).await {
            Ok(true) => {
                println!("✅ Valid homeserver");
            }
            Ok(false) => {
                println!("❌ Invalid homeserver (returned false)");
            }
            Err(e) => {
                println!("❌ Error: {}", e);
            }
        }
        println!();
    }

    println!("\n📝 How this works:");
    println!("1. The homeserver URL is normalized (add https:// if missing)");
    println!("2. Matrix SDK's ServerName::parse() validates the format");
    println!("3. A temporary Matrix client is created to test connectivity");
    println!("4. The client attempts to get server capabilities");
    println!("5. Results indicate whether the homeserver is valid and reachable");

    println!("\n🔧 Integration with LoginForm:");
    println!("In the login screen, you can call:");
    println!("  let result = login_form.validate_homeserver_async(&matrix_service).await;");
    println!("This provides real-time validation as users type their homeserver URL.");

    Ok(())
}
