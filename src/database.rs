//! MySQL database management via cPanel UAPI.
//!
//! These methods operate on a per-user basis (cPanel port **2083**) and manage
//! MySQL databases, users, and privileges.
//!
//! # Overview
//!
//! | Method | UAPI Call | Description |
//! |--------|-----------|-------------|
//! | [`CpanelClient::list_databases`] | `Mysql.list_dbs` | List all databases |
//! | [`CpanelClient::create_database`] | `Mysql.create_db` | Create a new database |
//! | [`CpanelClient::delete_database`] | `Mysql.drop_db` | Delete a database |
//! | [`CpanelClient::create_db_user`] | `Mysql.add_user` | Create a database user |
//! | [`CpanelClient::grant_privileges`] | `Mysql.set_user_privileges` | Grant privileges |
//! | [`CpanelClient::revoke_privileges`] | `Mysql.set_user_privileges` | Revoke privileges |
//! | [`CpanelClient::list_db_user_privileges`] | `Mysql.list_user_privileges` | List user privileges |
//! | [`CpanelClient::drop_db_user`] | `Mysql.drop_user` | Delete a database user |
//!
//! # Naming Conventions
//!
//! cPanel automatically prefixes database and user names with the cPanel username.
//! For example, creating a database named `mydb` for user `john` will result in
//! the database name `john_mydb`. You only need to provide the unprefixed name.
//!
//! # Privileges
//!
//! Valid privilege values include:
//!
//! | Privilege | Scope |
//! |-----------|-------|
//! | `ALL` | All privileges |
//! | `SELECT` | Read data |
//! | `INSERT` | Insert data |
//! | `UPDATE` | Update data |
//! | `DELETE` | Delete data |
//! | `CREATE` | Create tables |
//! | `DROP` | Drop tables |
//! | `INDEX` | Create/drop indexes |
//! | `ALTER` | Modify table structure |
//! | `REFERENCES` | Foreign key references |
//! | `CREATE TEMPORARY TABLES` | Create temp tables |
//! | `LOCK TABLES` | Lock tables |
//! | `EXECUTE` | Execute stored routines |
//! | `CREATE VIEW` | Create views |
//! | `SHOW VIEW` | Show views |
//! | `CREATE ROUTINE` | Create stored routines |
//! | `ALTER ROUTINE` | Alter stored routines |
//! | `EVENT` | Create events |
//! | `TRIGGER` | Create triggers |
//!
//! Use `"*"` for `table` to grant database-level privileges.
//!
//! # Example
//!
//! ```no_run
//! use cpanel_rs::{CpanelClient, models::CreateDatabase};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), cpanel_rs::CpanelError> {
//!     let client = CpanelClient::simple("server.example.com", 2083, "cpanel_user", "pass").unwrap();
//!
//!     // Create a database
//!     client.create_database("cpanel_user", &CreateDatabase {
//!         name: "myapp".into(),
//!         charset: Some("utf8mb4".into()),
//!     }).await?;
//!
//!     // Create a user and grant privileges
//!     client.create_db_user("cpanel_user", "appuser", "securepass").await?;
//!     client.grant_privileges("cpanel_user", "appuser", "myapp", "*", "ALL").await?;
//!
//!     // List all databases
//!     let dbs = client.list_databases("cpanel_user").await?;
//!     for db in &dbs {
//!         println!("{} ({} MB)", db.name, db.size.unwrap_or(0.0));
//!     }
//!
//!     Ok(())
//! }
//! ```

use crate::{
    models::{CreateDatabase, Database, DbPrivilege},
    CpanelClient, CpanelError,
};

impl CpanelClient {
    /// Lists all MySQL databases for a cPanel account.
    ///
    /// Database names are returned with the cPanel username prefix
    /// (e.g. `john_myapp`).
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    pub async fn list_databases(&self, user: &str) -> Result<Vec<Database>, CpanelError> {
        let result: Vec<Database> = self.uapi("Mysql", "list_dbs", &[("user", user)]).await?;
        Ok(result)
    }

    /// Creates a new MySQL database.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `create` — A [`CreateDatabase`] with the database name and optional charset.
    pub async fn create_database(
        &self,
        user: &str,
        create: &CreateDatabase,
    ) -> Result<(), CpanelError> {
        let mut params: Vec<(&str, &str)> = vec![("user", user), ("name", &create.name)];

        if let Some(charset) = &create.charset {
            params.push(("charset", charset.as_str()));
        }

        let _: () = self.uapi("Mysql", "create_db", &params).await?;
        Ok(())
    }

    /// Deletes a MySQL database and all its data.
    ///
    /// **Warning:** This operation is irreversible.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `name` — The database name (with or without the cPanel username prefix).
    pub async fn delete_database(&self, user: &str, name: &str) -> Result<(), CpanelError> {
        let _: () = self
            .uapi("Mysql", "drop_db", &[("user", user), ("name", name)])
            .await?;
        Ok(())
    }

    /// Creates a new MySQL user.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `db_user` — The desired database username (the cPanel prefix is added automatically).
    /// * `password` — The password for the database user.
    pub async fn create_db_user(
        &self,
        user: &str,
        db_user: &str,
        password: &str,
    ) -> Result<(), CpanelError> {
        let _: () = self
            .uapi(
                "Mysql",
                "add_user",
                &[("user", user), ("name", db_user), ("password", password)],
            )
            .await?;
        Ok(())
    }

    /// Grants privileges to a database user.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `db_user` — The database username.
    /// * `database` — The database name (without prefix).
    /// * `table` — The table name, or `"*"` for the entire database.
    /// * `privilege` — The privilege to grant (e.g. `"ALL"`, `"SELECT"`).
    pub async fn grant_privileges(
        &self,
        user: &str,
        db_user: &str,
        database: &str,
        table: &str,
        privilege: &str,
    ) -> Result<(), CpanelError> {
        let _: () = self
            .uapi(
                "Mysql",
                "set_user_privileges",
                &[
                    ("user", user),
                    ("dbuser", db_user),
                    ("dbname", database),
                    ("table", table),
                    ("privilege", privilege),
                ],
            )
            .await?;
        Ok(())
    }

    /// Revokes privileges from a database user.
    ///
    /// This is the inverse of [`grant_privileges`](CpanelClient::grant_privileges).
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `db_user` — The database username.
    /// * `database` — The database name.
    /// * `table` — The table name, or `"*"` for the entire database.
    /// * `privilege` — The privilege to revoke.
    pub async fn revoke_privileges(
        &self,
        user: &str,
        db_user: &str,
        database: &str,
        table: &str,
        privilege: &str,
    ) -> Result<(), CpanelError> {
        let _: () = self
            .uapi(
                "Mysql",
                "set_user_privileges",
                &[
                    ("user", user),
                    ("dbuser", db_user),
                    ("dbname", database),
                    ("table", table),
                    ("privilege", privilege),
                    ("revoke", "1"),
                ],
            )
            .await?;
        Ok(())
    }

    /// Lists all privileges granted to a database user.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `db_user` — The database username.
    pub async fn list_db_user_privileges(
        &self,
        user: &str,
        db_user: &str,
    ) -> Result<Vec<DbPrivilege>, CpanelError> {
        let result: Vec<DbPrivilege> = self
            .uapi(
                "Mysql",
                "list_user_privileges",
                &[("user", user), ("dbuser", db_user)],
            )
            .await?;
        Ok(result)
    }

    /// Deletes a MySQL user.
    ///
    /// **Warning:** This removes the user but does NOT remove any databases
    /// that the user may own. Use [`delete_database`](CpanelClient::delete_database)
    /// separately to remove those.
    ///
    /// # Arguments
    ///
    /// * `user` — The cPanel username.
    /// * `db_user` — The database username to delete.
    pub async fn drop_db_user(&self, user: &str, db_user: &str) -> Result<(), CpanelError> {
        let _: () = self
            .uapi("Mysql", "drop_user", &[("user", user), ("name", db_user)])
            .await?;
        Ok(())
    }
}
