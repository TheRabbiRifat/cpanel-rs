//! SSL certificate management via cPanel UAPI.
//!
//! These methods operate on a per-user basis (cPanel port **2083**) and manage
//! SSL/TLS certificates for domains.
//!
//! # Overview
//!
//! | Method | UAPI Call | Description |
//! |--------|-----------|-------------|
//! | [`CpanelClient::list_ssl_certs`] | `SSL.get_cert_info` | List all SSL certificates |
//! | [`CpanelClient::install_ssl`] | `SSL.install_cert` | Install a certificate |
//! | [`CpanelClient::delete_ssl`] | `SSL.delete_cert` | Remove a certificate |
//! | [`CpanelClient::issue_letsencrypt`] | `SSL.generate_csr` | Request Let's Encrypt cert |
//!
//! # Certificate Sources
//!
//! There are two ways to obtain SSL certificates:
//!
//! 1. **Let's Encrypt** — Free certificates issued automatically via
//!    [`issue_letsencrypt`](CpanelClient::issue_letsencrypt). Requires the
//!    cPanel server to have the Let's Encrypt plugin enabled.
//!
//! 2. **Manual install** — Provide your own CSR and certificate via
//!    [`install_ssl`](CpanelClient::install_ssl). Useful for commercial
//!    certificates (DigiCert, Let's Encrypt paid, etc.) or internal CA certs.
//!
//! # Example
//!
//! ```no_run
//! use cpanel_rs::{CpanelClient, models::{InstallSsl}};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), cpanel_rs::CpanelError> {
//!     let client = CpanelClient::simple("server.example.com", 2083, "cpanel_user", "pass").unwrap();
//!
//!     // List current certificates
//!     let certs = client.list_ssl_certs("cpanel_user").await?;
//!     for cert in &certs {
//!         let expiry = cert.not_after.as_deref().unwrap_or("unknown");
//!         let status = cert.active.map(|a| if a { "active" } else { "inactive" }).unwrap_or("unknown");
//!         println!("{} — {} — {}", cert.domain, expiry, status);
//!     }
//!
//!     // Install a certificate (requires a pre-generated CSR)
//!     client.install_ssl("cpanel_user", &InstallSsl {
//!         domain: "example.com".into(),
//!         csr: "-----BEGIN CERTIFICATE REQUEST-----\nMIIC...".into(),
//!         cert: "-----BEGIN CERTIFICATE-----\nMIID...".into(),
//!         ca: Some("-----BEGIN CERTIFICATE-----\nMIID...".into()),
//!         bundle: Some(true),
//!     }).await?;
//!
//!     // Or request a free Let's Encrypt certificate
//!     client.issue_letsencrypt("cpanel_user", "example.com").await?;
//!
//!     Ok(())
//! }
//! ```

use crate::{
    models::{InstallSsl, SslCertificate},
    CpanelClient, CpanelError,
};
use serde::Deserialize;

/// A list of SSL certificates as returned by [`list_ssl_certs`](CpanelClient::list_ssl_certs).
#[derive(Debug, Clone, Deserialize)]
pub struct SslList {
    /// The list of certificates.
    pub certificates: Vec<SslCertificate>,
}

impl CpanelClient {
    /// Lists all SSL certificates for a cPanel account.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    ///
    /// # Returns
    ///
    /// A vector of [`SslCertificate`] structs.
    pub async fn list_ssl_certs(&self, user: &str) -> Result<Vec<SslCertificate>, CpanelError> {
        let result: Vec<SslCertificate> =
            self.uapi("SSL", "get_cert_info", &[("user", user)]).await?;
        Ok(result)
    }

    /// Installs an SSL certificate for a domain.
    ///
    /// You must have a valid CSR (Certificate Signing Request) and the
    /// certificate in PEM format. The CSR should have been generated for
    /// the target domain.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `install` — An [`InstallSsl`] with the domain, CSR, certificate, and optional CA bundle.
    pub async fn install_ssl(&self, user: &str, install: &InstallSsl) -> Result<(), CpanelError> {
        let user_s = user.to_string();
        let ca_s = install.ca.clone();
        let bundle_s = install.bundle.map(|v| v.to_string());

        let mut params: Vec<(&str, &str)> = vec![
            ("domain", &install.domain),
            ("csr", &install.csr),
            ("cert", &install.cert),
            ("user", &user_s),
        ];
        if let Some(ref s) = ca_s {
            params.push(("ca", s));
        }
        if let Some(ref s) = bundle_s {
            params.push(("bundle", s));
        }

        let _: () = self.uapi("SSL", "install_cert", &params).await?;
        Ok(())
    }

    /// Deletes an SSL certificate for a domain.
    ///
    /// **Warning:** This will break HTTPS for the domain until a new certificate
    /// is installed.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `domain` — The domain whose certificate should be removed.
    pub async fn delete_ssl(&self, user: &str, domain: &str) -> Result<(), CpanelError> {
        let _: () = self
            .uapi("SSL", "delete_cert", &[("domain", domain), ("user", user)])
            .await?;
        Ok(())
    }

    /// Requests a free Let's Encrypt SSL certificate for a domain.
    ///
    /// This requires the cPanel server to have the Let's Encrypt plugin
    /// enabled (available in cPanel & WHM version 78+).
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `domain` — The domain to issue the certificate for.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use cpanel_rs::CpanelClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), cpanel_rs::CpanelError> {
    ///     let client = CpanelClient::simple("server.example.com", 2083, "user", "pass").unwrap();
    ///
    ///     // Request a free Let's Encrypt certificate
    ///     client.issue_letsencrypt("user", "example.com").await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn issue_letsencrypt(&self, user: &str, domain: &str) -> Result<(), CpanelError> {
        let _: () = self
            .uapi(
                "SSL",
                "generate_csr",
                &[("domain", domain), ("user", user), ("letsencrypt", "1")],
            )
            .await?;
        Ok(())
    }
}
