//! Backup management via cPanel UAPI.
//!
//! These methods operate on a per-user basis (cPanel port **2083**) and manage
//! account backups through the cPanel Backup system.
//!
//! # Overview
//!
//! | Method | UAPI Call | Description |
//! |--------|-----------|-------------|
//! | [`CpanelClient::get_backup_info`] | `Backup.get_config` | Get backup configuration |
//! | [`CpanelClient::create_backup`] | `Backup.create_backup` | Trigger a backup |
//! | [`CpanelClient::delete_backup`] | `Backup.delete_backup` | Delete a backup |
//! | [`CpanelClient::list_backups`] | `Backup.list_backups` | List available backups |
//! | [`CpanelClient::restore_backup`] | `Backup.restore_backup` | Restore from a backup |
//! | [`CpanelClient::get_backup_schedule`] | `Backup.get_config` | Get backup schedule config |
//! | [`CpanelClient::set_backup_schedule`] | `Backup.configure_backups` | Modify backup schedule |
//!
//! # Backup Locations
//!
//! cPanel supports several backup destinations:
//!
//! | Location | Description |
//! |----------|-------------|
//! | `local` | Server local storage (`/backup`) |
//! | `remote` | Remote FTP/SCP/SFTP server |
//! | `rsync` | Remote server via rsync |
//!
//! # Example
//!
//! ```no_run
//! use cpanel_rs::{CpanelClient, models::{CreateBackup, BackupSchedule}};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), cpanel_rs::CpanelError> {
//!     let client = CpanelClient::simple("server.example.com", 2083, "cpanel_user", "pass").unwrap();
//!
//!     // Check backup configuration
//!     let info = client.get_backup_info("cpanel_user").await?;
//!     println!("Last backup: {:?}", info.last_backup);
//!
//!     // Create a full backup
//!     client.create_backup("cpanel_user", &CreateBackup {
//!         full: Some(true),
//!         location: Some("local".into()),
//!         email: Some("admin@example.com".into()),
//!     }).await?;
//!
//!     // List available backups
//!     let backups = client.list_backups("cpanel_user").await?;
//!     for backup in &backups {
//!         println!("{} — {} — {:?}", backup.location.as_deref().unwrap_or("local"), backup.size.as_deref().unwrap_or(&String::from("unknown")), backup.incremental);
//!     }
//!
//!     // Configure backup schedule
//!     client.set_backup_schedule("cpanel_user", &BackupSchedule {
//!         frequency: Some("daily".into()),
//!         hour: Some(2),  // 2 AM
//!         email: Some("admin@example.com".into()),
//!         day_of_week: None,
//!         day_of_month: None,
//!     }).await?;
//!
//!     Ok(())
//! }
//! ```

use crate::{
    models::{BackupInfo, BackupSchedule, CreateBackup},
    CpanelClient, CpanelError,
};

impl CpanelClient {
    /// Gets backup configuration and info for a cPanel account.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    ///
    /// # Returns
    ///
    /// A [`BackupInfo`] struct with the backup location, last backup time,
    /// size, and whether incremental backups are enabled.
    pub async fn get_backup_info(&self, user: &str) -> Result<BackupInfo, CpanelError> {
        let result: BackupInfo = self.uapi("Backup", "get_config", &[("user", user)]).await?;
        Ok(result)
    }

    /// Creates a backup for a cPanel account.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `create` — A [`CreateBackup`] with backup options.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use cpanel_rs::{CpanelClient, models::CreateBackup};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), cpanel_rs::CpanelError> {
    ///     let client = CpanelClient::simple("server.example.com", 2083, "user", "pass").unwrap();
    ///
    ///     // Full backup to local storage
    ///     client.create_backup("user", &CreateBackup {
    ///         full: Some(true),
    ///         location: Some("local".into()),
    ///         ..Default::default()
    ///     }).await?;
    ///
    ///     // Partial backup to remote storage with email notification
    ///     client.create_backup("user", &CreateBackup {
    ///         full: Some(false),
    ///         location: Some("remote".into()),
    ///         email: Some("admin@example.com".into()),
    ///         ..Default::default()
    ///     }).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn create_backup(
        &self,
        user: &str,
        create: &CreateBackup,
    ) -> Result<(), CpanelError> {
        let user_s = user.to_string();
        let full_s = create.full.map(|v| v.to_string());
        let location_s = create.location.clone();
        let email_s = create.email.clone();

        let mut params: Vec<(&str, &str)> = vec![("user", &user_s)];
        if let Some(ref s) = full_s {
            params.push(("full", s));
        }
        if let Some(ref s) = location_s {
            params.push(("location", s));
        }
        if let Some(ref s) = email_s {
            params.push(("email", s));
        }

        let _: () = self.uapi("Backup", "create_backup", &params).await?;
        Ok(())
    }

    /// Deletes a backup by its ID.
    ///
    /// **Warning:** This operation is irreversible.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `backup_id` — The backup ID as returned by [`list_backups`](CpanelClient::list_backups).
    pub async fn delete_backup(&self, user: &str, backup_id: &str) -> Result<(), CpanelError> {
        let _: () = self
            .uapi(
                "Backup",
                "delete_backup",
                &[("user", user), ("backupid", backup_id)],
            )
            .await?;
        Ok(())
    }

    /// Lists all available backups for a cPanel account.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    ///
    /// # Returns
    ///
    /// A vector of [`BackupInfo`] structs, one per backup.
    pub async fn list_backups(&self, user: &str) -> Result<Vec<BackupInfo>, CpanelError> {
        let result: Vec<BackupInfo> = self
            .uapi("Backup", "list_backups", &[("user", user)])
            .await?;
        Ok(result)
    }

    /// Restores a cPanel account from a backup.
    ///
    /// **Warning:** This will overwrite the current account data with the backup contents.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `backup_id` — The backup ID to restore from.
    pub async fn restore_backup(&self, user: &str, backup_id: &str) -> Result<(), CpanelError> {
        let _: () = self
            .uapi(
                "Backup",
                "restore_backup",
                &[("user", user), ("backupid", backup_id)],
            )
            .await?;
        Ok(())
    }

    /// Gets the current backup schedule configuration.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    pub async fn get_backup_schedule(&self, user: &str) -> Result<BackupSchedule, CpanelError> {
        let result: BackupSchedule = self.uapi("Backup", "get_config", &[("user", user)]).await?;
        Ok(result)
    }

    /// Sets the backup schedule for a cPanel account.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `schedule` — A [`BackupSchedule`] with the desired schedule settings.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use cpanel_rs::{CpanelClient, models::BackupSchedule};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), cpanel_rs::CpanelError> {
    ///     let client = CpanelClient::simple("server.example.com", 2083, "user", "pass").unwrap();
    ///
    ///     // Daily backups at 3 AM with email notification
    ///     client.set_backup_schedule("user", &BackupSchedule {
    ///         frequency: Some("daily".into()),
    ///         hour: Some(3),
    ///         email: Some("admin@example.com".into()),
    ///         day_of_week: None,
    ///         day_of_month: None,
    ///     }).await?;
    ///
    ///     // Weekly backups on Sunday at 2 AM
    ///     client.set_backup_schedule("user", &BackupSchedule {
    ///         frequency: Some("weekly".into()),
    ///         day_of_week: Some(0), // Sunday
    ///         hour: Some(2),
    ///         email: None,
    ///         day_of_month: None,
    ///     }).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn set_backup_schedule(
        &self,
        user: &str,
        schedule: &BackupSchedule,
    ) -> Result<(), CpanelError> {
        let frequency_s = schedule.frequency.clone();
        let day_week_s = schedule.day_of_week.map(|v| v.to_string());
        let day_month_s = schedule.day_of_month.map(|v| v.to_string());
        let hour_s = schedule.hour.map(|v| v.to_string());
        let email_s = schedule.email.clone();

        let mut params: Vec<(&str, &str)> = vec![("user", user)];
        if let Some(ref s) = frequency_s {
            params.push(("frequency", s));
        }
        if let Some(ref s) = day_week_s {
            params.push(("dayofweek", s));
        }
        if let Some(ref s) = day_month_s {
            params.push(("dayofmonth", s));
        }
        if let Some(ref s) = hour_s {
            params.push(("hour", s));
        }
        if let Some(ref s) = email_s {
            params.push(("email", s));
        }

        let _: () = self.uapi("Backup", "configure_backups", &params).await?;
        Ok(())
    }
}
