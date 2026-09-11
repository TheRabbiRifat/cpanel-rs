//! Resource usage and PHP management via cPanel UAPI.
//!
//! These methods operate on a per-user basis (cPanel port **2083**) and provide
//! information about disk usage, bandwidth, inode counts, and PHP version settings.
//!
//! # Overview
//!
//! | Method | UAPI Call | Description |
//! |--------|-----------|-------------|
//! | [`CpanelClient::get_user_quota`] | `ResourceUsage.get_quota` | Get disk/quota usage |
//! | [`CpanelClient::list_php_versions`] | `MultiPHP.list_versions` | List available PHP versions |
//! | [`CpanelClient::set_php_version`] | `MultiPHP.set_version` | Change PHP version |
//!
//! # Example
//!
//! ```no_run
//! use cpanel_rs::CpanelClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), cpanel_rs::CpanelError> {
//!     let client = CpanelClient::simple("server.example.com", 2083, "cpanel_user", "pass").unwrap();
//!
//!     // Check resource usage
//!     let usage = client.get_user_quota("cpanel_user").await?;
//!     println!("Disk: {}/{} MB", usage.disk_used, usage.disk_quota);
//!     println!("Inodes: {}/{}", usage.inode_used, usage.inode_quota);
//!
//!     // List available PHP versions
//!     let versions = client.list_php_versions("cpanel_user").await?;
//!     for v in &versions {
//!         let marker = if v.default { " (default)" } else { "" };
//!         println!("{} {}{}", v.version, v.handler, marker);
//!     }
//!
//!     // Switch PHP version
//!     client.set_php_version("cpanel_user", "8.1").await?;
//!
//!     Ok(())
//! }
//! ```

use crate::{
    models::{PhpVersion, ResourceUsage},
    CpanelClient, CpanelError,
};

impl CpanelClient {
    /// Gets detailed resource usage for a cPanel account.
    ///
    /// Returns disk usage, inode counts, bandwidth, and CPU information.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    ///
    /// # Fields in the Response
    ///
    /// | Field | Unit | Notes |
    /// |-------|------|-------|
    /// | `disk_used` | MB | Current disk usage |
    /// | `disk_quota` | MB | `-1` means unlimited |
    /// | `inode_used` | count | Number of files/inodes used |
    /// | `inode_quota` | count | `-1` means unlimited |
    /// | `bandwidth_used` | MB | Total bandwidth consumed |
    /// | `bandwidth_quota` | MB | `-1` means unlimited |
    /// | `cpu_percent` | % | Current CPU usage percentage |
    pub async fn get_user_quota(&self, user: &str) -> Result<ResourceUsage, CpanelError> {
        let result: ResourceUsage = self
            .uapi("ResourceUsage", "get_quota", &[("user", user)])
            .await?;
        Ok(result)
    }

    /// Lists all available PHP versions for a cPanel account.
    ///
    /// The returned list includes the handler type (`php-fpm`, `proxy-php`, etc.)
    /// and which version is currently set as default.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
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
    ///     let versions = client.list_php_versions("user").await?;
    ///     let defaults: Vec<&str> = versions
    ///         .iter()
    ///         .filter(|v| v.default)
    ///         .map(|v| v.version.as_str())
    ///         .collect();
    ///
    ///     println!("Default PHP version(s): {:?}", defaults);
    ///     Ok(())
    /// }
    /// ```
    pub async fn list_php_versions(&self, user: &str) -> Result<Vec<PhpVersion>, CpanelError> {
        let result: Vec<PhpVersion> = self
            .uapi("MultiPHP", "list_versions", &[("user", user)])
            .await?;
        Ok(result)
    }

    /// Changes the PHP version for a cPanel account.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `version` — The PHP version to set (e.g. `"7.4"`, `"8.0"`, `"8.1"`, `"8.2"`, `"8.3"`).
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
    ///     // Upgrade to PHP 8.1
    ///     client.set_php_version("user", "8.1").await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn set_php_version(&self, user: &str, version: &str) -> Result<(), CpanelError> {
        let _: () = self
            .uapi(
                "MultiPHP",
                "set_version",
                &[("user", user), ("version", version)],
            )
            .await?;
        Ok(())
    }
}
