//! # cpanel-rs
//!
//! An unofficial Rust client for the [cPanel](https://cpanel.net/) &
//! [WHM](https://www.cpanel.net/products/whm/) APIs. This crate lets you manage
//! hosting accounts, DNS records, email, databases, files, SSL certificates, and
//! backups programmatically via UAPI and WHM JSON API endpoints.
//!
//! **This project is not affiliated with or endorsed by WebPros.**
//!
//! # Quick Start
//!
//! ```toml
//! # Cargo.toml
//! [dependencies]
//! cpanel-rs = "0.3"
//! ```
//!
//! ```
//! use cpanel_rs::{CpanelClient, CpanelBuilder};
//!
//! // Simple construction
//! let client = CpanelClient::simple("server.example.com", 2087, "root", "yourpass").unwrap();
//!
//! // Or with the builder for full control
//! let client = CpanelBuilder::new("server.example.com", 2087, "root")
//!     .password("secret")
//!     .timeout(std::time::Duration::from_secs(30))
//!     .disable_tls_verification()
//!     .build()
//!     .unwrap();
//! ```
//!
//! # Supported Operations
//!
//! | Module | Functionality |
//! |--------|---------------|
//! | [`account`] | List, create, delete, suspend, modify accounts (WHM) |
//! | [`backup`] | Backup creation, listing, restoration, scheduling (UAPI) |
//! | [`cron`] | List, add, edit, delete scheduled cron jobs (UAPI) |
//! | [`database`] | MySQL databases, users, and privileges (UAPI) |
//! | [`dns`] | List, add, delete DNS records; import/export zones (UAPI) |
//! | [`domains`] | Addon domains, parked domains (aliases), and subdomains (UAPI) |
//! | [`email`] | Email accounts, forwarders, autoresponders, mailing lists (UAPI) |
//! | [`filemanager`] | List, create, delete, upload, download, compress files (UAPI) |
//! | [`ftp`] | FTP account management: list, create, modify, delete (UAPI) |
//! | [`security`] | ModSecurity rules and IP blocker (blacklist/whitelist) (UAPI) |
//! | [`ssl`] | SSL certificate management and Let's Encrypt issuance (UAPI) |
//! | [`resources`] | Disk/quota usage, PHP version management (UAPI) |
//!
//! # Authentication
//!
//! Three auth methods are supported:
//!
//! - **Password** — Basic HTTP auth. Works for both cPanel (2083) and WHM (2087).
//! - **Raw Key** — `WHM user:key` header. Requires WHM access and a raw access key
//!   generated in WHM > Manage Remote Access Key.
//! - **API Token** — `Authorization: cpanel <user>:<token>`. Generates granular,
//!   scoped permissions without requiring a full account password or root access.
//!   Create tokens in cPanel > Home > Security Center > API Tokens.
//!
//! ```no_run
//! use cpanel_rs::CpanelBuilder;
//!
//! // Password auth
//! let client = CpanelBuilder::new("myhost.com", 2087, "user").password("pass").build().unwrap();
//!
//! // Raw key auth (WHM only)
//! let client = CpanelBuilder::new("myhost.com", 2087, "root").key("abc123...").build().unwrap();
//!
//! // API Token auth (cPanel UAPI)
//! let client = CpanelBuilder::new("myhost.com", 2083, "user").token("uI7fX2aB9cD...").build().unwrap();
//! ```
//!
//! # Retry & Backoff
//!
//! Enable the `retry` feature for automatic exponential backoff on transient errors
//! (429 rate limits, 503 service unavailable, connection resets):
//!
//! ```toml
//! [dependencies]
//! cpanel-rs = { version = "0.3", features = ["retry"] }
//! ```
//!
//! ```
//! use cpanel_rs::{CpanelClient, RetryPolicy};
//!
//! let client = CpanelClient::simple("server.example.com", 2087, "root", "pass").unwrap();
//! let client = client.with_retry_policy(
//!     RetryPolicy::default().max_retries(5).base_delay(std::time::Duration::from_millis(500)),
//! );
//! ```
//!
//! # Auto-Paginating Streams
//!
//! Enable the `stream` feature to use async streams for large listings:
//!
//! ```toml
//! [dependencies]
//! cpanel-rs = { version = "0.3", features = ["stream"] }
//! ```
//!
//! ```no_run
//! #[cfg(feature = "stream")]
//! {
//!     use cpanel_rs::CpanelClient;
//!     use futures::StreamExt;
//!
//!     #[tokio::main]
//!     async fn main() -> Result<(), cpanel_rs::CpanelError> {
//!         let client = CpanelClient::simple("server.example.com", 2087, "root", "pass").unwrap();
//!
//!         // Stream all accounts without manual pagination
//!         let mut stream = client.stream_accounts();
//!         while let Some(acct) = stream.next().await {
//!             let acct = acct?;
//!             println!("{} @ {}", acct.username, acct.domain);
//!         }
//!         Ok(())
//!     }
//! }
//! ```
//!
//! # Streaming File Operations
//!
//! Upload large files from an async reader without loading them entirely into memory:
//!
//! ```no_run
//! use cpanel_rs::CpanelClient;
//! use tokio::fs::File;
//! use tokio::io::AsyncReadExt;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), cpanel_rs::CpanelError> {
//!     let client = CpanelClient::simple("server.example.com", 2083, "user", "pass").unwrap();
//!     let mut file = File::open("large_upload.zip").await?;
//!     client.upload_file_from_reader("user", "/public_html/large_upload.zip", &mut file, "large_upload.zip", 1024 * 1024).await?;
//!     Ok(())
//! }
//! ```
//!
//! # Environment Variables (opt-in)
//!
//! Enable the `dotenv` feature to load credentials from a `.env` file:
//!
//! ```toml
//! [dependencies]
//! cpanel-rs = { version = "0.3", features = ["dotenv"] }
//! ```
//!
//! Then create a `.env` file:
//!
//! ```text
//! CPANEL_HOST=server.example.com
//! CPANEL_PORT=2087
//! CPANEL_USER=root
//! CPANEL_PASSWORD=yourpass
//! ```
//!
//! # Error Handling
//!
//! All methods return [`Result<_, CpanelError>`]. Errors fall into these categories:
//!
//! - **[`CpanelError::Http`]** — network or connection failure
//! - **[`CpanelError::ApiError`]** — the API returned a non-success status
//! - **[`CpanelError::ApiWarning`]** — the API returned success but with warnings
//! - **[`CpanelError::Auth`]** — missing or invalid credentials
//! - **[`CpanelError::Serialization`]** — invalid JSON response
//!
//! Use [`CpanelError::ApiError`] to inspect structured
//! failure details including all error messages, warnings, and stack traces.

#![warn(missing_docs, clippy::missing_errors_doc)]
#![allow(
    clippy::missing_errors_doc,
    clippy::result_large_err,
    clippy::type_complexity
)]

/// cPanel & WHM account management (WHM API).
pub mod account;
/// Backup management (UAPI).
pub mod backup;
/// Core client: [`CpanelClient`], [`CpanelBuilder`], [`AuthMethod`], and [`RetryPolicy`].
pub mod client;
/// Cron job management (UAPI).
pub mod cron;
/// MySQL database management (UAPI).
pub mod database;
/// DNS record management (UAPI).
pub mod dns;
/// Domain and subdomain management (UAPI).
pub mod domains;
/// Email account management (UAPI).
pub mod email;
/// File manager operations (UAPI).
pub mod filemanager;
/// FTP account management (UAPI).
pub mod ftp;
/// Data models shared across modules.
pub mod models;
/// Resource usage and PHP version management (UAPI).
pub mod resources;
/// Security: ModSecurity rules and IP blocker (UAPI).
pub mod security;
/// SSL certificate management (UAPI).
pub mod ssl;
/// Auto-paginating async streams for large listings (requires `stream` feature).
#[cfg(feature = "stream")]
pub mod streams;
/// Core types: errors, response envelopes, and query builders.
pub mod types;

pub use client::{AuthMethod, CpanelBuilder, CpanelClient, RetryPolicy};
pub use types::CpanelError;
