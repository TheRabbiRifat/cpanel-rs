//! Domain and subdomain management via cPanel UAPI.
//!
//! These methods operate on a per-user basis (cPanel port **2083**) and manage
//! addon domains, parked domains (aliases), and subdomains.
//!
//! # Overview
//!
//! | Module | Method | UAPI Call | Description |
//! |--------|--------|-----------|-------------|
//! | DomainInfo | [`CpanelClient::list_addon_domains`] | `DomainInfo.list_addon_domains` | List addon domains |
//! | DomainInfo | [`CpanelClient::create_addon_domain`] | `DomainInfo.add_addon_domain` | Add an addon domain |
//! | DomainInfo | [`CpanelClient::delete_addon_domain`] | `DomainInfo.del_addon_domain` | Delete an addon domain |
//! | SubDomain | [`CpanelClient::list_subdomains`] | `SubDomain.list_subdomains` | List subdomains |
//! | SubDomain | [`CpanelClient::create_subdomain`] | `SubDomain.add_subdomain` | Create a subdomain |
//! | SubDomain | [`CpanelClient::delete_subdomain`] | `SubDomain.del_subdomain` | Delete a subdomain |
//!
//! # Example
//!
//! ```no_run
//! use cpanel_rs::{CpanelClient, models::{AddAddonDomain, AddSubdomain}};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), cpanel_rs::CpanelError> {
//!     let client = CpanelClient::simple("server.example.com", 2083, "cpanel_user", "pass").unwrap();
//!
//!     // List addon domains
//!     let addons = client.list_addon_domains("cpanel_user").await?;
//!     for addon in &addons {
//!         println!("Addon: {} -> {}", addon.domain, addon.document_root);
//!     }
//!
//!     // Create an addon domain
//!     client.create_addon_domain("cpanel_user", &AddAddonDomain {
//!         domain: "newsite.com".into(),
//!         document_root: Some("/public_html/newsite".into()),
//!         redirect: None,
//!     }).await?;
//!
//!     // List subdomains
//!     let subs = client.list_subdomains("cpanel_user").await?;
//!     for sub in &subs {
//!         println!("Subdomain: {} -> {:?}", sub.name, sub.target);
//!     }
//!
//!     // Create a subdomain
//!     client.create_subdomain("cpanel_user", &AddSubdomain {
//!         subdomain: "blog".into(),
//!         domain: "example.com".into(),
//!         direction: Some("forward".into()),
//!         target: Some("https://blog.example.com".into()),
//!     }).await?;
//!
//!     Ok(())
//! }
//! ```

use crate::{
    models::{AddAddonDomain, AddSubdomain, AddonDomain, Subdomain},
    CpanelClient, CpanelError,
};
use serde::Deserialize;

/// A list result wrapping addon domains.
#[derive(Debug, Clone, Deserialize)]
pub struct AddonDomainList {
    /// The list of addon domains.
    pub domains: Vec<AddonDomain>,
}

/// A list result wrapping subdomains.
#[derive(Debug, Clone, Deserialize)]
pub struct SubdomainList {
    /// The list of subdomains.
    pub subdomains: Vec<Subdomain>,
}

impl CpanelClient {
    /// Lists all addon domains for a cPanel account.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    pub async fn list_addon_domains(&self, user: &str) -> Result<Vec<AddonDomain>, CpanelError> {
        let result: AddonDomainList = self
            .uapi("DomainInfo", "list_addon_domains", &[("user", user)])
            .await?;
        Ok(result.domains)
    }

    /// Creates a new addon domain.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `addon` — An [`AddAddonDomain`] with the domain and optional settings.
    pub async fn create_addon_domain(
        &self,
        user: &str,
        addon: &AddAddonDomain,
    ) -> Result<(), CpanelError> {
        let mut params: Vec<(&str, &str)> = vec![("user", user), ("domain", &addon.domain)];
        if let Some(ref s) = addon.document_root {
            params.push(("document_root", s));
        }
        if let Some(ref s) = addon.redirect {
            params.push(("redirect", s));
        }
        let _: () = self.uapi("DomainInfo", "add_addon_domain", &params).await?;
        Ok(())
    }

    /// Deletes an addon domain.
    ///
    /// **Warning:** This removes the addon domain but does not delete its files.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `domain` — The addon domain name to delete.
    pub async fn delete_addon_domain(&self, user: &str, domain: &str) -> Result<(), CpanelError> {
        let _: () = self
            .uapi(
                "DomainInfo",
                "del_addon_domain",
                &[("user", user), ("domain", domain)],
            )
            .await?;
        Ok(())
    }

    /// Lists all subdomains for a cPanel account.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    pub async fn list_subdomains(&self, user: &str) -> Result<Vec<Subdomain>, CpanelError> {
        let result: SubdomainList = self
            .uapi("SubDomain", "list_subdomains", &[("user", user)])
            .await?;
        Ok(result.subdomains)
    }

    /// Creates a new subdomain.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `sub` — An [`AddSubdomain`] with the subdomain details.
    pub async fn create_subdomain(
        &self,
        user: &str,
        sub: &AddSubdomain,
    ) -> Result<(), CpanelError> {
        let mut params: Vec<(&str, &str)> = vec![
            ("user", user),
            ("subdomain", &sub.subdomain),
            ("domain", &sub.domain),
        ];
        if let Some(ref s) = sub.direction {
            params.push(("direction", s));
        }
        if let Some(ref s) = sub.target {
            params.push(("target", s));
        }
        let _: () = self.uapi("SubDomain", "add_subdomain", &params).await?;
        Ok(())
    }

    /// Deletes a subdomain.
    ///
    /// **Warning:** This removes the subdomain but does not delete its files.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `subdomain` — The subdomain name (e.g. `"blog"`).
    /// * `domain` — The parent domain (e.g. `"example.com"`).
    pub async fn delete_subdomain(
        &self,
        user: &str,
        subdomain: &str,
        domain: &str,
    ) -> Result<(), CpanelError> {
        let _: () = self
            .uapi(
                "SubDomain",
                "del_subdomain",
                &[("user", user), ("subdomain", subdomain), ("domain", domain)],
            )
            .await?;
        Ok(())
    }
}
