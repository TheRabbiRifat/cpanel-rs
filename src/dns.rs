//! DNS management via cPanel UAPI.
//!
//! These methods operate on a per-user basis (cPanel port **2083**) and manage
//! DNS zones and records.
//!
//! # Overview
//!
//! | Method | UAPI Call | Description |
//! |--------|-----------|-------------|
//! | [`CpanelClient::list_dns_records`] | `DNS.list_zone_records` | List all DNS records for a zone |
//! | [`CpanelClient::add_dns_record`] | `DNS.add_zone_record` | Add a new DNS record |
//! | [`CpanelClient::delete_dns_record`] | `DNS.del_zone_record` | Delete a DNS record |
//! | [`CpanelClient::list_zones`] | `DNS.list_zones` | List all zones for a user |
//! | [`CpanelClient::create_zone`] | `DNS.add_zone` | Create a new DNS zone |
//! | [`CpanelClient::export_zone`] | `DNS.export_zone` | Export a zone file (BIND format) |
//! | [`CpanelClient::import_zone`] | `DNS.import_zone` | Import a zone file |
//!
//! # Record Types
//!
//! The `type_` field in [`AddDnsRecord`](crate::models::AddDnsRecord) supports all standard DNS record types:
//!
//! | Type | Description |
//! |------|-------------|
//! | `A` | IPv4 address |
//! | `AAAA` | IPv6 address |
//! | `CNAME` | Canonical name alias |
//! | `MX` | Mail exchange |
//! | `TXT` | Text record |
//! | `NS` | Name server |
//! | `SRV` | Service locator |
//! | `CAA` | Certification authority restriction |
//! | `PTR` | Pointer record |
//!
//! # Example
//!
//! ```no_run
//! use cpanel_rs::{CpanelClient, models::{AddDnsRecord, DnsRecord}};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), cpanel_rs::CpanelError> {
//!     let client = CpanelClient::simple("server.example.com", 2083, "cpanel_user", "pass").unwrap();
//!
//!     // List all DNS records for example.com
//!     let records = client.list_dns_records("cpanel_user", "example.com").await?;
//!     for record in &records {
//!         println!("{} {} {} {} {}", record.name, record.type_, record.class, record.ttl, record.value);
//!     }
//!
//!     // Add an A record
//!     client.add_dns_record("cpanel_user", &AddDnsRecord {
//!         type_: "A".into(),
//!         name: "www".into(),
//!         value: "1.2.3.4".into(),
//!         ttl: Some(3600),
//!     }).await?;
//!
//!     // Export the zone
//!     let zone = client.export_zone("cpanel_user", "example.com").await?;
//!     println!("Zone {}:\n{}", zone.zone, zone.content);
//!
//!     Ok(())
//! }
//! ```

use crate::{
    models::{AddDnsRecord, DnsRecord, ZoneFile},
    CpanelClient, CpanelError,
};
use serde::Deserialize;

/// A list of DNS zones as returned by [`list_zones`](CpanelClient::list_zones).
#[derive(Debug, Clone, Deserialize)]
pub struct ZoneList {
    /// The zone names.
    pub zones: Vec<String>,
}

impl CpanelClient {
    /// Lists all DNS records for a given zone.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `zone` — The zone name (e.g. `"example.com"`).
    ///
    /// # Returns
    ///
    /// A vector of [`DnsRecord`] structs.
    pub async fn list_dns_records(
        &self,
        user: &str,
        zone: &str,
    ) -> Result<Vec<DnsRecord>, CpanelError> {
        let result: Vec<DnsRecord> = self
            .uapi(
                "DNS",
                "list_zone_records",
                &[("zone", zone), ("user", user)],
            )
            .await?;
        Ok(result)
    }

    /// Adds a new DNS record to a zone.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `record` — An [`AddDnsRecord`] with the record details.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use cpanel_rs::{CpanelClient, models::AddDnsRecord};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), cpanel_rs::CpanelError> {
    ///     let client = CpanelClient::simple("server.example.com", 2083, "user", "pass").unwrap();
    ///
    ///     // Add an MX record
    ///     client.add_dns_record("user", &AddDnsRecord {
    ///         type_: "MX".into(),
    ///         name: "@".into(),
    ///         value: "mail.example.com".into(),
    ///         ttl: Some(3600),
    ///     }).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn add_dns_record(
        &self,
        user: &str,
        record: &AddDnsRecord,
    ) -> Result<(), CpanelError> {
        let user_s = user.to_string();
        let ttl_s = record.ttl.map(|v| v.to_string());

        let mut params: Vec<(&str, &str)> = vec![
            ("type", &record.type_),
            ("name", &record.name),
            ("value", &record.value),
            ("user", &user_s),
        ];
        if let Some(ref s) = ttl_s {
            params.push(("ttl", s));
        }

        let _: () = self.uapi("DNS", "add_zone_record", &params).await?;
        Ok(())
    }

    /// Deletes a DNS record from a zone.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `record_name` — The record name (e.g. `"www"`, `"@"`).
    /// * `record_type` — The record type (e.g. `"A"`, `"CNAME"`).
    pub async fn delete_dns_record(
        &self,
        user: &str,
        record_name: &str,
        record_type: &str,
    ) -> Result<(), CpanelError> {
        let _: () = self
            .uapi(
                "DNS",
                "del_zone_record",
                &[("name", record_name), ("type", record_type), ("user", user)],
            )
            .await?;
        Ok(())
    }

    /// Lists all DNS zones for a cPanel account.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    pub async fn list_zones(&self, user: &str) -> Result<Vec<String>, CpanelError> {
        let result: ZoneList = self.uapi("DNS", "list_zones", &[("user", user)]).await?;
        Ok(result.zones)
    }

    /// Creates a new DNS zone for a cPanel account.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `zone` — The zone name (e.g. `"example.com"`).
    pub async fn create_zone(&self, user: &str, zone: &str) -> Result<(), CpanelError> {
        let _: () = self
            .uapi("DNS", "add_zone", &[("zone", zone), ("user", user)])
            .await?;
        Ok(())
    }

    /// Exports a DNS zone file in BIND format.
    ///
    /// The returned [`ZoneFile`] contains the raw zone file content, which can
    /// be saved to disk or used with [`import_zone`](CpanelClient::import_zone).
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `zone` — The zone name to export.
    pub async fn export_zone(&self, user: &str, zone: &str) -> Result<ZoneFile, CpanelError> {
        let result: ZoneFile = self
            .uapi("DNS", "export_zone", &[("zone", zone), ("user", user)])
            .await?;
        Ok(result)
    }

    /// Imports a DNS zone file.
    ///
    /// The `content` parameter should be a complete BIND-format zone file string.
    /// This will replace the existing zone data entirely.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `zone` — The zone name to import into.
    /// * `content` — The BIND-format zone file content.
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
    ///     let zone_content = r#"
    /// $ORIGIN example.com.
    /// $TTL 86400
    /// @ IN SOA ns1.example.com. admin.example.com. (
    ///     2024010101 ; Serial
    ///     3600       ; Refresh
    ///     900        ; Retry
    ///     604800     ; Expire
    ///     86400      ; Minimum TTL
    /// )
    /// @ IN NS ns1.example.com.
    /// @ IN A 1.2.3.4
    /// "#;
    ///
    ///     client.import_zone("user", "example.com", zone_content).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn import_zone(
        &self,
        user: &str,
        zone: &str,
        content: &str,
    ) -> Result<(), CpanelError> {
        let _: () = self
            .uapi(
                "DNS",
                "import_zone",
                &[("zone", zone), ("user", user), ("content", content)],
            )
            .await?;
        Ok(())
    }
}
