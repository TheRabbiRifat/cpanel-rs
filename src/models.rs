//! Data models used throughout the crate.
//!
//! Each struct here maps to a specific cPanel/WHM API response or request body.
//! Request structs (used for create/modify operations) implement [`serde::Serialize`].
//! Response structs implement [`serde::Deserialize`].

use serde::{Deserialize, Serialize};

// ═════════════════════════════════════════════════════════════════════════════
// Account models (WHM)
// ═════════════════════════════════════════════════════════════════════════════

/// Information returned by [`listaccts`](crate::CpanelClient::list_accounts) and [`getacctinfo`](crate::CpanelClient::get_account_info).
///
/// # Fields
///
/// * `username` — The account username
/// * `domain` — The primary domain
/// * `plan` — The resource plan name, if assigned
/// * `ip` — The assigned IP address
/// * `package` — Deprecated alias for `plan`
/// * `created` — Account creation timestamp
/// * `suspended` — Whether the account is currently suspended
/// * `has_root` — Whether the account has root access
#[derive(Debug, Clone, Deserialize)]
pub struct AccountInfo {
    /// The cPanel/WHM username.
    pub username: String,
    /// The primary domain name.
    pub domain: String,
    /// The resource plan name.
    pub plan: Option<String>,
    /// The assigned IP address.
    pub ip: Option<String>,
    /// The package name (deprecated, same as `plan`).
    pub package: Option<String>,
    /// The account creation date string.
    pub created: Option<String>,
    /// Whether the account is suspended.
    pub suspended: Option<bool>,
    /// Whether the account has root access.
    pub has_root: Option<bool>,
}

/// Parameters for creating a new cPanel account via [`addacct`](crate::CpanelClient::create_account).
///
/// All fields except `username`, `password`, and `domain` are optional.
///
/// # Example
///
/// ```
/// use cpanel_rs::models::CreateAccount;
///
/// let acct = CreateAccount {
///     username: "newuser".into(),
///     password: "securepass123".into(),
///     domain: "example.com".into(),
///     plan: Some("default".into()),
///     max_sql: Some(5),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Serialize, Default)]
pub struct CreateAccount {
    /// The desired username (max 8 characters for cPanel).
    pub username: String,
    /// The account password.
    pub password: String,
    /// The primary domain for the account.
    pub domain: String,
    /// The resource plan to assign.
    pub plan: Option<String>,
    /// The IP address to assign.
    pub ip: Option<String>,
    /// Contact email address.
    pub contact_email: Option<String>,
    /// Maximum FTP accounts.
    pub max_ftp: Option<i32>,
    /// Maximum POP/IMAP accounts.
    pub max_pop: Option<i32>,
    /// Maximum MySQL databases.
    pub max_sql: Option<i32>,
    /// Maximum Linux user-level processes.
    pub max_list: Option<i32>,
    /// Maximum subdomains.
    pub max_sub: Option<i32>,
    /// Maximum parked domains.
    pub maxpark: Option<i32>,
    /// Qualified domain name (qmail).
    pub qname: Option<String>,
    /// Over-limit behavior (`warn` or `kill`).
    pub overlimit: Option<String>,
    /// Skip resource limit checks.
    pub skiplimits: Option<bool>,
}

/// Parameters for modifying an existing account via [`modifyacct`](crate::CpanelClient::modify_account).
///
/// # Example
///
/// ```
/// use cpanel_rs::models::ModifyAccount;
///
/// let modify = ModifyAccount {
///     username: "existinguser".into(),
///     plan: Some("premium".into()),
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

// ═════════════════════════════════════════════════════════════════════════════
// DNS models (UAPI)
// ═════════════════════════════════════════════════════════════════════════════

/// A single DNS record as returned by [`list_zone_records`](crate::CpanelClient::list_dns_records).
///
/// # Fields
///
/// * `name` — The record name (e.g. `www`, `@`)
/// * `type_` — The record type (`A`, `AAAA`, `CNAME`, `MX`, `TXT`, etc.)
/// * `class` — The DNS class (usually `IN`)
/// * `ttl` — Time-to-live in seconds
/// * `value` — The record value (IP address, hostname, etc.)
#[derive(Debug, Clone, Deserialize)]
pub struct DnsRecord {
    /// The record name.
    pub name: String,
    /// The record type.
    pub type_: String,
    /// The DNS class.
    pub class: String,
    /// TTL in seconds.
    pub ttl: String,
    /// The record data/value.
    pub value: String,
}

/// Parameters for adding a DNS record via [`add_zone_record`](crate::CpanelClient::add_dns_record).
///
/// # Example
///
/// ```
/// use cpanel_rs::models::AddDnsRecord;
///
/// let record = AddDnsRecord {
///     type_: "A".into(),
///     name: "www".into(),
///     value: "1.2.3.4".into(),
///     ttl: Some(3600),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Default)]
pub struct AddDnsRecord {
    /// The record type (`A`, `AAAA`, `CNAME`, `MX`, `TXT`, `SRV`, etc.).
    pub type_: String,
    /// The record name (e.g. `www`, `@`, `mail`).
    pub name: String,
    /// The record value (IP address, target host, etc.).
    pub value: String,
    /// TTL in seconds. Defaults to the zone's default TTL if not set.
    pub ttl: Option<u32>,
}

/// A DNS zone file returned by [`export_zone`](crate::CpanelClient::export_zone).
#[derive(Debug, Clone, Deserialize)]
pub struct ZoneFile {
    /// The zone name (e.g. `example.com`).
    pub zone: String,
    /// The full BIND-format zone file content.
    pub content: String,
}

// ═════════════════════════════════════════════════════════════════════════════
// Email models (UAPI)
// ═════════════════════════════════════════════════════════════════════════════

/// An email account as returned by [`list_popaccounts`](crate::CpanelClient::list_email_accounts).
///
/// # Fields
///
/// * `address` — The full email address
/// * `domain` — The domain
/// * `max_quota` — Maximum quota in MB (`None` = unlimited)
/// * `quota_used` — Current quota usage in MB
#[derive(Debug, Clone, Deserialize)]
pub struct EmailAccount {
    /// The email address (e.g. `user@example.com`).
    pub address: String,
    /// The domain.
    pub domain: String,
    /// Maximum quota in MB. `None` means unlimited.
    pub max_quota: Option<i64>,
    /// Current quota usage in MB.
    pub quota_used: Option<f64>,
}

/// Parameters for creating an email account via [`add_popaccount`](crate::CpanelClient::create_email).
///
/// # Example
///
/// ```
/// use cpanel_rs::models::CreateEmail;
///
/// let email = CreateEmail {
///     address: "info@example.com".into(),
///     password: "securepass".into(),
///     max_quota: Some(500), // 500 MB
/// };
/// ```
#[derive(Debug, Clone, Serialize, Default)]
pub struct CreateEmail {
    /// The full email address to create.
    pub address: String,
    /// The password for the email account.
    pub password: String,
    /// Maximum mailbox quota in MB. `None` = unlimited.
    pub max_quota: Option<i64>,
}

/// A mail forwarder as returned by [`list_forwarders`](crate::CpanelClient::list_forwarders).
///
/// # Fields
///
/// * `address` — The source email address
/// * `destination` — Where mail is forwarded to
/// * `type_` — The forwarder type (`forward` or `catchall`)
#[derive(Debug, Clone, Deserialize)]
pub struct Forwarder {
    /// The source email address.
    pub address: String,
    /// The destination address.
    pub destination: String,
    /// The forwarder type (`forward` or `catchall`).
    pub type_: String,
}

/// Parameters for creating an autoresponder via [`add_autoresponder`](crate::CpanelClient::create_autoresponder).
///
/// # Example
///
/// ```
/// use cpanel_rs::models::CreateAutoResponder;
///
/// let ar = CreateAutoResponder {
///     email: "user@example.com".into(),
///     subject: "Out of Office".into(),
///     message: "I am currently out of the office.".into(),
///     active: Some(true),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateAutoResponder {
    /// The email address to attach the autoresponder to.
    pub email: String,
    /// The subject line of the auto-reply.
    pub subject: String,
    /// The body of the auto-reply message.
    pub message: String,
    /// Start date in `YYYY-MM-DD` format.
    pub start_date: Option<String>,
    /// End date in `YYYY-MM-DD` format.
    pub end_date: Option<String>,
    /// Whether the autoresponder is initially active.
    pub active: Option<bool>,
}

/// An autoresponder as returned by [`list_autoreponders`](crate::CpanelClient::list_autoresponders).
#[derive(Debug, Clone, Deserialize)]
pub struct AutoResponder {
    /// The associated email address.
    pub email: String,
    /// The subject line.
    pub subject: String,
    /// The message body.
    pub message: String,
    /// Start date.
    pub start_date: Option<String>,
    /// End date.
    pub end_date: Option<String>,
    /// Whether the autoresponder is active.
    pub active: bool,
}

/// Parameters for creating a mailing list via [`add_listaccount`](crate::CpanelClient::create_mailing_list).
///
/// # Example
///
/// ```
/// use cpanel_rs::models::CreateMailingList;
///
/// let list = CreateMailingList {
///     name: "announcements".into(),
///     email: "announce@example.com".into(),
///     password: "listpass".into(),
///     description: Some("Site announcements".into()),
///     moderated: Some(true),
/// };
/// ```
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateMailingList {
    /// The list name (e.g. `announcements`).
    pub name: String,
    /// The full list email address.
    pub email: String,
    /// The list password.
    pub password: String,
    /// Description of the mailing list.
    pub description: Option<String>,
    /// Require moderator approval for posts.
    pub moderated: Option<bool>,
}

/// A mailing list as returned by [`list_listaccounts`](crate::CpanelClient::list_mailing_lists).
#[derive(Debug, Clone, Deserialize)]
pub struct MailingList {
    /// The list name.
    pub name: String,
    /// The list email address.
    pub email: String,
    /// Description.
    pub description: Option<String>,
    /// Whether moderation is enabled.
    pub moderated: Option<bool>,
    /// Number of subscribers.
    pub members: Option<usize>,
}

// ═════════════════════════════════════════════════════════════════════════════
// Database models (UAPI)
// ═════════════════════════════════════════════════════════════════════════════

/// A MySQL database as returned by [`list_dbs`](crate::CpanelClient::list_databases).
///
/// # Fields
///
/// * `name` — The database name (prefixed with the cPanel username, e.g. `user_db`)
/// * `size` — Size in MB
/// * `charset` — The character set
#[derive(Debug, Clone, Deserialize)]
pub struct Database {
    /// The database name.
    pub name: String,
    /// Size in MB.
    pub size: Option<f64>,
    /// The character set (e.g. `utf8_general_ci`).
    pub charset: Option<String>,
}

/// Parameters for creating a database via [`create_db`](crate::CpanelClient::create_database).
///
/// # Example
///
/// ```
/// use cpanel_rs::models::CreateDatabase;
///
/// let db = CreateDatabase {
///     name: "myapp".into(),
///     charset: Some("utf8mb4".into()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Default)]
pub struct CreateDatabase {
    /// The database name (without the cPanel username prefix — that is added automatically).
    pub name: String,
    /// The character set (e.g. `utf8mb4`, `utf8`).
    pub charset: Option<String>,
}

/// A privilege assignment as returned by [`list_user_privileges`](crate::CpanelClient::list_db_user_privileges).
#[derive(Debug, Clone, Deserialize)]
pub struct DbPrivilege {
    /// The database name.
    pub database: String,
    /// The database user.
    pub user: String,
    /// The granted privilege (e.g. `ALL`, `SELECT`, `INSERT`).
    pub privilege: String,
    /// The table name, or `None` for database-level privileges.
    pub table: Option<String>,
}

// ═════════════════════════════════════════════════════════════════════════════
// File Manager models (UAPI)
// ═════════════════════════════════════════════════════════════════════════════

/// A file or directory entry as returned by [`listdir`](crate::CpanelClient::list_files).
///
/// # Fields
///
/// * `name` — The file/directory name
/// * `type_` — `"file"` or `"dir"`
/// * `size` — File size in bytes (directories report total size)
/// * `mode` — Unix permission string (e.g. `0755`)
/// * `mtime` — Last modification time
/// * `path` — Full path to the entry
#[derive(Debug, Clone, Deserialize)]
pub struct FileManagerEntry {
    /// The file or directory name.
    pub name: String,
    /// The entry type (`"file"` or `"dir"`).
    pub type_: String,
    /// Size in bytes.
    pub size: Option<String>,
    /// Unix permission mode (e.g. `"0755"`).
    pub mode: Option<String>,
    /// Last modification timestamp.
    pub mtime: Option<String>,
    /// Full absolute path.
    pub path: Option<String>,
}

/// Parameters for creating a file via [`create_file`](crate::CpanelClient::create_file).
///
/// # Example
///
/// ```
/// use cpanel_rs::models::CreateFile;
///
/// let file = CreateFile {
///     path: "/public_html/index.html".into(),
///     content: Some("<h1>Hello</h1>".into()),
///     overwrite: false,
/// };
/// ```
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateFile {
    /// Full path where the file should be created (e.g. `/public_html/foo.txt`).
    pub path: String,
    /// File content. If `None`, an empty file is created.
    pub content: Option<String>,
    /// Overwrite the file if it already exists.
    pub overwrite: bool,
}

/// Parameters for renaming a file via [`rename`](crate::CpanelClient::rename_file).
///
/// # Example
///
/// ```
/// use cpanel_rs::models::RenameFile;
///
/// let rename = RenameFile {
///     source: "/public_html/old.txt".into(),
///     destination: "/public_html/new.txt".into(),
/// };
/// ```
#[derive(Debug, Clone, Default, Serialize)]
pub struct RenameFile {
    /// Current full path.
    pub source: String,
    /// New full path.
    pub destination: String,
}

/// Parameters for creating a directory via [`mkdir`](crate::CpanelClient::mkdir).
///
/// # Example
///
/// ```
/// use cpanel_rs::models::CreateDir;
///
/// let dir = CreateDir {
///     path: "/public_html/assets".into(),
/// };
/// ```
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateDir {
    /// Full path for the new directory.
    pub path: String,
}

/// Parameters for compressing files via [`compress`](crate::CpanelClient::compress).
///
/// # Example
///
/// ```
/// use cpanel_rs::models::CompressFiles;
///
/// let compress = CompressFiles {
///     paths: vec!["/home/user/public_html/wp-content/uploads".into()],
///     archive_name: "uploads_backup.zip".into(),
/// };
/// ```
#[derive(Debug, Clone, Default, Serialize)]
pub struct CompressFiles {
    /// List of file/directory paths to compress.
    pub paths: Vec<String>,
    /// Name of the resulting archive (e.g. `"backup.zip"`).
    pub archive_name: String,
}

// ═════════════════════════════════════════════════════════════════════════════
// SSL models (UAPI)
// ═════════════════════════════════════════════════════════════════════════════

/// An SSL certificate as returned by [`get_cert_info`](crate::CpanelClient::list_ssl_certs).
///
/// # Fields
///
/// * `domain` — The domain the certificate is for
/// * `issuer` — Certificate issuer
/// * `subject` — Certificate subject
/// * `not_before` — Not-before date
/// * `not_after` — Expiry date
/// * `active` — Whether the certificate is active
#[derive(Debug, Clone, Deserialize)]
pub struct SslCertificate {
    /// The domain name.
    pub domain: String,
    /// The issuing CA.
    pub issuer: Option<String>,
    /// The certificate subject.
    pub subject: Option<String>,
    /// Not-before date string.
    pub not_before: Option<String>,
    /// Expiry date string.
    pub not_after: Option<String>,
    /// Whether the certificate is currently active.
    pub active: Option<bool>,
}

/// Parameters for installing an SSL certificate via [`install_cert`](crate::CpanelClient::install_ssl).
///
/// # Example
///
/// ```
/// use cpanel_rs::models::InstallSsl;
///
/// let install = InstallSsl {
///     domain: "example.com".into(),
///     csr: "-----BEGIN CERTIFICATE REQUEST-----...".into(),
///     cert: "-----BEGIN CERTIFICATE-----...".into(),
///     ca: Some("-----BEGIN CERTIFICATE-----...".into()),
///     bundle: Some(true),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Default)]
pub struct InstallSsl {
    /// The domain to install the certificate for.
    pub domain: String,
    /// The CSR (Certificate Signing Request) in PEM format.
    pub csr: String,
    /// The certificate in PEM format.
    pub cert: String,
    /// The CA bundle in PEM format.
    pub ca: Option<String>,
    /// Whether to bundle the CA certificates with the response.
    pub bundle: Option<bool>,
}

// ═════════════════════════════════════════════════════════════════════════════
// Backup models (UAPI)
// ═════════════════════════════════════════════════════════════════════════════

/// Backup configuration/info as returned by [`get_config`](crate::CpanelClient::get_backup_info).
///
/// # Fields
///
/// * `location` — Backup storage location
/// * `last_backup` — Timestamp of the last backup
/// * `size` — Total backup size
/// * `incremental` — Whether incremental backups are enabled
#[derive(Debug, Clone, Deserialize)]
pub struct BackupInfo {
    /// The backup storage location.
    pub location: Option<String>,
    /// Timestamp of the last backup.
    pub last_backup: Option<String>,
    /// Total backup size.
    pub size: Option<String>,
    /// Whether incremental backups are enabled.
    pub incremental: Option<bool>,
}

/// Parameters for creating a backup via [`create_backup`](crate::CpanelClient::create_backup).
///
/// # Example
///
/// ```
/// use cpanel_rs::models::CreateBackup;
///
/// let backup = CreateBackup {
///     full: Some(true),       // full backup
///     location: Some("remote".into()), // remote storage
///     email: Some("admin@example.com".into()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Default)]
pub struct CreateBackup {
    /// If `Some(true)`, performs a full backup. Otherwise a partial backup.
    pub full: Option<bool>,
    /// Backup storage location (`local`, `remote`, etc.).
    pub location: Option<String>,
    /// Email address to notify on completion.
    pub email: Option<String>,
}

/// Backup schedule configuration as returned by [`get_config`](crate::CpanelClient::get_backup_schedule)
/// and set via [`configure_backups`](crate::CpanelClient::set_backup_schedule).
///
/// # Fields
///
/// * `frequency` — Backup frequency (`daily`, `weekly`, `monthly`)
/// * `day_of_week` — Day of week for weekly backups (0–6)
/// * `day_of_month` — Day of month for monthly backups (1–28)
/// * `hour` — Hour of day for the backup (0–23)
/// * `email` — Notification email
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BackupSchedule {
    /// Frequency: `"daily"`, `"weekly"`, or `"monthly"`.
    pub frequency: Option<String>,
    /// Day of week (0 = Sunday, 6 = Saturday) for weekly backups.
    pub day_of_week: Option<i32>,
    /// Day of month (1–28) for monthly backups.
    pub day_of_month: Option<i32>,
    /// Hour of day (0–23) to run the backup.
    pub hour: Option<i32>,
    /// Email address for notifications.
    pub email: Option<String>,
}

// ═════════════════════════════════════════════════════════════════════════════
// Resource usage models (UAPI)
// ═════════════════════════════════════════════════════════════════════════════

/// Detailed resource usage for a cPanel account as returned by [`get_quota`](crate::CpanelClient::get_user_quota).
///
/// # Fields
///
/// * `disk_used` — Disk space used in MB
/// * `disk_quota` — Disk quota in MB (`-1` = unlimited)
/// * `inode_used` — Inodes used
/// * `inode_quota` — Inode quota (`-1` = unlimited)
/// * `bandwidth_used` — Bandwidth used in MB
/// * `bandwidth_quota` — Bandwidth quota in MB
/// * `cpu_percent` — Current CPU usage percentage
#[derive(Debug, Clone, Deserialize)]
pub struct ResourceUsage {
    /// Disk space used in MB.
    pub disk_used: f64,
    /// Disk quota in MB (`-1` = unlimited).
    pub disk_quota: i64,
    /// Inodes used.
    pub inode_used: i64,
    /// Inode quota (`-1` = unlimited).
    pub inode_quota: i64,
    /// Bandwidth used in MB.
    pub bandwidth_used: Option<f64>,
    /// Bandwidth quota in MB.
    pub bandwidth_quota: Option<i64>,
    /// Current CPU usage percentage.
    pub cpu_percent: Option<f64>,
}

/// A PHP version handler as returned by [`list_versions`](crate::CpanelClient::list_php_versions).
///
/// # Fields
///
/// * `handler` — The SAPI handler (e.g. `php-fpm`, `proxy-php`)
/// * `version` — PHP version string (e.g. `8.1`)
/// * `default` — Whether this is the default version
#[derive(Debug, Clone, Deserialize)]
pub struct PhpVersion {
    /// The PHP handler (e.g. `"php-fpm"`).
    pub handler: String,
    /// The PHP version.
    pub version: String,
    /// Whether this is the default PHP version for the account.
    pub default: bool,
}

// ═════════════════════════════════════════════════════════════════════════════
// Cron Job models (UAPI)
// ═════════════════════════════════════════════════════════════════════════════

/// A cron job as returned by [`list_cron_jobs`](crate::CpanelClient::list_cron_jobs).
#[derive(Debug, Clone, Deserialize)]
pub struct CronJob {
    /// The unique job ID.
    pub id: String,
    /// Minute of the schedule (`0-59`, `*`, `/N`).
    pub minute: String,
    /// Hour of the schedule (`0-23`, `*`, `/N`).
    pub hour: String,
    /// Day of the month (`1-31`, `*`, `/N`).
    pub day: String,
    /// Month of the year (`1-12`, `*`, `/N`).
    pub month: String,
    /// Day of the week (`0-7`, `*`).
    pub weekday: String,
    /// The command to execute.
    pub command: String,
    /// Email address for job output notifications.
    pub email: Option<String>,
}

/// Parameters for adding a cron job via [`add_cron_job`](crate::CpanelClient::add_cron_job).
#[derive(Debug, Clone, Serialize, Default)]
pub struct AddCronJob {
    /// Minute of the schedule.
    pub minute: String,
    /// Hour of the schedule.
    pub hour: String,
    /// Day of the month.
    pub day: String,
    /// Month of the year.
    pub month: String,
    /// Day of the week.
    pub weekday: String,
    /// The command to execute.
    pub command: String,
    /// Email address for notification.
    pub email: Option<String>,
}

/// Parameters for editing a cron job via [`edit_cron_job`](crate::CpanelClient::edit_cron_job).
///
/// All fields except `job` are optional; only the provided fields will be updated.
#[derive(Debug, Clone, Default, Serialize)]
pub struct EditCronJob {
    /// The cron job ID to edit.
    pub job: String,
    /// New minute value.
    pub minute: Option<String>,
    /// New hour value.
    pub hour: Option<String>,
    /// New day-of-month value.
    pub day: Option<String>,
    /// New month value.
    pub month: Option<String>,
    /// New weekday value.
    pub weekday: Option<String>,
    /// New command.
    pub command: Option<String>,
    /// New email address.
    pub email: Option<String>,
}

// ═════════════════════════════════════════════════════════════════════════════
// Domain & Subdomain models (UAPI)
// ═════════════════════════════════════════════════════════════════════════════

/// An addon domain as returned by [`list_addon_domains`](crate::CpanelClient::list_addon_domains).
#[derive(Debug, Clone, Deserialize)]
pub struct AddonDomain {
    /// The addon domain name.
    pub domain: String,
    /// The document root path.
    pub document_root: String,
    /// Whether the domain is currently active.
    pub active: Option<bool>,
}

/// Parameters for adding an addon domain via [`create_addon_domain`](crate::CpanelClient::create_addon_domain).
#[derive(Debug, Clone, Serialize, Default)]
pub struct AddAddonDomain {
    /// The addon domain name (e.g. `"newsite.com"`).
    pub domain: String,
    /// The document root path (e.g. `"/home/user/public_html/newsite"`).
    pub document_root: Option<String>,
    /// Redirect type (`none`, `www`, `non-www`).
    pub redirect: Option<String>,
}

/// A subdomain as returned by [`list_subdomains`](crate::CpanelClient::list_subdomains).
#[derive(Debug, Clone, Deserialize)]
pub struct Subdomain {
    /// The subdomain name (e.g. `"blog"`).
    pub name: String,
    /// The parent domain.
    pub domain: String,
    /// The target (IP or URL).
    pub target: Option<String>,
    /// Whether the subdomain is active.
    pub active: Option<bool>,
}

/// Parameters for adding a subdomain via [`create_subdomain`](crate::CpanelClient::create_subdomain).
#[derive(Debug, Clone, Serialize, Default)]
pub struct AddSubdomain {
    /// The subdomain label (e.g. `"blog"`).
    pub subdomain: String,
    /// The parent domain (e.g. `"example.com"`).
    pub domain: String,
    /// Direction: `"forward"`, `"forward_only"`, or `"frame"`.
    pub direction: Option<String>,
    /// Target URL or IP for the subdomain.
    pub target: Option<String>,
}

// ═════════════════════════════════════════════════════════════════════════════
// FTP Account models (UAPI)
// ═════════════════════════════════════════════════════════════════════════════

/// An FTP account as returned by [`list_ftp_accounts`](crate::CpanelClient::list_ftp_accounts).
#[derive(Debug, Clone, Deserialize)]
pub struct FtpAccount {
    /// The FTP username.
    pub username: String,
    /// The FTP home directory path.
    pub path: Option<String>,
    /// Quota in MB (`None` = unlimited).
    pub quota: Option<i64>,
    /// Whether the account is active.
    pub active: Option<bool>,
}

/// Parameters for creating an FTP account via [`create_ftp_account`](crate::CpanelClient::create_ftp_account).
#[derive(Debug, Clone, Serialize, Default)]
pub struct CreateFtp {
    /// The FTP username.
    pub username: String,
    /// The FTP password.
    pub password: String,
    /// The FTP home directory path (defaults to `/`).
    pub path: Option<String>,
    /// Quota in MB (`None` = unlimited).
    pub quota: Option<i64>,
}

/// Parameters for modifying an FTP account via [`modify_ftp_account`](crate::CpanelClient::modify_ftp_account).
///
/// Only non-`None` fields will be updated.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ModifyFtp {
    /// The FTP username to modify.
    pub username: String,
    /// New password.
    pub password: Option<String>,
    /// New home directory path.
    pub path: Option<String>,
    /// New quota in MB (`None` = leave unchanged).
    pub quota: Option<i64>,
}

// ═════════════════════════════════════════════════════════════════════════════
// Security models (UAPI)
// ═════════════════════════════════════════════════════════════════════════════

/// A blocked IP entry as returned by [`list_ip_blocks`](crate::CpanelClient::list_ip_blocks).
#[derive(Debug, Clone, Deserialize)]
pub struct IpBlock {
    /// The blocked IP address.
    pub ip: String,
    /// The reason for blocking, if provided.
    pub reason: Option<String>,
}

/// A ModSecurity rule as returned by [`list_modsec_rules`](crate::CpanelClient::list_modsec_rules).
#[derive(Debug, Clone, Deserialize)]
pub struct ModSecRule {
    /// The rule ID (e.g. `"950001"`).
    pub id: String,
    /// The rule description.
    pub description: Option<String>,
    /// Whether the rule is currently enabled.
    pub enabled: bool,
    /// The rule category (e.g. `"MISC"`, `"RFI"`, `"LFI"`).
    pub category: Option<String>,
}
