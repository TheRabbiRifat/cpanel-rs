//! Cron job management via cPanel UAPI.
//!
//! These methods operate on a per-user basis (cPanel port **2083**) and manage
//! scheduled cron jobs for the account.
//!
//! # Overview
//!
//! | Method | UAPI Call | Description |
//! |--------|-----------|-------------|
//! | [`CpanelClient::list_cron_jobs`] | `Cron.list_cron_jobs` | List all cron jobs |
//! | [`CpanelClient::add_cron_job`] | `Cron.add_cron_job` | Create a new cron job |
//! | [`CpanelClient::edit_cron_job`] | `Cron.edit_cron_job` | Modify an existing cron job |
//! | [`CpanelClient::delete_cron_job`] | `Cron.delete_cron_job` | Delete a cron job |
//!
//! # Cron Schedule Fields
//!
//! | Field | Values | Description |
//! |-------|--------|-------------|
//! | `minute` | `0-59`, `*`, `/N` | Minute of hour |
//! | `hour` | `0-23`, `*`, `/N` | Hour of day |
//! | `day` | `1-31`, `*`, `/N` | Day of month |
//! | `month` | `1-12`, `*`, `/N` | Month of year |
//! | `weekday` | `0-7`, `*` | Day of week (0 and 7 = Sunday) |
//!
//! # Example
//!
//! ```no_run
//! use cpanel_rs::{CpanelClient, models::{AddCronJob, EditCronJob}};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), cpanel_rs::CpanelError> {
//!     let client = CpanelClient::simple("server.example.com", 2083, "cpanel_user", "pass").unwrap();
//!
//!     // List all cron jobs
//!     let jobs = client.list_cron_jobs("cpanel_user").await?;
//!     for job in &jobs {
//!         println!("{} — {} — {:?}", job.minute, job.command, job.email);
//!     }
//!
//!     // Add a cron job (every day at 2:30 AM)
//!     client.add_cron_job("cpanel_user", &AddCronJob {
//!         minute: "30".into(),
//!         hour: "2".into(),
//!         day: "*".into(),
//!         month: "*".into(),
//!         weekday: "*".into(),
//!         command: "/usr/bin/php /home/cpanel_user/script.php".into(),
//!         email: Some("admin@example.com".into()),
//!     }).await?;
//!
//!     // Delete a cron job by its ID
//!     if let Some(first) = jobs.first() {
//!         client.delete_cron_job("cpanel_user", &first.id).await?;
//!     }
//!
//!     Ok(())
//! }
//! ```

use crate::{
    models::{AddCronJob, CronJob, EditCronJob},
    CpanelClient, CpanelError,
};

impl CpanelClient {
    /// Lists all cron jobs for a cPanel account.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    pub async fn list_cron_jobs(&self, user: &str) -> Result<Vec<CronJob>, CpanelError> {
        let result: Vec<CronJob> = self
            .uapi("Cron", "list_cron_jobs", &[("user", user)])
            .await?;
        Ok(result)
    }

    /// Adds a new cron job.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `job` — An [`AddCronJob`] with the schedule and command details.
    pub async fn add_cron_job(&self, user: &str, job: &AddCronJob) -> Result<(), CpanelError> {
        let params: Vec<(&str, &str)> = vec![
            ("user", user),
            ("minute", &job.minute),
            ("hour", &job.hour),
            ("day", &job.day),
            ("month", &job.month),
            ("weekday", &job.weekday),
            ("command", &job.command),
        ];
        let _: () = self.uapi("Cron", "add_cron_job", &params).await?;
        Ok(())
    }

    /// Edits an existing cron job.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `edit` — An [`EditCronJob`] with the job ID and fields to update.
    pub async fn edit_cron_job(&self, user: &str, edit: &EditCronJob) -> Result<(), CpanelError> {
        let mut params: Vec<(&str, &str)> = vec![("user", user), ("job", &edit.job)];
        if let Some(ref s) = edit.minute {
            params.push(("minute", s));
        }
        if let Some(ref s) = edit.hour {
            params.push(("hour", s));
        }
        if let Some(ref s) = edit.day {
            params.push(("day", s));
        }
        if let Some(ref s) = edit.month {
            params.push(("month", s));
        }
        if let Some(ref s) = edit.weekday {
            params.push(("weekday", s));
        }
        if let Some(ref s) = edit.command {
            params.push(("command", s));
        }
        if let Some(ref s) = edit.email {
            params.push(("email", s));
        }
        let _: () = self.uapi("Cron", "edit_cron_job", &params).await?;
        Ok(())
    }

    /// Deletes a cron job by its job ID.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `job_id` — The cron job ID as returned by [`list_cron_jobs`](CpanelClient::list_cron_jobs).
    pub async fn delete_cron_job(&self, user: &str, job_id: &str) -> Result<(), CpanelError> {
        let _: () = self
            .uapi(
                "Cron",
                "delete_cron_job",
                &[("user", user), ("job", job_id)],
            )
            .await?;
        Ok(())
    }
}
