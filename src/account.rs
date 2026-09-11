//! Account management via WHM API.
//!
//! These methods require WHM (root or reseller) access on port **2087**.
//!
//! # Overview
//!
//! | Method | WHM Function | Description |
//! |--------|-------------|-------------|
//! | [`CpanelClient::list_accounts`] | `listaccts` | List all cPanel accounts |
//! | [`CpanelClient::get_account_info`] | `getacctinfo` | Get details for one account |
//! | [`CpanelClient::create_account`] | `addacct` | Create a new cPanel account |
//! | [`CpanelClient::delete_account`] | `delacct` | Delete an account |
//! | [`CpanelClient::suspend_account`] | `suspendacct` | Suspend an account |
//! | [`CpanelClient::unsuspend_account`] | `unsuspendacct` | Unsuspend an account |
//! | [`CpanelClient::modify_account`] | `modifyacct` | Change plan/IP/suspension |
//! | [`CpanelClient::change_password`] | `chpass` | Change account password |
//!
//! # Example
//!
//! ```no_run
//! use cpanel_rs::{CpanelClient, models::CreateAccount};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), cpanel_rs::CpanelError> {
//!     let client = CpanelClient::simple("server.example.com", 2087, "root", "pass").unwrap();
//!
//!     // List all accounts
//!     let accounts = client.list_accounts().await?;
//!
//!     // Create a new account
//!     client.create_account(&CreateAccount {
//!         username: "newuser".into(),
//!         password: "strongpass".into(),
//!         domain: "newsite.com".into(),
//!         plan: Some("default".into()),
//!         ..Default::default()
//!     }).await?;
//!
//!     Ok(())
//! }
//! ```

use crate::{
    models::{AccountInfo, CreateAccount},
    types::QueryParams,
    CpanelClient, CpanelError,
};
use serde::Serialize;

/// Parameters for modifying an existing account.
///
/// See [`CpanelClient::modify_account`] for usage.
///
/// # Example
///
/// ```
/// use cpanel_rs::models::ModifyAccount;
///
/// let modify = ModifyAccount {
///     username: "john".into(),
///     plan: Some("premium".into()),
///     suspend: Some(true),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Default, Serialize)]
pub struct ModifyAccount {
    /// The username of the account to modify.
    pub username: String,
    /// New resource plan.
    pub plan: Option<String>,
    /// New IP address.
    pub ip: Option<String>,
    /// Set to `Some(true)` to suspend, `Some(false)` to unsuspend.
    pub suspend: Option<bool>,
}

impl CpanelClient {
    /// Lists all cPanel accounts on the server.
    ///
    /// Requires WHM access. Returns a vector of [`AccountInfo`] for every
    /// account on the server.
    ///
    /// # Errors
    ///
    /// Returns [`CpanelError::ApiError`] if the WHM call fails, or
    /// [`CpanelError::Auth`] if credentials are invalid.
    pub async fn list_accounts(&self) -> Result<Vec<AccountInfo>, CpanelError> {
        let result: Vec<AccountInfo> = self.whm("listaccts", &[]).await?;
        Ok(result)
    }

    /// Lists accounts with optional pagination and filtering.
    ///
    /// Use [`QueryParams`] to filter by user, plan, page, or limit.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use cpanel_rs::{CpanelClient, types::QueryParams};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), cpanel_rs::CpanelError> {
    ///     let client = CpanelClient::simple("server.example.com", 2087, "root", "pass").unwrap();
    ///
    ///     // List accounts using the "premium" plan, page 1, 50 per page
    ///     let params = QueryParams::new().plan("premium").page(1).limit(50);
    ///     let accounts = client.list_accounts_paginated(&params).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn list_accounts_paginated(
        &self,
        params: &QueryParams,
    ) -> Result<Vec<AccountInfo>, CpanelError> {
        let pairs: Vec<(&str, String)> = params.to_pairs();
        let pairs: Vec<(&str, &str)> = pairs.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let result: Vec<AccountInfo> = self.whm("listaccts", &pairs).await?;
        Ok(result)
    }

    /// Returns detailed information about a single account.
    ///
    /// # Arguments
    ///
    /// * `username` — The cPanel username to look up.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use cpanel_rs::CpanelClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), cpanel_rs::CpanelError> {
    ///     let client = CpanelClient::simple("server.example.com", 2087, "root", "pass").unwrap();
    ///     let info = client.get_account_info("john").await?;
    ///     println!("Domain: {}", info.domain);
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_account_info(&self, username: &str) -> Result<AccountInfo, CpanelError> {
        let result: AccountInfo = self.whm("getacctinfo", &[("user", username)]).await?;
        Ok(result)
    }

    /// Creates a new cPanel account.
    ///
    /// # Arguments
    ///
    /// * `create` — A [`CreateAccount`] struct with the desired parameters.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use cpanel_rs::{CpanelClient, models::CreateAccount};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), cpanel_rs::CpanelError> {
    ///     let client = CpanelClient::simple("server.example.com", 2087, "root", "pass").unwrap();
    ///
    ///     client.create_account(&CreateAccount {
    ///         username: "jane".into(),
    ///         password: "securepass".into(),
    ///         domain: "jane.example.com".into(),
    ///         plan: Some("default".into()),
    ///         max_sql: Some(3),
    ///         ..Default::default()
    ///     }).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn create_account(&self, create: &CreateAccount) -> Result<(), CpanelError> {
        let username_s = create.username.clone();
        let password_s = create.password.clone();
        let domain_s = create.domain.clone();
        let plan_s = create.plan.clone();
        let ip_s = create.ip.clone();
        let contact_s = create.contact_email.clone();
        let max_ftp_s = create.max_ftp.map(|v| v.to_string());
        let max_pop_s = create.max_pop.map(|v| v.to_string());
        let max_sql_s = create.max_sql.map(|v| v.to_string());

        let mut params: Vec<(&str, &str)> = vec![
            ("username", &username_s),
            ("password", &password_s),
            ("domain", &domain_s),
        ];
        if let Some(ref s) = plan_s {
            params.push(("plan", s));
        }
        if let Some(ref s) = ip_s {
            params.push(("ip", s));
        }
        if let Some(ref s) = contact_s {
            params.push(("contact_email", s));
        }
        if let Some(ref s) = max_ftp_s {
            params.push(("max_ftp", s));
        }
        if let Some(ref s) = max_pop_s {
            params.push(("max_pop", s));
        }
        if let Some(ref s) = max_sql_s {
            params.push(("max_sql", s));
        }

        let _: () = self.whm("addacct", &params).await?;
        Ok(())
    }

    /// Deletes a cPanel account and all its data.
    ///
    /// **Warning:** This operation is irreversible. All files, databases, emails,
    /// and DNS records for the account will be permanently deleted.
    ///
    /// # Arguments
    ///
    /// * `username` — The username of the account to delete.
    pub async fn delete_account(&self, username: &str) -> Result<(), CpanelError> {
        let _: () = self.whm("delacct", &[("user", username)]).await?;
        Ok(())
    }

    /// Suspends a cPanel account.
    ///
    /// A suspended account cannot access cPanel, FTP, email, or any services.
    /// The account's data is preserved.
    ///
    /// # Arguments
    ///
    /// * `username` — The username of the account to suspend.
    pub async fn suspend_account(&self, username: &str) -> Result<(), CpanelError> {
        let _: () = self.whm("suspendacct", &[("user", username)]).await?;
        Ok(())
    }

    /// Unsuspends a previously suspended cPanel account.
    ///
    /// # Arguments
    ///
    /// * `username` — The username of the account to unsuspend.
    pub async fn unsuspend_account(&self, username: &str) -> Result<(), CpanelError> {
        let _: () = self.whm("unsuspendacct", &[("user", username)]).await?;
        Ok(())
    }

    /// Modifies an existing cPanel account.
    ///
    /// Use [`ModifyAccount`] to specify which fields to change. Only non-`None`
    /// fields in the struct will be updated.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use cpanel_rs::{CpanelClient, account::ModifyAccount};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), cpanel_rs::CpanelError> {
    ///     let client = CpanelClient::simple("server.example.com", 2087, "root", "pass").unwrap();
    ///
    ///     // Upgrade plan
    ///     client.modify_account(&ModifyAccount {
    ///         username: "john".into(),
    ///         plan: Some("premium".into()),
    ///         ..Default::default()
    ///     }).await?;
    ///
    ///     // Suspend
    ///     client.modify_account(&ModifyAccount {
    ///         username: "john".into(),
    ///         suspend: Some(true),
    ///         ..Default::default()
    ///     }).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn modify_account(&self, modify: &ModifyAccount) -> Result<(), CpanelError> {
        let user_s = modify.username.clone();
        let plan_s = modify.plan.clone();
        let ip_s = modify.ip.clone();
        let suspend_s = modify.suspend.map(|v| v.to_string());

        let mut params: Vec<(&str, &str)> = vec![("user", &user_s)];
        if let Some(ref s) = plan_s {
            params.push(("plan", s));
        }
        if let Some(ref s) = ip_s {
            params.push(("ip", s));
        }
        if let Some(ref s) = suspend_s {
            params.push(("suspend", s));
        }

        let _: () = self.whm("modifyacct", &params).await?;
        Ok(())
    }

    /// Changes the password for a cPanel account.
    ///
    /// # Arguments
    ///
    /// * `username` — The account username.
    /// * `password` — The new password.
    pub async fn change_password(&self, username: &str, password: &str) -> Result<(), CpanelError> {
        let _: () = self
            .whm("chpass", &[("user", username), ("password", password)])
            .await?;
        Ok(())
    }
}
