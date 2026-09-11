//! FTP account management via cPanel UAPI.
//!
//! These methods operate on a per-user basis (cPanel port **2083**) and manage
//! FTP accounts, including listing, creation, modification, and deletion.
//!
//! # Overview
//!
//! | Method | UAPI Call | Description |
//! |--------|-----------|-------------|
//! | [`CpanelClient::list_ftp_accounts`] | `Ftp.list_ftp_accounts` | List all FTP accounts |
//! | [`CpanelClient::create_ftp_account`] | `Ftp.add_ftp` | Create a new FTP account |
//! | [`CpanelClient::modify_ftp_account`] | `Ftp.edit_ftp` | Modify an FTP account |
//! | [`CpanelClient::delete_ftp_account`] | `Ftp.del_ftp` | Delete an FTP account |
//!
//! # Example
//!
//! ```no_run
//! use cpanel_rs::{CpanelClient, models::{CreateFtp, ModifyFtp}};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), cpanel_rs::CpanelError> {
//!     let client = CpanelClient::simple("server.example.com", 2083, "cpanel_user", "pass").unwrap();
//!
//!     // List all FTP accounts
//!     let accounts = client.list_ftp_accounts("cpanel_user").await?;
//!     for acct in &accounts {
//!         println!("{} — {:?} — quota: {:?}", acct.username, acct.path, acct.quota);
//!     }
//!
//!     // Create a new FTP account with a 500 MB quota
//!     client.create_ftp_account("cpanel_user", &CreateFtp {
//!         username: "fileuser".into(),
//!         password: "secureftppass".into(),
//!         path: Some("/public_html/uploads".into()),
//!         quota: Some(500),
//!     }).await?;
//!
//!     // Modify the quota
//!     client.modify_ftp_account("cpanel_user", &ModifyFtp {
//!         username: "fileuser".into(),
//!         quota: Some(1000),
//!         ..Default::default()
//!     }).await?;
//!
//!     // Delete the FTP account
//!     client.delete_ftp_account("cpanel_user", "fileuser").await?;
//!
//!     Ok(())
//! }
//! ```

use crate::{
    models::{CreateFtp, FtpAccount, ModifyFtp},
    CpanelClient, CpanelError,
};

impl CpanelClient {
    /// Lists all FTP accounts for a cPanel account.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    pub async fn list_ftp_accounts(&self, user: &str) -> Result<Vec<FtpAccount>, CpanelError> {
        let result: Vec<FtpAccount> = self
            .uapi("Ftp", "list_ftp_accounts", &[("user", user)])
            .await?;
        Ok(result)
    }

    /// Creates a new FTP account.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `create` — A [`CreateFtp`] with the account details.
    pub async fn create_ftp_account(
        &self,
        user: &str,
        create: &CreateFtp,
    ) -> Result<(), CpanelError> {
        let mut params: Vec<(&str, String)> = vec![
            ("user", user.to_string()),
            ("login", create.username.clone()),
            ("password", create.password.clone()),
        ];
        if let Some(ref s) = create.path {
            params.push(("path", s.clone()));
        }
        if let Some(q) = create.quota {
            params.push(("quota", q.to_string()));
        }
        let params: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let _: () = self.uapi("Ftp", "add_ftp", &params).await?;
        Ok(())
    }

    /// Modifies an existing FTP account.
    ///
    /// Only non-`None` fields in [`ModifyFtp`] will be updated.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `modify` — A [`ModifyFtp`] with the account username and fields to change.
    pub async fn modify_ftp_account(
        &self,
        user: &str,
        modify: &ModifyFtp,
    ) -> Result<(), CpanelError> {
        let mut params: Vec<(&str, String)> = vec![
            ("user", user.to_string()),
            ("login", modify.username.clone()),
        ];
        if let Some(ref s) = modify.password {
            params.push(("password", s.clone()));
        }
        if let Some(ref s) = modify.path {
            params.push(("path", s.clone()));
        }
        if let Some(q) = modify.quota {
            params.push(("quota", q.to_string()));
        }
        let params: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let _: () = self.uapi("Ftp", "edit_ftp", &params).await?;
        Ok(())
    }

    /// Deletes an FTP account.
    ///
    /// **Warning:** This operation is irreversible. The FTP user's files are NOT deleted.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `username` — The FTP username to delete.
    pub async fn delete_ftp_account(&self, user: &str, username: &str) -> Result<(), CpanelError> {
        let _: () = self
            .uapi("Ftp", "del_ftp", &[("user", user), ("login", username)])
            .await?;
        Ok(())
    }
}
