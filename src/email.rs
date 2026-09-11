//! Email management via cPanel UAPI.
//!
//! These methods operate on a per-user basis (cPanel port **2083**) and manage
//! email accounts, forwarders, autoresponders, and mailing lists.
//!
//! # Overview
//!
//! | Module | Method | UAPI Call | Description |
//! |--------|--------|-----------|-------------|
//! | Email | [`CpanelClient::list_email_accounts`] | `list_popaccounts` | List POP/IMAP accounts |
//! | Email | [`CpanelClient::create_email`] | `add_popaccount` | Create an email account |
//! | Email | [`CpanelClient::delete_email`] | `del_popaccount` | Delete an email account |
//! | Email | [`CpanelClient::change_email_password`] | `chpoppassword` | Change email password |
//! | Email | [`CpanelClient::list_forwarders`] | `list_forwarders` | List mail forwarders |
//! | Email | [`CpanelClient::create_forwarder`] | `add_forwarder` | Create a forwarder |
//! | Email | [`CpanelClient::delete_forwarder`] | `del_forwarder` | Delete a forwarder |
//! | Email | [`CpanelClient::list_autoresponders`] | `list_autoreponders` | List autoresponders |
//! | Email | [`CpanelClient::create_autoresponder`] | `add_autoresponder` | Create an autoresponder |
//! | Email | [`CpanelClient::delete_autoresponder`] | `del_autoresponder` | Delete an autoresponder |
//! | Email | [`CpanelClient::list_mailing_lists`] | `list_listaccounts` | List mailing lists |
//! | Email | [`CpanelClient::create_mailing_list`] | `add_listaccount` | Create a mailing list |
//! | Email | [`CpanelClient::delete_mailing_list`] | `del_listaccount` | Delete a mailing list |
//!
//! # Example
//!
//! ```no_run
//! use cpanel_rs::{CpanelClient, models::{CreateEmail, Forwarder, CreateAutoResponder}};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), cpanel_rs::CpanelError> {
//!     let client = CpanelClient::simple("server.example.com", 2083, "cpanel_user", "pass").unwrap();
//!
//!     // Create an email account
//!     client.create_email("cpanel_user", "example.com", &CreateEmail {
//!         address: "info@example.com".into(),
//!         password: "securepass".into(),
//!         max_quota: Some(500),
//!     }).await?;
//!
//!     // Create a forwarder
//!     client.create_forwarder("cpanel_user", "example.com", &Forwarder {
//!         address: "old@example.com".into(),
//!         destination: "new@example.com".into(),
//!         type_: "forward".into(),
//!     }).await?;
//!
//!     // Create an autoresponder
//!     client.create_autoresponder("cpanel_user", "example.com", &CreateAutoResponder {
//!         email: "info@example.com".into(),
//!         subject: "Out of Office".into(),
//!         message: "I am currently unavailable.".into(),
//!         active: Some(true),
//!         ..Default::default()
//!     }).await?;
//!
//!     Ok(())
//! }
//! ```

use crate::{
    models::{
        AutoResponder, CreateAutoResponder, CreateEmail, CreateMailingList, EmailAccount,
        Forwarder, MailingList,
    },
    CpanelClient, CpanelError,
};

impl CpanelClient {
    /// Lists all email (POP/IMAP) accounts for a domain.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `domain` — The domain name (e.g. `"example.com"`).
    pub async fn list_email_accounts(
        &self,
        user: &str,
        domain: &str,
    ) -> Result<Vec<EmailAccount>, CpanelError> {
        let result: Vec<EmailAccount> = self
            .uapi(
                "Email",
                "list_popaccounts",
                &[("user", user), ("domain", domain)],
            )
            .await?;
        Ok(result)
    }

    /// Creates a new email account.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `domain` — The domain for the email address.
    /// * `create` — A [`CreateEmail`] struct with the account details.
    pub async fn create_email(
        &self,
        user: &str,
        domain: &str,
        create: &CreateEmail,
    ) -> Result<(), CpanelError> {
        let user_s = user.to_string();
        let domain_s = domain.to_string();
        let quota_s = create.max_quota.map(|v| v.to_string());

        let mut params: Vec<(&str, &str)> = vec![
            ("user", &user_s),
            ("domain", &domain_s),
            ("email", &create.address),
            ("password", &create.password),
        ];
        if let Some(ref s) = quota_s {
            params.push(("maxquota", s));
        }

        let _: () = self.uapi("Email", "add_popaccount", &params).await?;
        Ok(())
    }

    /// Deletes an email account.
    ///
    /// **Warning:** This operation is irreversible. All mailbox data will be deleted.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `domain` — The domain of the email account.
    /// * `email` — The full email address to delete.
    pub async fn delete_email(
        &self,
        user: &str,
        domain: &str,
        email: &str,
    ) -> Result<(), CpanelError> {
        let _: () = self
            .uapi(
                "Email",
                "del_popaccount",
                &[("user", user), ("domain", domain), ("email", email)],
            )
            .await?;
        Ok(())
    }

    /// Changes the password for an email account.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `domain` — The domain of the email account.
    /// * `email` — The full email address.
    /// * `password` — The new password.
    pub async fn change_email_password(
        &self,
        user: &str,
        domain: &str,
        email: &str,
        password: &str,
    ) -> Result<(), CpanelError> {
        let _: () = self
            .uapi(
                "Email",
                "chpoppassword",
                &[
                    ("user", user),
                    ("domain", domain),
                    ("email", email),
                    ("password", password),
                ],
            )
            .await?;
        Ok(())
    }

    /// Lists all mail forwarders for a domain.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `domain` — The domain name.
    pub async fn list_forwarders(
        &self,
        user: &str,
        domain: &str,
    ) -> Result<Vec<Forwarder>, CpanelError> {
        let result: Vec<Forwarder> = self
            .uapi(
                "Email",
                "list_forwarders",
                &[("user", user), ("domain", domain)],
            )
            .await?;
        Ok(result)
    }

    /// Creates a mail forwarder.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `domain` — The domain name.
    /// * `forward` — A [`Forwarder`] with the source and destination addresses.
    pub async fn create_forwarder(
        &self,
        user: &str,
        domain: &str,
        forward: &Forwarder,
    ) -> Result<(), CpanelError> {
        let params: Vec<(&str, &str)> = vec![
            ("user", user),
            ("domain", domain),
            ("address", &forward.address),
            ("destination", &forward.destination),
            ("type", &forward.type_),
        ];
        let _: () = self.uapi("Email", "add_forwarder", &params).await?;
        Ok(())
    }

    /// Deletes a mail forwarder.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `domain` — The domain name.
    /// * `address` — The source email address of the forwarder.
    pub async fn delete_forwarder(
        &self,
        user: &str,
        domain: &str,
        address: &str,
    ) -> Result<(), CpanelError> {
        let _: () = self
            .uapi(
                "Email",
                "del_forwarder",
                &[("user", user), ("domain", domain), ("address", address)],
            )
            .await?;
        Ok(())
    }

    /// Lists all autoresponders for a domain.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `domain` — The domain name.
    pub async fn list_autoresponders(
        &self,
        user: &str,
        domain: &str,
    ) -> Result<Vec<AutoResponder>, CpanelError> {
        let result: Vec<AutoResponder> = self
            .uapi(
                "Email",
                "list_autoreponders",
                &[("user", user), ("domain", domain)],
            )
            .await?;
        Ok(result)
    }

    /// Creates an autoresponder for an email account.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `domain` — The domain name.
    /// * `create` — A [`CreateAutoResponder`] with the responder details.
    pub async fn create_autoresponder(
        &self,
        user: &str,
        domain: &str,
        create: &CreateAutoResponder,
    ) -> Result<(), CpanelError> {
        let user_s = user.to_string();
        let domain_s = domain.to_string();
        let start_s = create.start_date.clone();
        let end_s = create.end_date.clone();
        let active_s = create.active.map(|v| v.to_string());

        let mut params: Vec<(&str, &str)> = vec![
            ("user", &user_s),
            ("domain", &domain_s),
            ("email", &create.email),
            ("subject", &create.subject),
            ("message", &create.message),
        ];
        if let Some(ref s) = start_s {
            params.push(("startdate", s));
        }
        if let Some(ref s) = end_s {
            params.push(("enddate", s));
        }
        if let Some(ref s) = active_s {
            params.push(("active", s));
        }

        let _: () = self.uapi("Email", "add_autoresponder", &params).await?;
        Ok(())
    }

    /// Deletes an autoresponder.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `domain` — The domain name.
    /// * `email` — The email address the autoresponder is attached to.
    pub async fn delete_autoresponder(
        &self,
        user: &str,
        domain: &str,
        email: &str,
    ) -> Result<(), CpanelError> {
        let _: () = self
            .uapi(
                "Email",
                "del_autoresponder",
                &[("user", user), ("domain", domain), ("email", email)],
            )
            .await?;
        Ok(())
    }

    /// Lists all mailing lists for a cPanel account.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    pub async fn list_mailing_lists(&self, user: &str) -> Result<Vec<MailingList>, CpanelError> {
        let result: Vec<MailingList> = self
            .uapi("Email", "list_listaccounts", &[("user", user)])
            .await?;
        Ok(result)
    }

    /// Creates a new mailing list.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `create` — A [`CreateMailingList`] with the list details.
    pub async fn create_mailing_list(
        &self,
        user: &str,
        create: &CreateMailingList,
    ) -> Result<(), CpanelError> {
        let user_s = user.to_string();
        let desc_s = create.description.clone();
        let moderated_s = create.moderated.map(|v| v.to_string());

        let mut params: Vec<(&str, &str)> = vec![
            ("user", &user_s),
            ("name", &create.name),
            ("email", &create.email),
            ("password", &create.password),
        ];
        if let Some(ref s) = desc_s {
            params.push(("description", s));
        }
        if let Some(ref s) = moderated_s {
            params.push(("moderated", s));
        }

        let _: () = self.uapi("Email", "add_listaccount", &params).await?;
        Ok(())
    }

    /// Deletes a mailing list and all its subscribers.
    ///
    /// **Warning:** This operation is irreversible.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `list_name` — The name of the mailing list to delete.
    pub async fn delete_mailing_list(
        &self,
        user: &str,
        list_name: &str,
    ) -> Result<(), CpanelError> {
        let _: () = self
            .uapi(
                "Email",
                "del_listaccount",
                &[("user", user), ("list", list_name)],
            )
            .await?;
        Ok(())
    }
}
