# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2024-01-20

### Added
- `cron` module — Scheduled cron jobs management (`list_cron_jobs`, `add_cron_job`, `edit_cron_job`, `delete_cron_job`)
- `domains` module — Addon domains, parked domains, and subdomains management
- `ftp` module — FTP accounts management (`list_ftp_accounts`, `create_ftp_account`, `change_ftp_password`, `set_ftp_quota`, `delete_ftp_account`)
- `security` module — ModSecurity rule management and IP blocker (blacklist/whitelist)
- `streams` module — Auto-paginating async streams over large listings (`stream_accounts`, `stream_files`) behind `--features stream`
- `RetryPolicy` — Configurable exponential backoff with jitter for transient network failures and HTTP 429 / 503 errors behind `--features retry`
- Modern cPanel API Token authentication (`Authorization: cpanel <user>:<token>`) via `CpanelBuilder::token`
- Streaming large file uploads from `AsyncRead` via `upload_file_from_reader`
- Optional `tracing` instrumentation for structured request lifecycle logs
- `full` feature flag combining `dotenv`, `retry`, and `stream`

### Changed
- Upgraded `reqwest` to 0.12 with `http` 1.0 support
- Upgraded `base64` to 0.22

## [0.2.0] - 2024-01-15

### Added
- `CpanelBuilder` — builder pattern for full control over connections (timeout, TLS verification toggle, custom headers)
- `CpanelClient::new(host, port, user, password, token, use_ssl)` — explicit constructor without environment variable requirement
- `CpanelClient::simple(host, port, user, pass)` — convenience constructor for quick setup
- `AuthMethod` enum — support for Basic Password auth and WHM raw access key token auth
- `dotenv` support behind optional `--features dotenv` flag
- DNS zone import/export (`export_zone`, `import_zone`)
- File upload/download (`upload_file`, `download_file`)
- File manager operations: `mkdir`, `compress`, `extract`
- Email autoresponders (`list_autoresponders`, `create_autoresponder`, `delete_autoresponder`)
- Email mailing lists (`list_mailing_lists`, `create_mailing_list`, `delete_mailing_list`)
- Database privilege management (`grant_privileges`, `revoke_privileges`, `list_db_user_privileges`)
- Backup schedule management (`get_backup_schedule`, `set_backup_schedule`)
- SSL Let's Encrypt certificate issuance (`issue_letsencrypt`)
- Pagination support via `QueryParams`
- Comprehensive API documentation and doctests

### Changed
- Made `dotenv` an optional dependency
- Updated `CpanelClient::new` to accept explicit credentials
- Added default port auto-detection for HTTPS (ports 2083/2087 default to SSL)

### Fixed
- Improved client builder error handling during HTTP client initialization
- Resolved intra-doc link references and doctest compilation warnings

## [0.1.0] - 2024-01-01

### Added
- Initial release
- Core cPanel UAPI and WHM JSON API client
- Account, DNS, Email, Database, File Manager, SSL, Backup, and Resources modules
- Basic Password and WHM raw key authentication
- Environment variable credential loading

[Unreleased]: https://github.com/TheRabbiRifat/cpanel-rs/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/TheRabbiRifat/cpanel-rs/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/TheRabbiRifat/cpanel-rs/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/TheRabbiRifat/cpanel-rs/releases/tag/v0.1.0


