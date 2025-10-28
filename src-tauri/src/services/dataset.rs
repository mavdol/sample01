use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use crate::services::{DatabaseError, DatabaseService};
use rusqlite::Result as SqliteResult;

#[derive(Debug)]
pub enum DatasetError {
    NotFound(String),
    DatabaseError(String),
    InvalidInput(String),
    FsError(String),
}

impl fmt::Display for DatasetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatasetError::NotFound(msg) => write!(f, "Dataset not found: {}", msg),
            DatasetError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            DatasetError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            DatasetError::FsError(msg) => write!(f, "File system error: {}", msg),
        }
    }
}

impl std::error::Error for DatasetError {}

impl From<rusqlite::Error> for DatasetError {
    fn from(err: rusqlite::Error) -> Self {
        DatasetError::DatabaseError(err.to_string())
    }
}

impl From<serde_json::Error> for DatasetError {
    fn from(err: serde_json::Error) -> Self {
        DatasetError::FsError(err.to_string())
    }
}

impl From<DatabaseError> for DatasetError {
    fn from(err: DatabaseError) -> Self {
        DatasetError::DatabaseError(err.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetMetadata {
    pub id: i64,
    pub table_name: String,
    pub name: String,
    pub description: String,
    pub row_count: i64,
    pub created_at: String, // sqlite doesn't support i64 for timestamp :(
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Column {
    pub id: Option<i64>,
    pub table_name: String,
    pub dataset_id: i64,
    pub name: String,
    pub column_type: String,
    pub column_type_details: Option<String>,
    pub rules: String,
    pub position: i64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UpdatableColumnFields {
    pub name: Option<String>,
    pub rules: Option<String>,
    pub column_type: Option<String>,
    pub column_type_details: Option<String>,
    pub position: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Row {
    pub id: i64,
    pub data: Box<[RowData]>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowData {
    #[serde(rename = "columnId", alias = "column_id")]
    pub column_id: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedResponse {
    pub data: Vec<HashMap<String, serde_json::Value>>,
    pub page: i64,
    pub page_size: i64,
    pub total_rows: i64,
    pub total_pages: i64,
    pub has_next: bool,
    pub has_previous: bool,
}

#[derive(Clone)]
pub struct DatasetService {
    pub db: DatabaseService,
}

impl DatasetService {
    pub fn new(db: DatabaseService) -> Result<Self, DatabaseError> {
        let dataset_service = Self { db };

        dataset_service.create_dataset_metadata_default_table()?;
        dataset_service.create_columns_default_table()?;

        Ok(dataset_service)
    }

    pub fn create_dataset_metadata_default_table(&self) -> SqliteResult<(), DatabaseError> {
        let conn = self
            .db
            .conn
            .lock()
            .map_err(|_| DatabaseError::SqliteError("Failed to acquire mutex lock".to_string()))?;

        conn.execute(
            "
            CREATE TABLE IF NOT EXISTS datasets_metadata (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                table_name TEXT NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
        ",
            [],
        )?;

        conn.execute(
            "
            CREATE INDEX IF NOT EXISTS idx_datasets_metadata_name ON datasets_metadata(name)
        ",
            [],
        )?;

        Ok(())
    }

    pub fn create_columns_default_table(&self) -> SqliteResult<(), DatabaseError> {
        let conn = self
            .db
            .conn
            .lock()
            .map_err(|_| DatabaseError::SqliteError("Failed to acquire mutex lock".to_string()))?;

        conn.execute(
            "
            CREATE TABLE IF NOT EXISTS columns (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                table_name TEXT NOT NULL,
                dataset_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                column_type TEXT NOT NULL,
                column_type_details TEXT DEFAULT '',
                rules TEXT,
                position INTEGER NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (dataset_id) REFERENCES datasets_metadata(id) ON DELETE CASCADE
            )
        ",
            [],
        )?;

        conn.execute(
            "
            CREATE INDEX IF NOT EXISTS idx_column_position ON columns(position)
        ",
            [],
        )?;

        Ok(())
    }

    pub fn create(&self, name: &str, description: &str) -> Result<DatasetMetadata, DatasetError> {
        let next_id = self
            .db
            .conn
            .lock()
            .map_err(|_| DatabaseError::SqliteError("Failed to acquire mutex lock".to_string()))?
            .query_row("SELECT COALESCE(MAX(id), 0) + 1 FROM datasets_metadata", [], |row| {
                row.get::<_, i64>(0)
            })?;

        let table_name = format!("dataset{}", next_id);

        self.db.execute(
            "INSERT INTO datasets_metadata (table_name, name, description, created_at, updated_at) VALUES (?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
            [table_name, name.trim().to_string(), description.trim().to_string()],
        )?;

        let datasets = self
            .db
            .query("SELECT * FROM datasets_metadata ORDER BY id DESC LIMIT 1", [], |row| {
                Ok(DatasetMetadata {
                    id: row.get::<_, i64>(0)?,
                    table_name: row.get::<_, String>(1)?,
                    name: row.get::<_, String>(2)?,
                    description: row.get::<_, String>(3)?,
                    created_at: row.get::<_, String>(4)?,
                    updated_at: row.get::<_, String>(5)?,
                    row_count: 0,
                })
            })?;

        datasets
            .into_iter()
            .next()
            .ok_or_else(|| DatasetError::NotFound(format!("Dataset not found")))
    }

    pub fn find_by_id(&self, id: i64) -> Result<DatasetMetadata, DatasetError> {
        if id <= 0 {
            return Err(DatasetError::InvalidInput(
                "Dataset ID must be a positive integer".to_string(),
            ));
        }

        let datasets = self
            .db
            .query("SELECT * FROM datasets_metadata WHERE id = ?", [id], |row| {
                Ok(DatasetMetadata {
                    id: row.get::<_, i64>(0)?,
                    table_name: row.get::<_, String>(1)?,
                    name: row.get::<_, String>(2)?,
                    description: row.get::<_, String>(3)?,
                    created_at: row.get::<_, String>(4)?,
                    updated_at: row.get::<_, String>(5)?,
                    row_count: 0,
                })
            })?;

        let mut dataset = datasets
            .into_iter()
            .next()
            .ok_or_else(|| DatasetError::NotFound(format!("Dataset not found")))?;

        dataset.row_count = self.count_rows(&dataset.table_name).unwrap_or(0);

        Ok(dataset)
    }

    pub fn find_all(&self) -> Result<Vec<DatasetMetadata>, DatasetError> {
        let mut datasets = self
            .db
            .query("SELECT * FROM datasets_metadata ORDER BY created_at DESC", [], |row| {
                Ok(DatasetMetadata {
                    id: row.get::<_, i64>(0)?,
                    table_name: row.get::<_, String>(1)?,
                    name: row.get::<_, String>(2)?,
                    description: row.get::<_, String>(3)?,
                    created_at: row.get::<_, String>(4)?,
                    updated_at: row.get::<_, String>(5)?,
                    row_count: 0,
                })
            })?;

        for dataset in &mut datasets {
            dataset.row_count = self.count_rows(&dataset.table_name).unwrap_or(0);
        }

        Ok(datasets)
    }

    pub fn update(
        &self,
        id: i64,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<DatasetMetadata, DatasetError> {
        if id <= 0 {
            return Err(DatasetError::InvalidInput(
                "Dataset ID must be a positive integer".to_string(),
            ));
        }

        if let Some(name) = name {
            if name.trim().is_empty() {
                return Err(DatasetError::InvalidInput("Dataset name cannot be empty".to_string()));
            }
            self.db.execute(
                "UPDATE datasets_metadata SET name = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                [name.trim(), &id.to_string()],
            )?;
        }

        if let Some(description) = description {
            self.db.execute(
                "UPDATE datasets_metadata SET description = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                [description, &id.to_string()],
            )?;
        }

        self.find_by_id(id)
    }

    pub fn delete(&self, id: i64) -> Result<(), DatasetError> {
        if id <= 0 {
            return Err(DatasetError::InvalidInput(
                "Dataset ID must be a positive integer".to_string(),
            ));
        }

        self.find_by_id(id)?;

        self.db.execute("DELETE FROM datasets_metadata WHERE id = ?", [id])?;
        Ok(())
    }

    pub fn add_columns(&self, dataset_id: i64, columns: &[Column]) -> Result<Vec<Column>, DatasetError> {
        let dataset_metadata = self.find_by_id(dataset_id)?;

        let table_name = dataset_metadata.table_name;

        if !self.db.table_exists(&table_name)? {
            self.db
                .create_table(&table_name, &["data JSON DEFAULT '{}' CHECK(json_valid(data))"], &[])?;
        }

        let insert_query = "INSERT INTO columns (dataset_id, table_name, name, column_type, column_type_details, rules, position) VALUES (?, ?, ?, ?, ?, ?, ?)";
        self.db.execute_batch(
            &insert_query,
            &columns
                .iter()
                .map(|c| {
                    [
                        c.dataset_id.to_string(),
                        c.table_name.to_string(),
                        c.name.trim().to_string(),
                        c.column_type.trim().to_string(),
                        c.column_type_details
                            .clone()
                            .unwrap_or("".to_string())
                            .trim()
                            .to_string(),
                        c.rules.trim().to_string(),
                        c.position.to_string(),
                    ]
                })
                .collect::<Vec<_>>(),
        )?;

        let new_columns = self.get_columns(dataset_id)?;

        let update_query = format!(
            "UPDATE {} SET data = json_insert(
                data,
                '$[#]',
                json_object('column_id', ?, 'value', ?)
            )
            WHERE json_type(data) = 'array' AND data != '[]'",
            table_name
        );

        let params: Vec<[String; 2]> = new_columns
            .iter()
            .rev()
            .take(columns.len())
            .map(|c| {
                let column_id = c.id.expect("Column should have an ID after insertion");
                [column_id.to_string(), "".to_string()]
            })
            .collect();

        self.db.execute_batch(&update_query, &params)?;

        Ok(new_columns)
    }

    pub fn get_columns(&self, dataset_id: i64) -> Result<Vec<Column>, DatasetError> {
        let columns = self.db.query(
            "SELECT * FROM columns WHERE dataset_id = ? ORDER BY position ASC",
            [dataset_id],
            |row| {
                Ok(Column {
                    id: Some(row.get::<_, i64>(0)?),
                    table_name: row.get::<_, String>(1)?,
                    dataset_id: row.get::<_, i64>(2)?,
                    name: row.get::<_, String>(3)?,
                    column_type: row.get::<_, String>(4)?,
                    column_type_details: Some(row.get::<_, String>(5)?),
                    rules: row.get::<_, String>(6)?,
                    position: row.get::<_, i64>(7)?,
                })
            },
        )?;

        Ok(columns)
    }

    pub fn update_column(&self, id: i64, updates: UpdatableColumnFields) -> Result<Column, DatasetError> {
        let mut set_parts: Vec<String> = Vec::new();
        let mut dyn_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        for (column_name, value_option) in [
            ("name", updates.name.as_ref()),
            ("rules", updates.rules.as_ref()),
            ("column_type", updates.column_type.as_ref()),
            ("column_type_details", updates.column_type_details.as_ref()),
            ("position", updates.position.as_ref()),
        ] {
            if let Some(value) = value_option {
                let trimmed_value = value.trim();
                if !trimmed_value.is_empty() {
                    set_parts.push(format!("{} = ?", column_name));
                    dyn_params.push(Box::new(trimmed_value.to_string()));
                }
            }
        }

        if set_parts.is_empty() {
            return Err(DatasetError::InvalidInput("No updates provided".to_string()));
        }

        let set_clause = format!("{} , updated_at = CURRENT_TIMESTAMP", set_parts.join(", "));
        let query = format!("UPDATE columns SET {} WHERE id = ?", set_clause);

        let mut param_refs: Vec<&dyn rusqlite::ToSql> =
            dyn_params.iter().map(|p| p.as_ref() as &dyn rusqlite::ToSql).collect();
        param_refs.push(&id);

        self.db.execute(&query, &param_refs[..])?;

        let column = self
            .db
            .query("SELECT * FROM columns WHERE id = ?", [id], |row| {
                Ok(Column {
                    id: Some(row.get::<_, i64>(0)?),
                    table_name: row.get::<_, String>(1)?,
                    dataset_id: row.get::<_, i64>(2)?,
                    name: row.get::<_, String>(3)?,
                    column_type: row.get::<_, String>(4)?,
                    column_type_details: Some(row.get::<_, String>(5)?),
                    rules: row.get::<_, String>(6)?,
                    position: row.get::<_, i64>(7)?,
                })
            })?
            .into_iter()
            .next()
            .ok_or_else(|| DatasetError::NotFound(format!("Column with id {} not found", id)))?;

        Ok(column)
    }

    pub fn delete_column(&self, id: i64) -> Result<(), DatasetError> {
        let column = self.db.query("SELECT * FROM columns WHERE id = ?", [id], |row| {
            Ok(Column {
                id: Some(row.get::<_, i64>(0)?),
                table_name: row.get::<_, String>(1)?,
                dataset_id: row.get::<_, i64>(2)?,
                name: row.get::<_, String>(3)?,
                column_type: row.get::<_, String>(4)?,
                column_type_details: Some(row.get::<_, String>(5)?),
                rules: row.get::<_, String>(6)?,
                position: row.get::<_, i64>(7)?,
            })
        })?;

        if column.is_empty() {
            return Err(DatasetError::NotFound(format!("Column with id {} not found", id)));
        }

        let table_name = &column[0].table_name;

        let update_query = format!(
            "UPDATE {} SET data = (
                SELECT COALESCE(json_group_array(value), '[]')
                FROM json_each(data)
                WHERE CAST(json_extract(value, '$.column_id') AS TEXT) != ?
            )
            WHERE EXISTS (
                SELECT 1 FROM json_each(data)
                WHERE CAST(json_extract(value, '$.column_id') AS TEXT) = ?
            )",
            table_name
        );

        let position_str = column[0].position.to_string();
        let dataset_id_str = column[0].dataset_id.to_string();
        let id_str = id.to_string();

        self.db.execute_transaction(&[
            (
                "UPDATE columns SET position = position - 1 WHERE position > ? AND dataset_id = ?",
                &[
                    &position_str as &dyn rusqlite::ToSql,
                    &dataset_id_str as &dyn rusqlite::ToSql,
                ],
            ),
            (
                &update_query,
                &[&id_str as &dyn rusqlite::ToSql, &id_str as &dyn rusqlite::ToSql],
            ),
            ("DELETE FROM columns WHERE id = ?", &[&id_str as &dyn rusqlite::ToSql]),
        ])?;

        Ok(())
    }

    pub fn get_all_rows(&self, table_name: &str) -> Result<Vec<Row>, DatasetError> {
        let rows = self.db.query(
            &format!(
                "SELECT id, data, created_at, updated_at FROM {} ORDER BY id ASC",
                table_name
            ),
            [],
            |row| {
                let data_json: String = row.get(1)?;
                let row_data: Vec<RowData> = serde_json::from_str(&data_json)?;

                Ok(Row {
                    id: row.get::<_, i64>(0)?,
                    data: row_data.into_boxed_slice(),
                    created_at: row.get::<_, String>(2)?,
                    updated_at: row.get::<_, String>(3)?,
                })
            },
        )?;

        Ok(rows)
    }

    pub fn get_rows(&self, dataset_id: i64, page: i64, page_size: i64) -> Result<PaginatedResponse, DatasetError> {
        if page <= 0 {
            return Err(DatasetError::InvalidInput(
                "Page number must be a positive integer".to_string(),
            ));
        }

        if page_size <= 0 {
            return Err(DatasetError::InvalidInput(
                "Page size must be a positive integer".to_string(),
            ));
        }

        let dataset_metadata = self.find_by_id(dataset_id)?;
        let table_name = dataset_metadata.table_name;

        let total_rows = self.count_rows(&table_name)?;
        let total_pages = ((total_rows as f64) / (page_size as f64)).ceil() as i64;

        if total_pages > 0 && page > total_pages {
            return Err(DatasetError::InvalidInput(format!(
                "Page {} exceeds total pages {}",
                page, total_pages
            )));
        }

        let offset = (page - 1) * page_size;

        let table_exists = self.db.table_exists(&table_name)?;
        if !table_exists {
            return Ok(PaginatedResponse {
                data: Vec::new(),
                page,
                page_size,
                total_rows: 0,
                total_pages: 0,
                has_next: false,
                has_previous: false,
            });
        }

        let column_info = self
            .db
            .query(&format!("PRAGMA table_info({})", table_name), [], |row| {
                Ok(row.get::<_, String>(1)?)
            })?;

        let rows = self.db.query(
            &format!("SELECT * FROM {} ORDER BY id ASC LIMIT ? OFFSET ?", table_name),
            [page_size, offset],
            |row| {
                let mut map = HashMap::new();
                for (i, column_name) in column_info.iter().enumerate() {
                    let value: serde_json::Value = if column_name == "data" {
                        if let Ok(text_val) = row.get::<_, String>(i) {
                            serde_json::from_str(&text_val).unwrap_or(serde_json::Value::String(text_val))
                        } else {
                            serde_json::Value::Null
                        }
                    } else if let Ok(int_val) = row.get::<_, i64>(i) {
                        serde_json::Value::String(int_val.to_string())
                    } else if let Ok(text_val) = row.get::<_, String>(i) {
                        serde_json::Value::String(text_val)
                    } else {
                        serde_json::Value::String("".to_string())
                    };
                    map.insert(column_name.clone(), value);
                }
                Ok(map)
            },
        )?;

        Ok(PaginatedResponse {
            data: rows,
            page,
            page_size,
            total_rows,
            total_pages,
            has_next: page < total_pages,
            has_previous: page > 1,
        })
    }

    pub fn add_row(&self, dataset_id: i64, data: &Vec<RowData>) -> Result<Row, DatasetError> {
        let dataset_metadata = self.find_by_id(dataset_id)?;
        let table_name = dataset_metadata.table_name;

        let columns = self.get_columns(dataset_id)?;

        let mut row_data = Vec::new();
        for column in columns {
            let column_id = column
                .id
                .expect("Column should have an ID when retrieved from database");

            let value = data
                .iter()
                .find(|r| r.column_id == column_id.to_string())
                .map(|r| r.value.clone())
                .ok_or_else(|| DatasetError::NotFound(format!("Column with id {} not found", column_id)))?;

            row_data.push(RowData {
                column_id: column_id.to_string(),
                value,
            });
        }

        let json_data = serde_json::to_string(&row_data)?;

        self.db.execute_transaction(&[
            (&format!("INSERT INTO {} (data) VALUES (?)", table_name), &[&json_data]),
            (
                "UPDATE datasets_metadata SET updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                &[&dataset_id.to_string()],
            ),
        ])?;

        let row = self.db.query(
            &format!(
                "SELECT id, data, created_at, updated_at FROM {} ORDER BY id DESC LIMIT 1",
                table_name
            ),
            [],
            |row| {
                let data_json: String = row.get(1)?;
                let row_data: Vec<RowData> = serde_json::from_str(&data_json)?;
                Ok(Row {
                    id: row.get::<_, i64>(0)?,
                    data: row_data.into_boxed_slice(),
                    created_at: row.get::<_, String>(2)?,
                    updated_at: row.get::<_, String>(3)?,
                })
            },
        )?;

        Ok(row
            .into_iter()
            .next()
            .ok_or_else(|| DatasetError::NotFound(format!("Dataset not found")))?)
    }

    pub fn update_row(
        &self,
        dataset_id: i64,
        row_id: i64,
        updates: &HashMap<i64, String>,
    ) -> Result<Row, DatasetError> {
        let dataset_metadata = self.find_by_id(dataset_id)?;
        let table_name = dataset_metadata.table_name;

        let rows = self.db.query(
            &format!("SELECT data FROM {} WHERE id = ?", table_name),
            [row_id],
            |row| {
                row.get::<_, String>(0)
                    .map_err(|e| DatabaseError::SqliteError(e.to_string()))
            },
        )?;

        if rows.is_empty() {
            return Err(DatasetError::NotFound(format!("Row with id {} not found", row_id)));
        }

        let mut row_data: Vec<RowData> = serde_json::from_str(&rows[0])?;

        for data_item in &mut row_data {
            if let Ok(column_id_i64) = data_item.column_id.parse::<i64>() {
                if let Some(new_value) = updates.get(&column_id_i64) {
                    data_item.value = new_value.clone();
                }
            }
        }

        let json_data = serde_json::to_string(&row_data)?;

        self.db.execute_transaction(&[
            (
                &format!(
                    "UPDATE {} SET data = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                    table_name
                ),
                &[&json_data as &dyn rusqlite::ToSql, &row_id],
            ),
            (
                "UPDATE datasets_metadata SET updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                &[&dataset_id as &dyn rusqlite::ToSql],
            ),
        ])?;

        let row = self.db.query(
            &format!(
                "SELECT id, data, created_at, updated_at FROM {} WHERE id = ?",
                table_name
            ),
            [row_id],
            |row| {
                let data_json: String = row.get(1)?;
                let row_data: Vec<RowData> = serde_json::from_str(&data_json)?;

                Ok(Row {
                    id: row.get::<_, i64>(0)?,
                    data: row_data.into_boxed_slice(),
                    created_at: row.get::<_, String>(2)?,
                    updated_at: row.get::<_, String>(3)?,
                })
            },
        )?;

        row.into_iter()
            .next()
            .ok_or_else(|| DatasetError::NotFound(format!("Row with id {} not found", row_id)))
    }

    pub fn delete_row(&self, dataset_id: i64, row_id: i64) -> Result<(), DatasetError> {
        let dataset_metadata = self.find_by_id(dataset_id)?;
        let table_name = dataset_metadata.table_name;

        let rows = self.db.query(
            &format!("SELECT id FROM {} WHERE id = ?", table_name),
            [row_id],
            |row| {
                row.get::<_, i64>(0)
                    .map_err(|e| DatabaseError::SqliteError(e.to_string()))
            },
        )?;

        if rows.is_empty() {
            return Err(DatasetError::NotFound(format!("Row with id {} not found", row_id)));
        }

        self.db.execute_transaction(&[
            (
                &format!("DELETE FROM {} WHERE id = ?", table_name),
                &[&row_id.to_string()],
            ),
            (
                "UPDATE datasets_metadata SET updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                &[&dataset_id.to_string()],
            ),
        ])?;

        Ok(())
    }

    pub fn count_rows(&self, table_name: &str) -> Result<i64, DatasetError> {
        let rows = self
            .db
            .query(&format!("SELECT COUNT(*) FROM {}", table_name), [], |row| {
                Ok(row.get::<_, i64>(0)?)
            })?;

        rows.into_iter()
            .next()
            .ok_or_else(|| DatasetError::DatabaseError("Failed to retrieve row count".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    mod creation {
        use crate::services::ModelService;

        use super::*;

        #[test]
        fn test_create_dataset_metadata_default_table() {
            let db = DatabaseService::new(None).expect("Failed to create database");
            let dataset = DatasetService::new(db).expect("Failed to create dataset service");

            {
                let conn = dataset.db.conn.lock().unwrap();
                conn.execute("DROP TABLE IF EXISTS models", [])
                    .expect("Failed to delete models table");
                conn.execute("DROP TABLE IF EXISTS columns", [])
                    .expect("Failed to delete columns table");
                conn.execute("DROP TABLE IF EXISTS datasets_metadata", [])
                    .expect("Failed to delete datasets_metadata table");
            }

            dataset
                .create_dataset_metadata_default_table()
                .expect("Failed to create datasets_metadata table");

            let conn = dataset.db.conn.lock().unwrap();

            let mut datasets_metadata_stmt = conn
                .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='datasets_metadata'")
                .expect("Failed to prepare query");

            let mut datasets_metadata_index_stmt = conn
                .prepare("SELECT name FROM sqlite_master WHERE type='index' AND name='idx_datasets_metadata_name'")
                .expect("Failed to prepare query");
            let datasets_metadata_exists: bool = datasets_metadata_stmt
                .exists([])
                .expect("Failed to check if table exists");
            let datasets_metadata_index_exists: bool = datasets_metadata_index_stmt
                .exists([])
                .expect("Failed to check if index exists");

            assert!(
                datasets_metadata_index_exists,
                "datasets_metadata index was not created"
            );
            assert!(datasets_metadata_exists, "datasets_metadata table was not created");
        }

        #[test]
        fn test_create_columns_default_table() {
            let db = DatabaseService::new(None).expect("Failed to create database");
            let dataset = DatasetService::new(db).expect("Failed to create dataset service");

            {
                let conn = dataset.db.conn.lock().unwrap();
                conn.execute("DROP TABLE IF EXISTS models", [])
                    .expect("Failed to delete models table");
                conn.execute("DROP TABLE IF EXISTS columns", [])
                    .expect("Failed to delete columns table");
                conn.execute("DROP TABLE IF EXISTS datasets_metadata", [])
                    .expect("Failed to delete datasets_metadata table");
            }

            dataset
                .create_columns_default_table()
                .expect("Failed to create columns table");

            let conn = dataset.db.conn.lock().unwrap();

            let mut columns_stmt = conn
                .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='columns'")
                .expect("Failed to prepare query");
            let mut columns_index_stmt = conn
                .prepare("SELECT name FROM sqlite_master WHERE type='index' AND name='idx_column_position'")
                .expect("Failed to prepare query");

            let columns_exists: bool = columns_stmt.exists([]).expect("Failed to check if table exists");
            let columns_index_exists: bool = columns_index_stmt.exists([]).expect("Failed to check if index exists");

            assert!(columns_index_exists, "columns index was not created");
            assert!(columns_exists, "columns table was not created");
        }

        #[test]
        fn test_new_dataset_connection_to_database() {
            let db = DatabaseService::new(None).expect("Failed to create database");
            let dataset = DatasetService::new(db).expect("Failed to create dataset service");
            let _ = ModelService::new(None, dataset.db.clone()).expect("Failed to create model service");

            let conn = dataset.db.conn.lock().unwrap();

            let mut datasets_metadata_stmt = conn
                .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='datasets_metadata'")
                .expect("Failed to prepare query");
            let datasets_metadata_exists: bool = datasets_metadata_stmt
                .exists([])
                .expect("Failed to check if table exists");
            assert!(datasets_metadata_exists, "datasets_metadata table was not created");

            let mut columns_stmt = conn
                .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='columns'")
                .expect("Failed to prepare query");
            let columns_exists: bool = columns_stmt.exists([]).expect("Failed to check if table exists");
            assert!(columns_exists, "columns table was not created");

            let mut models_stmt = conn
                .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='models'")
                .expect("Failed to prepare query");
            let models_exists: bool = models_stmt.exists([]).expect("Failed to check if table exists");
            assert!(models_exists, "models table was not created");
        }

        #[test]
        fn test_create_dataset() {
            let db = DatabaseService::new(None).expect("Failed to create database");
            let dataset: DatasetService = DatasetService::new(db).expect("Failed to create dataset service");

            let dataset_result = dataset.create("test", "test").expect("Failed to create dataset");
            assert!(dataset_result.name == "test", "Failed to create dataset");
            assert!(dataset_result.description == "test", "Failed to create dataset");

            let conn = dataset.db.conn.lock().unwrap();

            let mut datasets_metadata_stmt = conn
                .prepare(&format!(
                    "SELECT name FROM datasets_metadata WHERE id = {}",
                    dataset_result.id
                ))
                .expect("Failed to prepare query");

            let datasets_metadata_exists: bool = datasets_metadata_stmt
                .exists([])
                .expect("Failed to check if dataset exists");

            assert!(datasets_metadata_exists, "dataset was not created");
        }
    }

    mod queries {
        use super::*;

        #[test]
        fn test_find_by_id_dataset() {
            let db = DatabaseService::new(None).expect("Failed to create database");
            let dataset: DatasetService = DatasetService::new(db).expect("Failed to create dataset service");

            {
                let conn = dataset.db.conn.lock().unwrap();
                conn.execute(
                    "INSERT INTO datasets_metadata (table_name, name, description) VALUES (?, ?, ?)",
                    ["dataset_test", "test", "test"],
                )
                .expect("Failed to insert dataset");
            }

            let dataset_result = dataset.find_by_id(1).expect("Failed to find dataset");

            assert!(dataset_result.name == "test", "Failed to find dataset");
        }

        #[test]
        fn test_find_all_dataset() {
            let db = DatabaseService::new(None).expect("Failed to create database");
            let dataset: DatasetService = DatasetService::new(db).expect("Failed to create dataset service");

            {
                let conn = dataset.db.conn.lock().unwrap();
                conn.execute(
                    "INSERT INTO datasets_metadata (table_name, name, description) VALUES (?, ?, ?)",
                    ["dataset_test", "test", "test"],
                )
                .expect("Failed to insert dataset");
                conn.execute(
                    "INSERT INTO datasets_metadata (table_name, name, description) VALUES (?, ?, ?)",
                    ["dataset_test", "test2", "test2"],
                )
                .expect("Failed to insert dataset");
            }

            let dataset_results = dataset.find_all().expect("Failed to find dataset");

            assert!(dataset_results.len() == 2, "Failed to find dataset");
            assert!(dataset_results[0].name == "test", "Failed to find dataset");
            assert!(dataset_results[1].name == "test2", "Failed to find dataset");
        }

        #[test]
        fn test_get_all_rows() {
            let db = DatabaseService::new(None).expect("Failed to create database");
            let dataset: DatasetService = DatasetService::new(db).expect("Failed to create dataset service");

            {
                let conn = dataset.db.conn.lock().unwrap();

                conn.execute(
                    "CREATE TABLE IF NOT EXISTS dataset_test (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        data JSON DEFAULT '{}' CHECK(json_valid(data)),
                        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                    )",
                    [],
                )
                .expect("failed to create database");

                conn.execute(
                    "INSERT INTO dataset_test (data) VALUES (?)",
                    [r#"[{"column_id":"1","value":"test"}]"#],
                )
                .expect("Failed to insert row 1");

                conn.execute(
                    "INSERT INTO dataset_test (data) VALUES (?)",
                    [r#"[{"column_id":"1","value":"hello"},{"column_id":"2","value":"world"}]"#],
                )
                .expect("Failed to insert row 2");

                conn.execute(
                    "INSERT INTO dataset_test (data) VALUES (?)",
                    [r#"[{"column_id":"1","value":"test with \"quotes\""},{"column_id":"2","value":"123"},{"column_id":"3","value":"special: !@#$%"}]"#],
                )
                .expect("Failed to insert row 3");

                conn.execute(
                    "INSERT INTO dataset_test (data) VALUES (?)",
                    [r#"[{"column_id":"1","value":""}]"#],
                )
                .expect("Failed to insert row 4");

                let large_text = "a".repeat(1000);
                let large_row_data = format!(r#"[{{"column_id":"1","value":"{}"}}]"#, large_text);
                conn.execute("INSERT INTO dataset_test (data) VALUES (?)", [large_row_data.as_str()])
                    .expect("Failed to insert row 5");
            }

            let rows = dataset.get_all_rows("dataset_test").expect("Failed to get all rows");
            assert_eq!(rows.len(), 5, "Should return exactly 5 rows");

            for i in 0..rows.len() {
                assert_eq!(rows[i].id, (i + 1) as i64, "Rows should be ordered by id ascending");
            }

            assert_eq!(rows[0].data.len(), 1, "First row should have 1 column");
            assert_eq!(rows[0].data[0].column_id, "1", "First row column_id should be '1'");
            assert_eq!(rows[0].data[0].value, "test", "First row value should be 'test'");

            assert_eq!(rows[1].data.len(), 2, "Second row should have 2 columns");
            assert_eq!(
                rows[1].data[0].column_id, "1",
                "Second row first column_id should be '1'"
            );
            assert_eq!(
                rows[1].data[0].value, "hello",
                "Second row first value should be 'hello'"
            );
            assert_eq!(
                rows[1].data[1].column_id, "2",
                "Second row second column_id should be '2'"
            );
            assert_eq!(
                rows[1].data[1].value, "world",
                "Second row second value should be 'world'"
            );

            assert_eq!(rows[2].data.len(), 3, "Third row should have 3 columns");
            assert_eq!(
                rows[2].data[0].value, "test with \"quotes\"",
                "Should handle escaped quotes"
            );
            assert_eq!(rows[2].data[1].value, "123", "Should handle numeric strings");
            assert_eq!(
                rows[2].data[2].value, "special: !@#$%",
                "Should handle special characters"
            );

            assert_eq!(rows[3].data.len(), 1, "Fourth row should have 1 column");
            assert_eq!(rows[3].data[0].value, "", "Should handle empty string values");

            assert_eq!(rows[4].data.len(), 1, "Fifth row should have 1 column");
            assert_eq!(rows[4].data[0].value.len(), 1000, "Should handle large text values");
            assert!(
                rows[4].data[0].value.chars().all(|c| c == 'a'),
                "Large text should be all 'a's"
            );

            for (idx, row) in rows.iter().enumerate() {
                assert!(
                    !row.created_at.is_empty(),
                    "Row {} should have created_at timestamp",
                    idx
                );
                assert!(
                    !row.updated_at.is_empty(),
                    "Row {} should have updated_at timestamp",
                    idx
                );
            }

            let mut ids: Vec<i64> = rows.iter().map(|r| r.id).collect();
            ids.sort();
            ids.dedup();
            assert_eq!(ids.len(), 5, "All rows should have unique ids");
        }
    }

    mod updates {
        use super::*;

        #[test]
        fn test_update_dataset() {
            let db = DatabaseService::new(None).expect("Failed to create database");
            let dataset: DatasetService = DatasetService::new(db).expect("Failed to create dataset service");

            {
                let conn = dataset.db.conn.lock().unwrap();
                conn.execute(
                    "INSERT INTO datasets_metadata (table_name, name, description) VALUES (?, ?, ?)",
                    ["dataset_test", "test", "test"],
                )
                .expect("Failed to insert dataset");
            }

            let updated_dataset = dataset
                .update(1, Some("testUpdated0"), Some("testUpdated"))
                .expect("Failed to update dataset");
            assert!(updated_dataset.name == "testUpdated0", "Failed to update dataset");
            assert!(updated_dataset.description == "testUpdated", "Failed to update dataset");

            let conn = dataset.db.conn.lock().unwrap();

            let result = conn
                .query_row("SELECT * FROM datasets_metadata WHERE id = 1", [], |row| {
                    Ok(DatasetMetadata {
                        id: row.get(0)?,
                        table_name: row.get(1)?,
                        name: row.get(2)?,
                        description: row.get(3)?,
                        created_at: row.get(4)?,
                        updated_at: row.get(5)?,
                        row_count: 0,
                    })
                })
                .expect("Failed to query dataset");

            assert_eq!(result.name, "testUpdated0", "Failed to update dataset");
            assert_eq!(result.description, "testUpdated", "Failed to update dataset");
        }

        #[test]
        fn test_delete_dataset() {
            let db = DatabaseService::new(None).expect("Failed to create database");
            let dataset: DatasetService = DatasetService::new(db).expect("Failed to create dataset service");

            {
                let conn = dataset.db.conn.lock().unwrap();
                conn.execute(
                    "INSERT INTO datasets_metadata (table_name, name, description) VALUES (?, ?, ?)",
                    ["dataset_test", "test", "test"],
                )
                .expect("Failed to insert dataset");
            }

            let deleted_dataset = dataset.delete(1);
            assert!(deleted_dataset.is_ok(), "Failed to delete dataset");

            let conn = dataset.db.conn.lock().unwrap();

            let mut datasets_metadata_stmt = conn
                .prepare("SELECT * FROM datasets_metadata WHERE id = 1")
                .expect("Failed to prepare query");
            let datasets_metadata_exists: bool = datasets_metadata_stmt
                .exists([])
                .expect("Failed to check if dataset exists");

            assert!(!datasets_metadata_exists, "dataset was not deleted");
        }
    }

    mod columns {
        use super::*;

        #[test]
        fn test_dataset_add_columns() {
            let db = DatabaseService::new(None).expect("Failed to create database");
            let dataset: DatasetService = DatasetService::new(db).expect("Failed to create dataset service");

            let columns = vec![
                Column {
                    id: None,
                    table_name: "dataset001".to_string(),
                    dataset_id: 1,
                    name: "test".to_string(),
                    column_type: "TEXT".to_string(),
                    column_type_details: None,
                    rules: "test".to_string(),
                    position: 1,
                },
                Column {
                    id: None,
                    table_name: "dataset001".to_string(),
                    dataset_id: 1,
                    name: "test2".to_string(),
                    column_type: "TEXT".to_string(),
                    column_type_details: None,
                    rules: "test2".to_string(),
                    position: 2,
                },
            ];

            {
                let conn = dataset.db.conn.lock().unwrap();
                conn.execute(
                    "INSERT INTO datasets_metadata (table_name, name, description) VALUES (?, ?, ?)",
                    ["dataset001", "test", "test"],
                )
                .expect("Failed to insert dataset");
            }

            let added_columns = dataset.add_columns(1, &columns);
            assert!(added_columns.is_ok(), "Failed to add columns");

            let conn = dataset.db.conn.lock().unwrap();

            let mut columns_stmt = conn
                .prepare("SELECT * FROM columns WHERE dataset_id = 1")
                .expect("Failed to prepare query");

            let columns_map = columns_stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, i64>(6)?,
                    ))
                })
                .expect("Failed to query columns");

            let columns_length = columns_map.count();
            assert!(columns_length > 1, "columns were not added");

            let mut dataset_001_stmt = conn
                .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='dataset001'")
                .expect("Failed to prepare query");
            let dataset_001_exists: bool = dataset_001_stmt
                .exists([])
                .expect("Failed to check if dataset_001 exists");
            assert!(dataset_001_exists, "dataset001 was not created");
        }

        #[test]
        fn test_dataset_add_columns_with_existing_table() {
            let db = DatabaseService::new(None).expect("Failed to create database");
            let dataset: DatasetService = DatasetService::new(db).expect("Failed to create dataset service");

            let columns = vec![
                Column {
                    id: None,
                    table_name: "dataset001".to_string(),
                    dataset_id: 1,
                    name: "test".to_string(),
                    column_type: "TEXT".to_string(),
                    column_type_details: None,
                    rules: "test".to_string(),
                    position: 1,
                },
                Column {
                    id: None,
                    table_name: "dataset001".to_string(),
                    dataset_id: 1,
                    name: "test2".to_string(),
                    column_type: "TEXT".to_string(),
                    column_type_details: None,
                    rules: "test2".to_string(),
                    position: 2,
                },
            ];

            {
                let conn = dataset.db.conn.lock().unwrap();
                conn.execute(
                    "INSERT INTO datasets_metadata (table_name, name, description) VALUES (?, ?, ?)",
                    ["dataset001", "test", "test"],
                )
                .expect("Failed to insert dataset");

                conn.execute(
                    "CREATE TABLE IF NOT EXISTS dataset001 (
                            id INTEGER PRIMARY KEY AUTOINCREMENT,
                            data JSON DEFAULT '{}' CHECK(json_valid(data)),
                            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                        )",
                    [],
                )
                .expect("failed to create database");

                conn.execute(
                    "INSERT INTO columns (dataset_id, table_name, name, column_type, rules, position) VALUES (?, ?, ?, ?, ?, ?)",
                    ["1", "dataset001", "test", "TEXT", "test", "1"],
                )
                .expect("Failed to insert dataset");

                conn.execute(
                    "INSERT INTO dataset001 (data) VALUES (?)",
                    ["[{ \"column_id\": \"1\", \"value\": \"test\" }]"],
                )
                .expect("Failed to insert dataset");
            }

            let added_columns = dataset.add_columns(1, &columns);
            assert!(added_columns.is_ok(), "Failed to add columns");

            let conn = dataset.db.conn.lock().unwrap();
            let mut dataset_001_stmt = conn
                .prepare("SELECT * FROM dataset001")
                .expect("Failed to prepare query");

            let values: Result<Vec<_>, _> = dataset_001_stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                })
                .expect("Failed to query dataset")
                .collect();

            let values = values
                .expect("Failed to collect values")
                .into_iter()
                .map(|(_, data_json, _, _)| data_json);

            let expected_column_ids = ["2", "3"];

            for data_json in values {
                let row_data: Vec<RowData> = serde_json::from_str(&data_json).expect("Failed to parse JSON data");

                let found_column_ids: std::collections::HashSet<&str> =
                    row_data.iter().map(|item| item.column_id.as_str()).collect();

                for expected_id in expected_column_ids {
                    assert!(
                        found_column_ids.contains(expected_id),
                        "Row data missing expected column ID: '{}'. Found columns: {:?}",
                        expected_id,
                        found_column_ids
                    );
                }
            }
        }

        #[test]
        fn test_dataset_get_columns() {
            let db = DatabaseService::new(None).expect("Failed to create database");
            let dataset: DatasetService = DatasetService::new(db).expect("Failed to create dataset service");

            {
                let conn = dataset.db.conn.lock().unwrap();
                conn.execute(
                    "INSERT INTO datasets_metadata (table_name, name, description) VALUES (?, ?, ?)",
                    ["dataset001", "test", "test"],
                )
                .expect("Failed to insert dataset");

                conn.execute(
                    "INSERT INTO columns (dataset_id, table_name, name, column_type, rules, position) VALUES (?, ?, ?, ?, ?, ?)",
                    ["1", "dataset001", "test1", "TEXT", "test", "1"],
                )
                .expect("Failed to insert dataset");

                conn.execute(
                    "INSERT INTO columns (dataset_id, table_name, name, column_type, rules, position) VALUES (?, ?, ?, ?, ?, ?)",
                    ["1", "dataset001", "test2", "NUMBER", "test", "1"],
                )
                .expect("Failed to insert dataset");
            }

            let columns = dataset.get_columns(1);
            assert!(columns.is_ok(), "Failed to get columns");
            assert!(columns.unwrap().len() == 2, "Failed to get columns");
        }

        #[test]
        fn test_dataset_update_column() {
            let db = DatabaseService::new(None).expect("Failed to create database");
            let dataset: DatasetService = DatasetService::new(db).expect("Failed to create dataset service");

            {
                let conn = dataset.db.conn.lock().unwrap();
                conn.execute(
                    "INSERT INTO datasets_metadata (table_name, name, description) VALUES (?, ?, ?)",
                    ["dataset001", "test", "test"],
                )
                .expect("Failed to insert dataset");

                conn.execute(
                    "INSERT INTO columns (dataset_id, table_name, name, column_type, rules, position) VALUES (?, ?, ?, ?, ?, ?)",
                    ["1", "dataset001", "test1", "TEXT", "test", "1"],
                )
                .expect("Failed to insert dataset");

                conn.execute(
                    "INSERT INTO columns (dataset_id, table_name, name, column_type, rules, position) VALUES (?, ?, ?, ?, ?, ?)",
                    ["1", "dataset001", "test2", "NUMBER", "test", "1"],
                )
                .expect("Failed to insert dataset");
            }

            let updated_column = dataset.update_column(
                1,
                UpdatableColumnFields {
                    name: Some("test1".to_string()),
                    rules: Some("test".to_string()),
                    column_type: None,
                    column_type_details: None,
                    position: Some("1".to_string()),
                },
            );
            assert!(updated_column.is_ok(), "Failed to update column");

            let conn = dataset.db.conn.lock().unwrap();

            let column = conn
                .query_row(
                    "SELECT id, table_name, dataset_id, name, column_type, column_type_details, rules, position FROM columns WHERE id = 1",
                    [],
                    |row| {
                        Ok(Column {
                            id: Some(row.get::<_, i64>(0)?),
                            table_name: row.get::<_, String>(1)?,
                            dataset_id: row.get::<_, i64>(2)?,
                            name: row.get::<_, String>(3)?,
                            column_type: row.get::<_, String>(4)?,
                            column_type_details: Some(row.get::<_, String>(5)?),
                            rules: row.get::<_, String>(6)?,
                            position: row.get::<_, i64>(7)?
                        })
                    },
                )
                .expect("Failed to query column");

            assert_eq!(column.name, "test1", "Failed to update column");
            assert_eq!(column.rules, "test", "Failed to update column");
            assert_eq!(column.position, 1, "Failed to update column");
        }

        #[test]
        fn test_dataset_delete_column() {
            let db = DatabaseService::new(None).expect("Failed to create database");
            let dataset: DatasetService = DatasetService::new(db).expect("Failed to create dataset service");

            {
                let conn = dataset.db.conn.lock().unwrap();
                conn.execute(
                    "INSERT INTO datasets_metadata (table_name, name, description) VALUES (?, ?, ?)",
                    ["dataset001", "test", "test"],
                )
                .expect("Failed to insert dataset");

                conn.execute(
                    "CREATE TABLE IF NOT EXISTS dataset001 (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        data JSON DEFAULT '{}' CHECK(json_valid(data)),
                        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                    )",
                    [],
                )
                .expect("failed to create database");

                conn.execute(
                    "INSERT INTO columns (dataset_id, table_name, name, column_type, rules, position) VALUES (?, ?, ?, ?, ?, ?)",
                    ["1", "dataset001", "test1", "TEXT", "test", "1"],
                )
                .expect("Failed to insert dataset");

                conn.execute(
                    "INSERT INTO columns (dataset_id, table_name, name, column_type, rules, position) VALUES (?, ?, ?, ?, ?, ?)",
                    ["1", "dataset001", "test2", "NUMBER", "test", "2"],
                )
                .expect("Failed to insert dataset");

                conn.execute(
                    "INSERT INTO dataset001 (data) VALUES (?)",
                    [r#"[{"column_id": "1", "value": "John"}, {"column_id": "2", "value": "30"}]"#],
                )
                .expect("Failed to insert data row 1");

                conn.execute(
                    "INSERT INTO dataset001 (data) VALUES (?)",
                    [r#"[{"column_id": "1", "value": "Jane"}, {"column_id": "2", "value": "25"}]"#],
                )
                .expect("Failed to insert data row 2");
            }

            let deleted_column = dataset.delete_column(1);
            assert!(
                deleted_column.is_ok(),
                "Failed to delete column: {:?}",
                deleted_column.err()
            );

            let conn = dataset.db.conn.lock().unwrap();

            let mut column_stmt = conn
                .prepare("SELECT * FROM columns WHERE id = 1")
                .expect("Failed to check if column exists");
            let exists: bool = column_stmt.exists([]).expect("Failed to check if table exists");
            assert!(!exists, "Failed to delete column from columns table");

            let mut data_stmt = conn
                .prepare("SELECT data FROM dataset001")
                .expect("Failed to prepare data query");
            let data_rows: Vec<String> = data_stmt
                .query_map([], |row| row.get(0))
                .expect("Failed to query data")
                .collect::<Result<Vec<_>, _>>()
                .expect("Failed to collect data");

            for data_json in data_rows {
                let row_data: Vec<RowData> = serde_json::from_str(&data_json).expect("Failed to parse JSON data");

                for item in &row_data {
                    assert_ne!(item.column_id, "1", "Column data was not removed from rows");
                }

                let has_column_2 = row_data.iter().any(|item| item.column_id == "2");
                assert!(has_column_2, "Other column data was incorrectly removed");
            }
        }
    }

    mod rows {
        use super::*;

        #[test]
        fn test_dataset_get_rows() {
            let db = DatabaseService::new(None).expect("Failed to create database");
            let dataset = DatasetService::new(db).expect("Failed to create dataset service");

            {
                let conn = dataset.db.conn.lock().unwrap();
                conn.execute(
                    "INSERT INTO datasets_metadata (table_name, name, description) VALUES (?, ?, ?)",
                    ["dataset001", "test", "test"],
                )
                .expect("Failed to insert dataset");

                conn.execute(
                    "CREATE TABLE IF NOT EXISTS dataset001 (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        data JSON DEFAULT '{}' CHECK(json_valid(data)),
                        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                    )",
                    [],
                )
                .expect("failed to create database");

                conn.execute(
                    "INSERT INTO columns (dataset_id, table_name, name, column_type, rules, position) VALUES (?, ?, ?, ?, ?, ?)",
                    ["1", "dataset001", "test1", "TEXT", "test", "1"],
                )
                .expect("Failed to insert dataset");

                conn.execute(
                    "INSERT INTO columns (dataset_id, table_name, name, column_type, rules, position) VALUES (?, ?, ?, ?, ?, ?)",
                    ["1", "dataset001", "test2", "NUMBER", "test", "2"],
                )
                .expect("Failed to insert dataset");

                for i in 0..11 {
                    let data = format!(
                        r#"[{{ "column_id": "1", "value": "John"}}, {{ "column_id": "2", "value": "{}"}}]"#,
                        i
                    );

                    let data = data.as_str();
                    conn.execute("INSERT INTO dataset001 (data) VALUES (?)", [data])
                        .expect("Failed to insert data row 1");
                }
            }

            let next_rows = dataset.get_rows(1, 1, 5);
            assert!(next_rows.is_ok(), "Failed to get next rows");
            assert!(next_rows.unwrap().data.len() == 5, "Failed to get next rows");
        }

        #[test]
        fn test_dataset_count_rows() {
            let db = DatabaseService::new(None).expect("Failed to create database");
            let dataset = DatasetService::new(db).expect("Failed to create dataset service");

            {
                let conn = dataset.db.conn.lock().unwrap();
                conn.execute(
                    "INSERT INTO datasets_metadata (table_name, name, description) VALUES (?, ?, ?)",
                    ["dataset001", "test", "test"],
                )
                .expect("Failed to insert dataset");

                conn.execute(
                    "CREATE TABLE IF NOT EXISTS dataset001 (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        data JSON DEFAULT '{}' CHECK(json_valid(data)),
                        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                    )",
                    [],
                )
                .expect("failed to create database");

                conn.execute(
                    "INSERT INTO columns (dataset_id, table_name, name, column_type, rules, position) VALUES (?, ?, ?, ?, ?, ?)",
                    ["1", "dataset001", "test1", "TEXT", "test", "1"],
                )
                .expect("Failed to insert dataset");

                conn.execute(
                    "INSERT INTO columns (dataset_id, table_name, name, column_type, rules, position) VALUES (?, ?, ?, ?, ?, ?)",
                    ["1", "dataset001", "test2", "NUMBER", "test", "2"],
                )
                .expect("Failed to insert dataset");

                for i in 0..10 {
                    let data = format!(
                        r#"[{{ "column_id": "1", "value": "John"}}, {{ "column_id": "2", "value": "{}"}}]"#,
                        i
                    );

                    let data = data.as_str();
                    conn.execute("INSERT INTO dataset001 (data) VALUES (?)", [data])
                        .expect("Failed to insert data row 1");
                }
            }

            let count = dataset
                .count_rows("dataset001")
                .expect("Failed to  call count_rows function");
            assert!(count == 10, "Failed to count rows");
        }

        #[test]
        fn test_dataset_update_row() {
            let db = DatabaseService::new(None).expect("Failed to create database");
            let dataset = DatasetService::new(db).expect("Failed to create dataset service");

            {
                let conn = dataset.db.conn.lock().unwrap();

                conn.execute(
                    "INSERT INTO datasets_metadata (table_name, name, description) VALUES (?, ?, ?)",
                    ["dataset001", "test", "test"],
                )
                .expect("Failed to insert dataset");

                conn.execute(
                    "CREATE TABLE IF NOT EXISTS dataset001 (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        data JSON DEFAULT '{}' CHECK(json_valid(data)),
                        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                    )",
                    [],
                )
                .expect("failed to create database");

                conn.execute(
                    "INSERT INTO columns (dataset_id, table_name, name, column_type, rules, position) VALUES (?, ?, ?, ?, ?, ?)",
                    ["1", "dataset001", "test1", "TEXT", "test", "1"],
                )
                .expect("Failed to insert dataset");

                conn.execute(
                    "INSERT INTO columns (dataset_id, table_name, name, column_type, rules, position) VALUES (?, ?, ?, ?, ?, ?)",
                    ["1", "dataset001", "test2", "NUMBER", "test", "2"],
                )
                .expect("Failed to insert dataset");

                conn.execute(
                    "INSERT INTO dataset001 (data) VALUES (?)",
                    [r#"[{"column_id": "1", "value": "John"}, {"column_id": "2", "value": "30"}]"#],
                )
                .expect("Failed to insert data row 1");

                conn.execute(
                    "INSERT INTO dataset001 (data) VALUES (?)",
                    [r#"[{"column_id": "1", "value": "Jane"}, {"column_id": "2", "value": "25"}]"#],
                )
                .expect("Failed to insert data row 2");
            }

            let updated_row =
                dataset.update_row(1, 2, &HashMap::from([(2, "30".to_string()), (1, "Johnny".to_string())]));
            assert!(updated_row.is_ok(), "Failed to update row");

            let conn = dataset.db.conn.lock().unwrap();

            let row = conn
                .query_row("SELECT * FROM dataset001 WHERE id = 2", [], |row| {
                    Ok(Row {
                        id: row.get::<_, i64>(0)?,
                        data: serde_json::from_str(&row.get::<_, String>(1)?).expect("Failed to parse JSON data"),
                        created_at: row.get::<_, String>(2)?,
                        updated_at: row.get::<_, String>(3)?,
                    })
                })
                .expect("Failed to query row");

            assert_eq!(row.data[0].value, "Johnny", "Failed to update row");
            assert_eq!(row.data[1].value, "30", "Failed to update row");
        }

        #[test]
        fn test_add_row() {
            let db = DatabaseService::new(None).expect("Failed to create database");
            let dataset = DatasetService::new(db).expect("Failed to create dataset service");

            {
                let conn = dataset.db.conn.lock().unwrap();

                conn.execute(
                    "INSERT INTO datasets_metadata (table_name, name, description) VALUES (?, ?, ?)",
                    ["dataset001", "test", "test"],
                )
                .expect("Failed to insert dataset");

                conn.execute(
                    "CREATE TABLE IF NOT EXISTS dataset001 (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        data JSON DEFAULT '{}' CHECK(json_valid(data)),
                        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                    )",
                    [],
                )
                .expect("failed to create database");

                conn.execute(
                    "INSERT INTO columns (dataset_id, table_name, name, column_type, rules, position) VALUES (?, ?, ?, ?, ?, ?)",
                    ["1", "dataset001", "test1", "TEXT", "test", "1"],
                )
                .expect("Failed to insert dataset");

                conn.execute(
                    "INSERT INTO columns (dataset_id, table_name, name, column_type, rules, position) VALUES (?, ?, ?, ?, ?, ?)",
                    ["1", "dataset001", "test2", "NUMBER", "test", "2"],
                )
                .expect("Failed to insert dataset");

                conn.execute(
                    "INSERT INTO dataset001 (data) VALUES (?)",
                    [r#"[{"column_id": "1", "value": "John"}, {"column_id": "2", "value": "30"}]"#],
                )
                .expect("Failed to insert data row 1");
            }

            let new_row = dataset.add_row(
                1,
                &vec![
                    RowData {
                        column_id: "1".to_string(),
                        value: "John".to_string(),
                    },
                    RowData {
                        column_id: "2".to_string(),
                        value: "30".to_string(),
                    },
                ],
            );
            assert!(new_row.is_ok(), "Failed to add row");

            let conn = dataset.db.conn.lock().unwrap();

            let row = conn
                .query_row("SELECT * FROM dataset001 WHERE id = 1", [], |row| {
                    Ok(Row {
                        id: row.get::<_, i64>(0)?,
                        data: serde_json::from_str(&row.get::<_, String>(1)?).expect("Failed to parse JSON data"),
                        created_at: row.get::<_, String>(2)?,
                        updated_at: row.get::<_, String>(3)?,
                    })
                })
                .expect("Failed to query row");

            assert_eq!(row.data[0].value, "John", "Failed to add row");
            assert_eq!(row.data[1].value, "30", "Failed to add row");
        }

        #[test]
        fn test_delete_row() {
            let db = DatabaseService::new(None).expect("Failed to create database");
            let dataset = DatasetService::new(db).expect("Failed to create dataset service");

            {
                let conn = dataset.db.conn.lock().unwrap();

                conn.execute(
                    "INSERT INTO datasets_metadata (table_name, name, description) VALUES (?, ?, ?)",
                    ["dataset001", "test", "test"],
                )
                .expect("Failed to insert dataset");

                conn.execute(
                    "CREATE TABLE IF NOT EXISTS dataset001 (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        data JSON DEFAULT '{}' CHECK(json_valid(data)),
                        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                    )",
                    [],
                )
                .expect("failed to create database");

                conn.execute(
                    "INSERT INTO columns (dataset_id, table_name, name, column_type, rules, position) VALUES (?, ?, ?, ?, ?, ?)",
                    ["1", "dataset001", "test1", "TEXT", "test", "1"],
                )
                .expect("Failed to insert dataset");

                conn.execute(
                    "INSERT INTO columns (dataset_id, table_name, name, column_type, rules, position) VALUES (?, ?, ?, ?, ?, ?)",
                    ["1", "dataset001", "test2", "NUMBER", "test", "2"],
                )
                .expect("Failed to insert dataset");

                conn.execute(
                    "INSERT INTO dataset001 (data) VALUES (?)",
                    [r#"[{"column_id": "1", "value": "John"}, {"column_id": "2", "value": "30"}]"#],
                )
                .expect("Failed to insert data row 1");

                conn.execute(
                    "INSERT INTO dataset001 (data) VALUES (?)",
                    [r#"[{"column_id": "1", "value": "Jane"}, {"column_id": "2", "value": "25"}]"#],
                )
                .expect("Failed to insert data row 2");
            }

            let deleted_row = dataset.delete_row(1, 2);
            assert!(deleted_row.is_ok(), "Failed to delete row");
        }
    }
}
