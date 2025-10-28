use rusqlite::{Connection, Error as SqliteError, Result as SqliteResult, Row};
use std::fmt;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};

use crate::error::AppError;

#[derive(Debug)]
pub enum DatabaseError {
    SqliteError(String),
    FsError(String),
    InvalidQuery(String),
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseError::SqliteError(msg) => write!(f, "SQLite error: {}", msg),
            DatabaseError::FsError(msg) => write!(f, "File system error: {}", msg),
            DatabaseError::InvalidQuery(msg) => write!(f, "Invalid query: {}", msg),
        }
    }
}

impl std::error::Error for DatabaseError {}

impl From<SqliteError> for DatabaseError {
    fn from(err: SqliteError) -> Self {
        DatabaseError::SqliteError(err.to_string())
    }
}

impl From<serde_json::Error> for DatabaseError {
    fn from(err: serde_json::Error) -> Self {
        DatabaseError::FsError(err.to_string())
    }
}

impl From<std::io::Error> for DatabaseError {
    fn from(err: std::io::Error) -> Self {
        DatabaseError::FsError(err.to_string())
    }
}

#[derive(Clone)]
pub struct DatabaseService {
    pub conn: Arc<Mutex<Connection>>,
}

impl DatabaseService {
    pub fn new(app: Option<&AppHandle>) -> Result<Self, AppError> {
        let conn = match app {
            Some(handle) => {
                let app_data_dir = handle.path().app_data_dir().map_err(|e| AppError::Io(e.to_string()))?;

                std::fs::create_dir_all(&app_data_dir).map_err(|e| AppError::Io(e.to_string()))?;

                let db_path = app_data_dir.join("database.db");
                Connection::open(&db_path).map_err(|e| AppError::Io(e.to_string()))?
            }
            None => Connection::open(":memory:").map_err(|e| AppError::Io(e.to_string()))?,
        };

        conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA cache_size = -64000;
            PRAGMA foreign_keys = ON;
            PRAGMA temp_store = MEMORY;
            PRAGMA mmap_size = 30000000000;
        ",
        )
        .map_err(|e| AppError::Io(e.to_string()))?;

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        Ok(db)
    }

    pub fn create_table(&self, table: &str, columns: &[&str], constraints: &[&str]) -> SqliteResult<()> {
        self.validate_table_name(table)?;

        let conn = self.conn.lock().map_err(|_| SqliteError::InvalidQuery)?;

        let mut all_columns = vec!["id INTEGER PRIMARY KEY AUTOINCREMENT"];

        all_columns.extend_from_slice(columns);
        all_columns.push("created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP");
        all_columns.push("updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP");

        let mut definitions = all_columns.iter().map(|s| s.to_string()).collect::<Vec<_>>();

        definitions.extend(constraints.iter().map(|s| s.to_string()));

        let sql = format!(
            "CREATE TABLE IF NOT EXISTS {} (\n                {}\n            )",
            table,
            definitions.join(",\n                ")
        );

        conn.execute(&sql, [])?;

        Ok(())
    }

    pub fn execute<P>(&self, query: &str, params: P) -> SqliteResult<usize>
    where
        P: rusqlite::Params,
    {
        let conn = self.conn.lock().map_err(|_| SqliteError::InvalidQuery)?;

        let result = conn.execute(query, params)?;

        Ok(result)
    }

    pub fn execute_batch<P>(&self, query: &str, params_list: &[P]) -> SqliteResult<()>
    where
        P: rusqlite::Params + Clone,
    {
        let mut conn = self.conn.lock().map_err(|_| SqliteError::InvalidQuery)?;

        let tx = conn.transaction()?;

        {
            let mut stmt = tx.prepare(query)?;
            for params in params_list {
                stmt.execute(params.clone())?;
            }
        }

        tx.commit()?;

        Ok(())
    }

    pub fn execute_transaction(&self, queries: &[(&str, &[&dyn rusqlite::ToSql])]) -> SqliteResult<()> {
        let mut conn = self.conn.lock().map_err(|_| SqliteError::InvalidQuery)?;
        let tx = conn.transaction()?;

        for (query, params) in queries {
            tx.execute(query, *params)?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn query<P, F, T>(&self, query: &str, params: P, mut mapper: F) -> Result<Vec<T>, DatabaseError>
    where
        P: rusqlite::Params,
        F: FnMut(&Row) -> Result<T, DatabaseError>,
    {
        let conn = self
            .conn
            .lock()
            .map_err(|_| DatabaseError::SqliteError("Failed to acquire mutex lock".to_string()))?;

        let mut stmt = conn
            .prepare(query)
            .map_err(|e| DatabaseError::SqliteError(e.to_string()))?;

        let rows = stmt.query_map(params, |row| mapper(row).map_err(|_| SqliteError::InvalidQuery))?;

        let mut results = Vec::new();
        for row in rows {
            let value = row.map_err(|e| DatabaseError::SqliteError(e.to_string()))?;
            results.push(value);
        }

        Ok(results)
    }

    pub fn table_exists(&self, table: &str) -> SqliteResult<bool> {
        let conn = self.conn.lock().map_err(|_| SqliteError::InvalidQuery)?;

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?",
            [table],
            |row| row.get(0),
        )?;

        Ok(count > 0)
    }

    pub fn validate_table_name(&self, table: &str) -> SqliteResult<()> {
        if table.is_empty() || table.len() > 64 {
            return Err(SqliteError::InvalidParameterName(
                "Table name must be between 1-64 characters".to_string(),
            ));
        }

        if !table.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(SqliteError::InvalidParameterName(format!(
                "Table name can only contain alphanumeric characters and underscores for {}",
                table
            )));
        }

        if table.chars().next().unwrap().is_numeric() {
            return Err(SqliteError::InvalidParameterName(
                "Table name cannot start with a number".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod creation {
        use super::*;

        #[test]
        fn test_new_database_creates_in_memory_database() {
            let db = DatabaseService::new(None);
            assert!(db.is_ok(), "Failed to create in-memory database");
        }

        #[test]
        fn test_create_table() {
            let db = DatabaseService::new(None).expect("Failed to create database");
            db.create_table("test_table", &["name TEXT NOT NULL", "path TEXT NOT NULL"], &[])
                .expect("Failed to create test table");

            let conn = db.conn.lock().unwrap();

            let mut test_table_stmt = conn
                .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='test_table'")
                .expect("Failed to prepare query");

            let test_table_exists: bool = test_table_stmt.exists([]).expect("Failed to check if table exists");

            assert!(test_table_exists, "test table was not created");
        }

        #[test]
        fn test_create_table_with_constraints() {
            let db = DatabaseService::new(None).expect("Failed to create database");
            db.create_table(
                "test_table",
                &["name TEXT NOT NULL", "path TEXT NOT NULL"],
                &["UNIQUE (path)"],
            )
            .expect("Failed to create test table");

            let conn = db.conn.lock().unwrap();

            conn.execute("INSERT INTO test_table (name, path) VALUES (?, ?)", ["test", "test"])
                .expect("Failed to insert test");
            assert!(
                conn.execute("INSERT INTO test_table (name, path) VALUES (?, ?)", ["test", "test"])
                    .is_err(),
                "Second insert should have failed"
            );
        }
    }

    mod execution {
        use super::*;

        #[test]
        fn test_execute() {
            let db = DatabaseService::new(None).expect("Failed to create database");

            {
                let conn = db.conn.lock().unwrap();
                conn.execute(
                    "CREATE TABLE IF NOT EXISTS test_table (name TEXT NOT NULL, description TEXT NOT NULL)",
                    [],
                )
                .expect("Failed to create test table");
            }

            let result = db
                .execute(
                    "INSERT INTO test_table (name, description) VALUES (?, ?)",
                    ["test", "test"],
                )
                .expect("Failed to execute test");
            assert!(result > 0, "Failed to execute test");
        }

        #[test]
        fn test_execute_batch() {
            let db = DatabaseService::new(None).expect("Failed to create database");

            {
                let conn = db.conn.lock().unwrap();
                conn.execute(
                    "CREATE TABLE IF NOT EXISTS test_table (name TEXT NOT NULL, description TEXT NOT NULL)",
                    [],
                )
                .expect("Failed to create test table");
            }

            db.execute_batch(
                "INSERT INTO test_table (name, description) VALUES (?, ?)",
                &[["test", "test"], ["test2", "test2"]],
            )
            .expect("Failed to execute test");

            let conn = db.conn.lock().unwrap();

            let mut test_item_1_stmt = conn
                .prepare("SELECT name FROM test_table WHERE name = 'test'")
                .expect("Failed to prepare query");

            let mut test_item_2_stmt = conn
                .prepare("SELECT name FROM test_table WHERE name = 'test2'")
                .expect("Failed to prepare query");

            let test_item_1_exists: bool = test_item_1_stmt
                .exists([])
                .expect("Failed to check if test item 1 exists");
            let test_item_2_exists: bool = test_item_2_stmt
                .exists([])
                .expect("Failed to check if test item 2 exists");

            assert!(test_item_1_exists, "test item 1 was not created");
            assert!(test_item_2_exists, "test item 2 was not created");
        }

        #[test]
        fn test_execute_transaction() {
            let db = DatabaseService::new(None).expect("Failed to create database");

            {
                let conn = db.conn.lock().unwrap();
                conn.execute(
                    "CREATE TABLE IF NOT EXISTS test_table (name TEXT NOT NULL, description TEXT NOT NULL)",
                    [],
                )
                .expect("Failed to create test table");
            }

            db.execute_transaction(&[
                (
                    "INSERT INTO test_table (name, description) VALUES (?, ?)",
                    &[&"test", &"test"],
                ),
                (
                    "INSERT INTO test_table (name, description) VALUES (?, ?)",
                    &[&"test2", &"test2"],
                ),
            ])
            .expect("Failed to execute test");

            let conn = db.conn.lock().unwrap();

            let mut test_item_1_stmt = conn
                .prepare("SELECT name FROM test_table WHERE name = 'test'")
                .expect("Failed to prepare query");

            let mut test_item_2_stmt = conn
                .prepare("SELECT name FROM test_table WHERE name = 'test2'")
                .expect("Failed to prepare query");

            let test_item_1_exists: bool = test_item_1_stmt
                .exists([])
                .expect("Failed to check if test item 1 exists");

            let test_item_2_exists: bool = test_item_2_stmt
                .exists([])
                .expect("Failed to check if test item 2 exists");

            assert!(test_item_1_exists, "test item 1 was not created");
            assert!(test_item_2_exists, "test item 2 was not created");
        }
    }

    mod queries {
        use super::*;

        #[test]
        fn test_query() {
            let db = DatabaseService::new(None).expect("Failed to create database");

            {
                let conn = db.conn.lock().unwrap();
                conn.execute(
                    "CREATE TABLE IF NOT EXISTS test_table (name TEXT NOT NULL, description TEXT NOT NULL)",
                    [],
                )
                .expect("Failed to create test table");

                conn.execute(
                    "INSERT INTO test_table (name, description) VALUES (?, ?)",
                    ["test1", "desc1"],
                )
                .expect("Failed to insert test");
            }

            let result = db
                .query("SELECT * FROM test_table", [], |row| Ok((row.get::<_, String>(0)?,)))
                .expect("Failed to query test");

            assert!(result.len() > 0, "Failed to execute test");
        }

        #[test]
        fn test_query_multiple_rows() {
            let db = DatabaseService::new(None).expect("Failed to create database");

            {
                let conn = db.conn.lock().unwrap();
                conn.execute(
                    "CREATE TABLE IF NOT EXISTS test_table (name TEXT NOT NULL, description TEXT NOT NULL)",
                    [],
                )
                .expect("Failed to create test table");

                conn.execute(
                    "INSERT INTO test_table (name, description) VALUES (?, ?)",
                    ["test1", "desc1"],
                )
                .expect("Failed to insert test");
                conn.execute(
                    "INSERT INTO test_table (name, description) VALUES (?, ?)",
                    ["test2", "desc2"],
                )
                .expect("Failed to insert test");
                conn.execute(
                    "INSERT INTO test_table (name, description) VALUES (?, ?)",
                    ["test3", "desc3"],
                )
                .expect("Failed to insert test");
            }

            let result = db
                .query("SELECT * FROM test_table", [], |row| Ok((row.get::<_, String>(0)?,)))
                .expect("Failed to query test");

            assert!(result.len() == 3, "Failed to execute test");
        }
    }

    mod utilities {
        use super::super::*;

        #[test]
        fn test_table_exists() {
            let db = DatabaseService::new(None).expect("Failed to create database");

            {
                let conn = db.conn.lock().unwrap();
                conn.execute(
                    "CREATE TABLE IF NOT EXISTS test_table (name TEXT NOT NULL, description TEXT NOT NULL)",
                    [],
                )
                .expect("Failed to create test table");
            }

            let table_exists: bool = db.table_exists("test_table").expect("Failed to check if table exists");

            assert!(table_exists, "test table was not created");
        }

        #[test]
        fn test_validate_table_name() {
            let db = DatabaseService::new(None).expect("Failed to create database");

            let test_simple_name: Result<(), rusqlite::Error> = db.validate_table_name("test_table");
            assert!(test_simple_name.is_ok(), "Failed to validate table name");

            let test_start_by_number = db.validate_table_name("123test_table");
            assert!(test_start_by_number.is_err(), "Failed to validate table name");

            let test_start_by_number_and_special_characters = db.validate_table_name("test; dede");
            assert!(
                test_start_by_number_and_special_characters.is_err(),
                "Failed to validate table name"
            );

            let test_long_name =
                db.validate_table_name("A_long_naaaaaaaaaaaaaaammmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmme");
            assert!(test_long_name.is_err(), "Failed to validate table name");
        }
    }
}
