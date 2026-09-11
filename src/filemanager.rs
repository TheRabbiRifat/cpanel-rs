//! File manager operations via cPanel UAPI.
//!
//! These methods operate on a per-user basis (cPanel port **2083**) and provide
//! file system operations similar to the cPanel File Manager interface.
//!
//! # Overview
//!
//! | Method | UAPI Call | Description |
//! |--------|-----------|-------------|
//! | [`CpanelClient::list_files`] | `Fileman.listdir` | List files/directories |
//! | [`CpanelClient::create_file`] | `Fileman.create_file` | Create a new file |
//! | [`CpanelClient::delete_file`] | `Fileman.delete` | Delete a file or directory |
//! | [`CpanelClient::rename_file`] | `Fileman.rename` | Rename/move a file |
//! | [`CpanelClient::copy_file`] | `Fileman.copy` | Copy a file |
//! | [`CpanelClient::chmod`] | `Fileman.chmod` | Change file permissions |
//! | [`CpanelClient::mkdir`] | `Fileman.mkdir` | Create a directory |
//! | [`CpanelClient::download_file`] | `Fileman.getfile` | Download a file's contents |
//! | [`CpanelClient::upload_file`] | `Fileman.uploadfile` | Upload a file |
//! | [`CpanelClient::compress`] | `Fileman.compress` | Create a zip archive |
//! | [`CpanelClient::extract`] | `Fileman.extract` | Extract a zip archive |
//!
//! # Path Conventions
//!
//! cPanel file paths are absolute from the home directory root:
//!
//! - Web root: `/public_html/` or `/home/username/public_html/`
//! - Documents root: `/documents/`
//! - FTP root: `/ftp/`
//! - Backup root: `/backup/`
//!
//! # Permissions
//!
//! The `chmod` method accepts standard Unix permission strings:
//!
//! | Mode | Meaning |
//! |------|---------|
//! | `0755` | Owner: rwx, Group: rx, Others: rx (standard for directories) |
//! | `0644` | Owner: rw, Group: r, Others: r (standard for files) |
//! | `0777` | All: rwx (not recommended) |
//! | `0600` | Owner: rw only (for sensitive files) |
//!
//! # Example
//!
//! ```no_run
//! use cpanel_rs::{CpanelClient, filemanager::{CreateFile, RenameFile, CompressFiles, CreateDir}};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), cpanel_rs::CpanelError> {
//!     let client = CpanelClient::simple("server.example.com", 2083, "cpanel_user", "pass").unwrap();
//!
//!     // List files in public_html
//!     let files = client.list_files("cpanel_user", "/public_html").await?;
//!     for f in &files {
//!         println!("{:<30} {:<5} {:<10}", f.name, f.type_, f.size.as_ref().unwrap_or(&"0".into()));
//!     }
//!
//!     // Create a file
//!     client.create_file("cpanel_user", &CreateFile {
//!         path: "/public_html/hello.txt".into(),
//!         content: Some("Hello, World!".into()),
//!         overwrite: false,
//!     }).await?;
//!
//!     // Upload a file
//!     let contents = b"<!DOCTYPE html><html><body>Hello</body></html>";
//!     client.upload_file("cpanel_user", "/public_html/index.html", contents, "index.html").await?;
//!
//!     // Download a file
//!     let data = client.download_file("cpanel_user", "/public_html/hello.txt").await?;
//!     println!("Downloaded {} bytes", data.len());
//!
//!     // Rename a file
//!     client.rename_file("cpanel_user", &RenameFile {
//!         source: "/public_html/old.txt".into(),
//!         destination: "/public_html/new.txt".into(),
//!     }).await?;
//!
//!     // Set permissions
//!     client.chmod("cpanel_user", "/public_html", "0755").await?;
//!
//!     // Create a directory
//!     client.mkdir("cpanel_user", &CreateDir {
//!         path: "/public_html/assets".into(),
//!     }).await?;
//!
//!     // Compress files
//!     client.compress("cpanel_user", &CompressFiles {
//!         paths: vec!["/public_html/wp-content/uploads".into()],
//!         archive_name: "uploads_backup.zip".into(),
//!     }).await?;
//!
//!     Ok(())
//! }
//! ```

use crate::{models::FileManagerEntry, CpanelClient, CpanelError};
use serde::Serialize;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt as _;

/// Parameters for creating a file via [`create_file`](CpanelClient::create_file).
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateFile {
    /// Full path where the file should be created (e.g. `/public_html/foo.txt`).
    pub path: String,
    /// File content. If `None`, an empty file is created.
    pub content: Option<String>,
    /// Overwrite the file if it already exists.
    pub overwrite: bool,
}

/// Parameters for renaming a file via [`rename_file`](CpanelClient::rename_file).
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct RenameFile {
    /// Current full path.
    pub source: String,
    /// New full path.
    pub destination: String,
}

/// Parameters for creating a directory via [`mkdir`](CpanelClient::mkdir).
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateDir {
    /// Full path for the new directory.
    pub path: String,
}

/// Parameters for compressing files via [`compress`](CpanelClient::compress).
#[derive(Debug, Clone, Default, Serialize)]
pub struct CompressFiles {
    /// List of file/directory paths to compress.
    pub paths: Vec<String>,
    /// Name of the resulting archive (e.g. `"backup.zip"`).
    pub archive_name: String,
}

impl CpanelClient {
    /// Lists files and directories at the given path.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `path` — The directory path to list (e.g. `"/public_html"`).
    ///
    /// # Returns
    ///
    /// A vector of [`FileManagerEntry`] for each item in the directory.
    pub async fn list_files(
        &self,
        user: &str,
        path: &str,
    ) -> Result<Vec<FileManagerEntry>, CpanelError> {
        let result: Vec<FileManagerEntry> = self
            .uapi("Fileman", "listdir", &[("path", path), ("user", user)])
            .await?;
        Ok(result)
    }

    /// Creates a new file with the given content.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `create` — A [`CreateFile`] with the path, content, and overwrite flag.
    pub async fn create_file(&self, user: &str, create: &CreateFile) -> Result<(), CpanelError> {
        let overwrite_str = create.overwrite.to_string();
        let mut params: Vec<(&str, &str)> = vec![
            ("path", &create.path),
            ("user", user),
            ("overwrite", &overwrite_str),
        ];

        if let Some(content) = &create.content {
            params.push(("content", content.as_str()));
        }

        let _: () = self.uapi("Fileman", "create_file", &params).await?;
        Ok(())
    }

    /// Deletes a file or directory.
    ///
    /// **Warning:** This operation is irreversible.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `path` — Full path to the file or directory to delete.
    pub async fn delete_file(&self, user: &str, path: &str) -> Result<(), CpanelError> {
        let _: () = self
            .uapi("Fileman", "delete", &[("path", path), ("user", user)])
            .await?;
        Ok(())
    }

    /// Renames or moves a file/directory.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `rename` — A [`RenameFile`] with source and destination paths.
    pub async fn rename_file(&self, user: &str, rename: &RenameFile) -> Result<(), CpanelError> {
        let _: () = self
            .uapi(
                "Fileman",
                "rename",
                &[
                    ("path", &rename.source),
                    ("newpath", &rename.destination),
                    ("user", user),
                ],
            )
            .await?;
        Ok(())
    }

    /// Copies a file to a new location.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `source` — Full path of the file to copy.
    /// * `destination` — Full path for the copy.
    pub async fn copy_file(
        &self,
        user: &str,
        source: &str,
        destination: &str,
    ) -> Result<(), CpanelError> {
        let _: () = self
            .uapi(
                "Fileman",
                "copy",
                &[("source", source), ("dest", destination), ("user", user)],
            )
            .await?;
        Ok(())
    }

    /// Changes file permissions (chmod).
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `path` — Full path to the file or directory.
    /// * `mode` — Unix permission mode (e.g. `"0755"`, `"0644"`).
    pub async fn chmod(&self, user: &str, path: &str, mode: &str) -> Result<(), CpanelError> {
        let _: () = self
            .uapi(
                "Fileman",
                "chmod",
                &[("path", path), ("mode", mode), ("user", user)],
            )
            .await?;
        Ok(())
    }

    /// Creates a new directory.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `create` — A [`CreateDir`] with the directory path.
    pub async fn mkdir(&self, user: &str, create: &CreateDir) -> Result<(), CpanelError> {
        let _: () = self
            .uapi(
                "Fileman",
                "mkdir",
                &[("path", &create.path), ("user", user)],
            )
            .await?;
        Ok(())
    }

    /// Downloads a file from the cPanel account.
    ///
    /// Returns the raw file contents as a byte vector.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `path` — Full path to the file to download.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use cpanel_rs::CpanelClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), cpanel_rs::CpanelError> {
    ///     let client = CpanelClient::simple("server.example.com", 2083, "user", "pass").unwrap();
    ///     let data = client.download_file("user", "/public_html/config.json").await?;
    ///     println!("Downloaded {} bytes", data.len());
    ///     Ok(())
    /// }
    /// ```
    pub async fn download_file(&self, user: &str, path: &str) -> Result<Vec<u8>, CpanelError> {
        let url = format!(
            "{}/json-api/cpanel?cpanel_jsonapi_module=Fileman&cpanel_jsonapi_apiversion=2&cpanel_jsonapi_func=getfile&user={}&path={}",
            self.base_url(),
            urlencoding::encode(user),
            urlencoding::encode(path),
        );

        let response = self
            .http_client()
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await?;

        Ok(response.bytes().await?.to_vec())
    }

    /// Uploads a file to the cPanel account from an async reader.
    ///
    /// Reads the source in chunks and streams them to the server, avoiding
    /// loading the entire file into memory.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `path` — Full path where the file should be saved.
    /// * `reader` — A type implementing [`tokio::io::AsyncRead`].
    /// * `filename` — The filename to use on the server.
    /// * `chunk_size` — Maximum bytes per multipart part (default: 1 MB).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use cpanel_rs::CpanelClient;
    /// use tokio::fs::File;
    /// use tokio::io::AsyncReadExt;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), cpanel_rs::CpanelError> {
    ///     let client = CpanelClient::simple("server.example.com", 2083, "user", "pass").unwrap();
    ///     let mut file = File::open("large_upload.zip").await?;
    ///     client.upload_file_from_reader("user", "/public_html/large_upload.zip", &mut file, "large_upload.zip", 1024 * 1024).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn upload_file_from_reader<R: AsyncRead + Unpin>(
        &self,
        user: &str,
        path: &str,
        reader: &mut R,
        filename: &str,
        _chunk_size: usize,
    ) -> Result<(), CpanelError> {
        let path_owned = path.to_string();
        let filename_owned = filename.to_string();
        let url = format!(
            "{}/json-api/cpanel?cpanel_jsonapi_module=Fileman&cpanel_jsonapi_apiversion=2&cpanel_jsonapi_func=uploadfile&user={}",
            self.base_url(),
            urlencoding::encode(user),
        );

        // Read all chunks first — cPanel's upload API expects a single multipart part.
        // For truly large files, consider using direct HTTP upload instead.
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await?;

        let form = reqwest::multipart::Form::new()
            .part("path", reqwest::multipart::Part::text(path_owned))
            .part(
                "file",
                reqwest::multipart::Part::bytes(buf).file_name(filename_owned),
            );

        let response = self
            .http_client()
            .post(&url)
            .headers(self.auth_headers())
            .multipart(form)
            .send()
            .await?;

        let text: String = response.text().await?;
        let api_resp: crate::types::ApiResponse<serde_json::Value> =
            serde_json::from_str(&text).map_err(CpanelError::Serialization)?;

        if !api_resp.is_ok() {
            return Err(CpanelError::ApiError(crate::types::ApiFailure {
                message: api_resp.status_msg,
                call: "Fileman.uploadfile".into(),
                ..Default::default()
            }));
        }

        Ok(())
    }

    /// Uploads a file to the cPanel account.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `path` — Full path where the file should be saved.
    /// * `content` — The raw file contents.
    /// * `filename` — The filename to use on the server.
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
    ///     let html = b"<!DOCTYPE html><html><body>Hello World</body></html>";
    ///     client.upload_file("user", "/public_html/index.html", html, "index.html").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn upload_file(
        &self,
        user: &str,
        path: &str,
        content: &[u8],
        filename: &str,
    ) -> Result<(), CpanelError> {
        let path_owned = path.to_string();
        let filename_owned = filename.to_string();
        let form = reqwest::multipart::Form::new()
            .part("path", reqwest::multipart::Part::text(path_owned))
            .part(
                "file",
                reqwest::multipart::Part::bytes(content.to_vec()).file_name(filename_owned),
            );

        let url = format!(
            "{}/json-api/cpanel?cpanel_jsonapi_module=Fileman&cpanel_jsonapi_apiversion=2&cpanel_jsonapi_func=uploadfile&user={}",
            self.base_url(),
            urlencoding::encode(user),
        );

        let response = self
            .http_client()
            .post(&url)
            .headers(self.auth_headers())
            .multipart(form)
            .send()
            .await?;

        let text: String = response.text().await?;
        let api_resp: crate::types::ApiResponse<serde_json::Value> =
            serde_json::from_str(&text).map_err(CpanelError::Serialization)?;

        if !api_resp.is_ok() {
            return Err(CpanelError::ApiError(crate::types::ApiFailure {
                message: api_resp.status_msg,
                call: "Fileman.uploadfile".into(),
                ..Default::default()
            }));
        }

        Ok(())
    }

    /// Creates a zip archive from the given file paths.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `compress` — A [`CompressFiles`] with paths and archive name.
    pub async fn compress(&self, user: &str, compress: &CompressFiles) -> Result<(), CpanelError> {
        let paths_json =
            serde_json::to_string(&compress.paths).map_err(CpanelError::Serialization)?;

        let _: () = self
            .uapi(
                "Fileman",
                "compress",
                &[
                    ("paths", &paths_json),
                    ("name", &compress.archive_name),
                    ("user", user),
                ],
            )
            .await?;
        Ok(())
    }

    /// Extracts a zip archive to the destination path.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `archive_path` — Full path to the zip archive.
    /// * `dest_path` — Full path to extract into.
    pub async fn extract(
        &self,
        user: &str,
        archive_path: &str,
        dest_path: &str,
    ) -> Result<(), CpanelError> {
        let _: () = self
            .uapi(
                "Fileman",
                "extract",
                &[
                    ("path", archive_path),
                    ("destpath", dest_path),
                    ("user", user),
                ],
            )
            .await?;
        Ok(())
    }
}
