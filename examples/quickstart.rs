//! cpanel-rs usage examples.
//!
//! Run with:
//! ```bash
//! cargo run --example quickstart
//! ```

use cpanel_rs::{CpanelBuilder, CpanelClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ── Method 1: Simple password auth ──────────────────────────────────────
    //
    // For ports 2083 (cPanel) and 2087 (WHM), HTTPS is auto-detected.
    let client = CpanelClient::simple("server.example.com", 2087, "root", "password")?;

    println!("Connected to: {}", client.base_url());
    println!("Auth user:   {}", client.username());

    // ── Method 2: Full control with new() ───────────────────────────────────
    //
    // Explicit password or token. Set `use_ssl` manually for custom ports.
    let _client = CpanelClient::new(
        "192.168.1.100", // host
        8443,            // port (custom)
        "admin",         // username
        Some("mypass"),  // password (Some for password, None for token)
        None,            // token (None for password, Some for key)
        false,           // use_ssl = false (HTTP for custom port)
    )?;

    // ── Method 3: Builder with timeout, TLS, custom headers ────────────────
    //
    // Use this when you need fine-grained control.
    let _client = CpanelBuilder::new("server.example.com", 2087, "root")
        .password("secret") // or .key("whm-api-key")
        .timeout(std::time::Duration::from_secs(30))
        .disable_tls_verification() // for self-signed certs
        .header("X-Request-ID", "req-001")
        .build()?;

    // ── Example: List all WHM accounts ─────────────────────────────────────
    //
    // Uncomment to run against a real server:
    // let accounts = client.list_accounts().await?;
    // for acct in accounts {
    //     println!("{} @ {} (plan: {:?})", acct.username, acct.domain, acct.plan);
    // }

    // ── Example: Add a DNS record ──────────────────────────────────────────
    //
    // Use port 2083 for cPanel UAPI calls:
    // let uapi = CpanelClient::simple("server.example.com", 2083, "cpanel_user", "pass")?;
    // uapi.add_dns_record("cpanel_user", &AddDnsRecord {
    //     type_: "A".into(),
    //     name: "www".into(),
    //     value: "1.2.3.4".into(),
    //     ttl: Some(3600),
    // }).await?;

    println!("\nExamples loaded successfully!");
    Ok(())
}
