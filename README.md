# cpanel-rs

[![Crates.io](https://img.shields.io/crates/v/cpanel-rs.svg)](https://crates.io/crates/cpanel-rs)
[![Docs.rs](https://docs.rs/cpanel-rs/badge.svg)](https://docs.rs/cpanel-rs/latest/cpanel_rs/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust 1.70+](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)

An unofficial, async Rust client for the [cPanel](https://cpanel.net/) & [WHM](https://www.cpanel.net/products/whm/) APIs. Manage hosting accounts, DNS zones and records, email accounts and forwarders, databases, cron jobs, domains/subdomains, FTP, file manager operations, SSL certificates, backups, security rules, and server resources programmatically.

> **Disclaimer:** This project is an unofficial open-source library and is not affiliated with, endorsed by, or sponsored by WebPros / cPanel, L.L.C.

---

## Features

- **WHM API v1** — Account management, package listing, suspension/unsuspension, termination
- **cPanel UAPI** — Complete coverage: Cron, Databases, DNS, Domains/Subdomains, Email, File Manager, FTP, Resources, Security (ModSecurity & IP Blocker), and SSL
- **Triple Authentication** — HTTP Basic (Username + Password), WHM API Key (`WHM user:key`), and cPanel API Token (`cpanel user:token`)
- **Resilient Client** — Configurable exponential retry policy with jitter for transient errors (429, 503, connection drops)
- **Auto-Paginating Streams** — Async streams over large account and file listings powered by `futures::Stream`
- **Memory-Bounded Streaming** — Upload large files directly from async readers (`AsyncRead`)
- **Flexible Builder** — Custom timeouts, TLS verification toggle (for self-signed certs), and custom HTTP headers
- **Async & Fast** — Built on top of `tokio` and `reqwest` 0.12 with JSON serialization via `serde`
- **Optional Dotenv & Tracing** — Load credentials from `.env` files and enable structured diagnostics

---

## Installation

Add `cpanel-rs` to your `Cargo.toml`:

```toml
[dependencies]
cpanel-rs = "0.3"
```

### Feature Flags

| Feature | Description | Default |
|---------|-------------|---------|
| `retry` | Enables exponential backoff retry support via `RetryPolicy` | No |
| `stream` | Enables async streaming paginators (`stream_accounts`, `stream_files`) | No |
| `dotenv` | Enables loading credentials from `.env` files via `CpanelClient::from_env()` | No |
| `full` | Enables all features (`retry`, `stream`, `dotenv`) | No |

```toml
[dependencies]
cpanel-rs = { version = "0.3", features = ["full"] }
```

---

## Quick Start

```rust
use cpanel_rs::CpanelClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to WHM (port 2087) with username and password
    let client = CpanelClient::simple("server.example.com", 2087, "root", "secret_password")?;

    // List all cPanel accounts on the server
    let accounts = client.list_accounts().await?;
    for acct in accounts {
        println!("{} ({}) - Plan: {:?}", acct.username, acct.domain, acct.plan);
    }

    Ok(())
}
```

---

## Authentication & Client Configuration

### 1. Simple Constructor (Password Auth)

```rust
use cpanel_rs::CpanelClient;

// Automatically selects HTTPS if port is 2083 or 2087
let client = CpanelClient::simple("server.example.com", 2087, "root", "password")?;
```

### 2. Client Builder (Advanced Options)

Use `CpanelBuilder` for full control over auth methods, TLS verification, timeouts, and headers:

```rust
use std::time::Duration;
use cpanel_rs::CpanelBuilder;

// cPanel API Token Authentication
let client = CpanelBuilder::new("server.example.com", 2083, "johndoe")
    .token("API_TOKEN_HERE")
    .timeout(Duration::from_secs(30))
    .disable_tls_verification() // Useful for servers with self-signed SSL
    .build()?;
```

### 3. Retry Policy (Exponential Backoff)

```rust
use std::time::Duration;
use cpanel_rs::{CpanelClient, RetryPolicy};

let client = CpanelClient::simple("server.example.com", 2087, "root", "password")?;
let client = client.with_retry_policy(
    RetryPolicy::default()
        .max_retries(5)
        .base_delay(Duration::from_millis(500))
        .max_delay(Duration::from_secs(10)),
);
```

### 4. Loading from Environment Variables

Enable the `dotenv` feature:

```bash
# .env
CPANEL_HOST=server.example.com
CPANEL_PORT=2087
CPANEL_USER=root
CPANEL_PASSWORD=your_password
# Or use CPANEL_TOKEN / CPANEL_KEY:
# CPANEL_TOKEN=your_token
CPANEL_SSL=true
```

```rust
use cpanel_rs::CpanelClient;

let client = CpanelClient::from_env()?;
```

---

## API Modules Overview

| Module | Target API | Description |
|---|---|---|
| [`account`](https://docs.rs/cpanel-rs/latest/cpanel_rs/account/) | WHM API 1 | Create, list, modify, suspend, unsuspend, and terminate hosting accounts |
| [`backup`](https://docs.rs/cpanel-rs/latest/cpanel_rs/backup/) | UAPI | Create backups, restore backups, configure automated backup schedules |
| [`cron`](https://docs.rs/cpanel-rs/latest/cpanel_rs/cron/) | UAPI | List, add, edit, and delete scheduled cron jobs |
| [`database`](https://docs.rs/cpanel-rs/latest/cpanel_rs/database/) | UAPI | Create and manage MySQL databases, database users, and grant privileges |
| [`dns`](https://docs.rs/cpanel-rs/latest/cpanel_rs/dns/) | UAPI | Manage DNS records (A, CNAME, TXT, MX), export and import zone files |
| [`domains`](https://docs.rs/cpanel-rs/latest/cpanel_rs/domains/) | UAPI | Manage addon domains, parked domains (aliases), and subdomains |
| [`email`](https://docs.rs/cpanel-rs/latest/cpanel_rs/email/) | UAPI | Manage email accounts, forwarders, autoresponders, and mailing lists |
| [`filemanager`](https://docs.rs/cpanel-rs/latest/cpanel_rs/filemanager/) | UAPI | List files, create directories, upload, download, compress, and extract archives |
| [`ftp`](https://docs.rs/cpanel-rs/latest/cpanel_rs/ftp/) | UAPI | List, create, change passwords/quotas, and delete FTP accounts |
| [`resources`](https://docs.rs/cpanel-rs/latest/cpanel_rs/resources/) | UAPI | Retrieve disk usage, bandwidth consumption, and manage PHP versions |
| [`security`](https://docs.rs/cpanel-rs/latest/cpanel_rs/security/) | UAPI | Toggle ModSecurity rules and manage IP blocker blacklist/whitelist |
| [`ssl`](https://docs.rs/cpanel-rs/latest/cpanel_rs/ssl/) | UAPI | List and install SSL certificates, issue Let's Encrypt certificates |

---

## Usage Examples

### Streaming Accounts (Paginator)

```rust
use cpanel_rs::CpanelClient;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = CpanelClient::simple("server.example.com", 2087, "root", "secret_pass")?;

    let mut stream = client.stream_accounts();
    while let Some(acct) = stream.next().await {
        let acct = acct?;
        println!("Account: {} ({})", acct.username, acct.domain);
    }

    Ok(())
}
```

### Managing Cron Jobs (UAPI)

```rust
use cpanel_rs::{CpanelClient, models::AddCronJob};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = CpanelClient::simple("server.example.com", 2083, "johndoe", "user_password")?;

    let cron = AddCronJob {
        command: "/usr/local/bin/php /home/johndoe/artisan schedule:run".into(),
        minute: "*".into(),
        hour: "*".into(),
        day: "*".into(),
        month: "*".into(),
        weekday: "*".into(),
    };

    client.add_cron_job("johndoe", &cron).await?;
    println!("Cron job added successfully");

    Ok(())
}
```

### Adding an Addon Domain (UAPI)

```rust
use cpanel_rs::{CpanelClient, models::AddAddonDomain};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = CpanelClient::simple("server.example.com", 2083, "johndoe", "user_password")?;

    client.add_addon_domain("johndoe", &AddAddonDomain {
        domain: "sub.example.com".into(),
        subdomain: "sub".into(),
        dir: "public_html/sub".into(),
        password: Some("StrongPassword123!".into()),
    }).await?;

    Ok(())
}
```

### Streaming File Upload (AsyncRead)

```rust
use cpanel_rs::CpanelClient;
use tokio::fs::File;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = CpanelClient::simple("server.example.com", 2083, "johndoe", "user_password")?;

    let mut file = File::open("backup_archive.tar.gz").await?;
    client.upload_file_from_reader(
        "johndoe",
        "public_html/backup_archive.tar.gz",
        &mut file,
        "backup_archive.tar.gz",
        1024 * 1024, // 1MB chunks
    ).await?;

    println!("Large file uploaded without full in-memory buffering");
    Ok(())
}
```

---

## Building and Testing

```bash
# Clone the repository
git clone https://github.com/TheRabbiRifat/cpanel-rs.git
cd cpanel-rs

# Build the crate
cargo build --all-features

# Run unit and documentation tests
cargo test --all-features

# Run linter
cargo clippy --all-targets --all-features -- -D warnings

# Build documentation
cargo doc --no-deps --all-features --open
```

---

## Contributing

Contributions are welcome! Please check out [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on development and submitting pull requests.

---

## License

This project is licensed under the [MIT License](LICENSE).

