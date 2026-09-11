//! Security: ModSecurity rules and IP blocker via cPanel UAPI.
//!
//! These methods operate on a per-user basis (cPanel port **2083**) and manage
//! web application firewall rules (ModSecurity) and IP address blocking.
//!
//! # Overview
//!
//! ## ModSecurity
//!
//! | Method | UAPI Call | Description |
//! |--------|-----------|-------------|
//! | [`CpanelClient::list_modsec_rules`] | `ModSecurity.list_rules` | List ModSecurity rules |
//! | [`CpanelClient::disable_modsec_rule`] | `ModSecurity.disable_rule` | Disable a specific rule |
//!
//! ## IP Blocker
//!
//! | Method | UAPI Call | Description |
//! |--------|-----------|-------------|
//! | [`CpanelClient::list_ip_blocks`] | `IPBlocker.list_ip_blocks` | List blocked IPs |
//! | [`CpanelClient::block_ip`] | `IPBlocker.add_ip_block` | Block an IP address |
//! | [`CpanelClient::unblock_ip`] | `IPBlocker.del_ip_block` | Unblock an IP address |
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
//!     // List blocked IPs
//!     let blocks = client.list_ip_blocks("cpanel_user").await?;
//!     for block in &blocks {
//!         println!("Blocked: {} — {:?}", block.ip, block.reason);
//!     }
//!
//!     // Block an IP
//!     client.block_ip("cpanel_user", "192.0.2.1", Some("suspected attack")).await?;
//!
//!     // List ModSecurity rules
//!     let rules = client.list_modsec_rules("cpanel_user").await?;
//!     for rule in &rules {
//!         println!("Rule: {} — enabled: {}", rule.id, rule.enabled);
//!     }
//!
//!     // Disable a ModSecurity rule
//!     client.disable_modsec_rule("cpanel_user", "950001").await?;
//!
//!     // Unblock an IP
//!     client.unblock_ip("cpanel_user", "192.0.2.1").await?;
//!
//!     Ok(())
//! }
//! ```

use crate::{
    models::{IpBlock, ModSecRule},
    CpanelClient, CpanelError,
};
use serde::Deserialize;

/// Result wrapper for IP blocks.
#[derive(Debug, Clone, Deserialize)]
pub struct IpBlockList {
    /// The list of blocked IPs.
    pub blocks: Vec<IpBlock>,
}

/// Result wrapper for ModSecurity rules.
#[derive(Debug, Clone, Deserialize)]
pub struct ModSecRuleList {
    /// The list of ModSecurity rules.
    pub rules: Vec<ModSecRule>,
}

impl CpanelClient {
    /// Lists all blocked IP addresses for a cPanel account.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    pub async fn list_ip_blocks(&self, user: &str) -> Result<Vec<IpBlock>, CpanelError> {
        let result: IpBlockList = self
            .uapi("IPBlocker", "list_ip_blocks", &[("user", user)])
            .await?;
        Ok(result.blocks)
    }

    /// Blocks an IP address.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `ip` — The IP address to block (v4 or v6).
    /// * `reason` — Optional human-readable reason for the block.
    pub async fn block_ip(
        &self,
        user: &str,
        ip: &str,
        reason: Option<&str>,
    ) -> Result<(), CpanelError> {
        let mut params: Vec<(&str, &str)> = vec![("user", user), ("ip", ip)];
        if let Some(r) = reason {
            params.push(("reason", r));
        }
        let _: () = self.uapi("IPBlocker", "add_ip_block", &params).await?;
        Ok(())
    }

    /// Removes an IP block.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `ip` — The IP address to unblock.
    pub async fn unblock_ip(&self, user: &str, ip: &str) -> Result<(), CpanelError> {
        let _: () = self
            .uapi("IPBlocker", "del_ip_block", &[("user", user), ("ip", ip)])
            .await?;
        Ok(())
    }

    /// Lists ModSecurity rules for a cPanel account.
    ///
    /// Returns all rules known to the server, with their enabled/disabled state.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    pub async fn list_modsec_rules(&self, user: &str) -> Result<Vec<ModSecRule>, CpanelError> {
        let result: ModSecRuleList = self
            .uapi("ModSecurity", "list_rules", &[("user", user)])
            .await?;
        Ok(result.rules)
    }

    /// Disables a specific ModSecurity rule by its rule ID.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `rule_id` — The ModSecurity rule ID (e.g. `"950001"`).
    pub async fn disable_modsec_rule(&self, user: &str, rule_id: &str) -> Result<(), CpanelError> {
        let _: () = self
            .uapi(
                "ModSecurity",
                "disable_rule",
                &[("user", user), ("rule_id", rule_id)],
            )
            .await?;
        Ok(())
    }
}
