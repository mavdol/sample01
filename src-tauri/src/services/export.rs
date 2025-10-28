use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::Write;

use crate::services::database::DatabaseError;
use crate::services::dataset::{Column, DatasetError, Row};
use crate::services::{DatabaseService, DatasetService};

#[derive(Debug)]
pub enum ExportError {
    NotFound(String),
    DatabaseError(String),
    FsError(String),
    InvalidInput(String),
    DatasetError(String),
}

impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExportError::NotFound(msg) => write!(f, "Not found: {}", msg),
            ExportError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            ExportError::FsError(msg) => write!(f, "File system error: {}", msg),
            ExportError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            ExportError::DatasetError(msg) => write!(f, "Dataset error: {}", msg),
        }
    }
}

impl std::error::Error for ExportError {}

impl From<rusqlite::Error> for ExportError {
    fn from(err: rusqlite::Error) -> Self {
        ExportError::DatabaseError(err.to_string())
    }
}

impl From<std::io::Error> for ExportError {
    fn from(err: std::io::Error) -> Self {
        ExportError::FsError(err.to_string())
    }
}

impl From<serde_json::Error> for ExportError {
    fn from(err: serde_json::Error) -> Self {
        ExportError::FsError(err.to_string())
    }
}

impl From<DatabaseError> for ExportError {
    fn from(err: DatabaseError) -> Self {
        ExportError::DatabaseError(err.to_string())
    }
}

impl From<DatasetError> for ExportError {
    fn from(err: DatasetError) -> Self {
        ExportError::DatasetError(err.to_string())
    }
}

#[derive(Clone)]
pub struct ExportService {
    pub db: DatabaseService,
    pub dataset_service: DatasetService,
}

impl ExportService {
    pub fn new(db: DatabaseService, dataset_service: DatasetService) -> Self {
        Self { db, dataset_service }
    }

    pub fn export_to_csv(&self, dataset_id: i64, file_path: &str) -> Result<(), ExportError> {
        if dataset_id <= 0 {
            return Err(ExportError::InvalidInput(
                "Dataset ID must be a positive integer".to_string(),
            ));
        }

        let dataset_metadata = self.dataset_service.find_by_id(dataset_id)?;
        let table_name = &dataset_metadata.table_name;

        let columns = self.dataset_service.get_columns(dataset_id)?;
        if columns.is_empty() {
            return Err(ExportError::NotFound("No columns found for this dataset".to_string()));
        }

        let rows = self.dataset_service.get_all_rows(table_name)?;

        let csv_content = self.create_csv_content(&columns, &rows)?;

        self.write_to_file(file_path, &csv_content)?;

        Ok(())
    }

    pub fn create_csv_content(&self, columns: &[Column], rows: &[Row]) -> Result<String, ExportError> {
        let mut csv_content = String::new();

        let headers: Vec<String> = columns.iter().map(|c| self.escape_csv_field(&c.name)).collect();
        csv_content.push_str(&headers.join(","));
        csv_content.push('\n');

        for row in rows {
            let mut row_values = Vec::new();

            let value_map: HashMap<String, String> = row
                .data
                .iter()
                .map(|rd| (rd.column_id.clone(), rd.value.clone()))
                .collect();

            for column in columns {
                let column_id = column.id.expect("Column should have an ID").to_string();
                let value = value_map.get(&column_id).cloned().unwrap_or_else(|| "".to_string());
                row_values.push(self.escape_csv_field(&value));
            }

            csv_content.push_str(&row_values.join(","));
            csv_content.push('\n');
        }

        Ok(csv_content)
    }

    pub fn escape_csv_field(&self, field: &str) -> String {
        if field.contains(',') || field.contains('"') || field.contains('\n') || field.contains('\r') {
            let escaped = field.replace('"', "\"\"");
            format!("\"{}\"", escaped)
        } else {
            field.to_string()
        }
    }

    pub fn write_to_file(&self, file_path: &str, content: &str) -> Result<(), ExportError> {
        let mut file = File::create(file_path)?;
        file.write_all(content.as_bytes())?;
        file.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    mod creation {
        use super::*;

        #[test]
        fn test_new_export_service() {
            let db = DatabaseService::new(None).expect("Failed to create database");
            let dataset_service = DatasetService::new(db.clone()).expect("Failed to create dataset service");
            let export = ExportService::new(db, dataset_service);

            assert!(
                export.db.conn.lock().is_ok(),
                "Export service should have valid database connection"
            );
        }
    }

    mod csv_processing {
        use super::*;
        use crate::services::dataset::RowData;

        #[test]
        fn test_create_csv_content() {
            let db = DatabaseService::new(None).expect("Failed to create database");
            let dataset_service = DatasetService::new(db.clone()).expect("Failed to create dataset service");
            let export = ExportService::new(db, dataset_service);

            let columns = vec![
                Column {
                    id: Some(1),
                    dataset_id: 1,
                    table_name: "test_table".to_string(),
                    name: "name".to_string(),
                    column_type: "TEXT".to_string(),
                    column_type_details: None,
                    rules: "Name column".to_string(),
                    position: 1,
                },
                Column {
                    id: Some(2),
                    dataset_id: 1,
                    table_name: "test_table".to_string(),
                    name: "age".to_string(),
                    column_type: "INTEGER".to_string(),
                    column_type_details: None,
                    rules: "Age column".to_string(),
                    position: 2,
                },
            ];

            let rows = vec![
                Row {
                    id: 1,
                    data: vec![
                        RowData {
                            column_id: "1".to_string(),
                            value: "John".to_string(),
                        },
                        RowData {
                            column_id: "2".to_string(),
                            value: "25".to_string(),
                        },
                    ]
                    .into_boxed_slice(),
                    created_at: "2023-01-01".to_string(),
                    updated_at: "2023-01-01".to_string(),
                },
                Row {
                    id: 2,
                    data: vec![
                        RowData {
                            column_id: "1".to_string(),
                            value: "Jane".to_string(),
                        },
                        RowData {
                            column_id: "2".to_string(),
                            value: "30".to_string(),
                        },
                    ]
                    .into_boxed_slice(),
                    created_at: "2023-01-01".to_string(),
                    updated_at: "2023-01-01".to_string(),
                },
            ];

            let csv_content = export
                .create_csv_content(&columns, &rows)
                .expect("Should create CSV content");
            let lines: Vec<&str> = csv_content.lines().collect();

            assert_eq!(lines[0], "name,age");
            assert_eq!(lines[1], "John,25");
            assert_eq!(lines[2], "Jane,30");
        }

        #[test]
        fn test_create_csv_content_empty_rows() {
            let db = DatabaseService::new(None).expect("Failed to create database");
            let dataset_service = DatasetService::new(db.clone()).expect("Failed to create dataset service");
            let export = ExportService::new(db, dataset_service);

            let columns = vec![Column {
                id: Some(1),
                dataset_id: 1,
                table_name: "test_table".to_string(),
                name: "name".to_string(),
                column_type: "TEXT".to_string(),
                column_type_details: None,
                rules: "Name column".to_string(),
                position: 1,
            }];

            let rows = vec![];

            let csv_content = export
                .create_csv_content(&columns, &rows)
                .expect("Should create CSV content");
            let lines: Vec<&str> = csv_content.lines().collect();

            assert_eq!(lines.len(), 1);
            assert_eq!(lines[0], "name");
        }

        #[test]
        fn test_escape_csv_field_normal() {
            let db = DatabaseService::new(None).expect("Failed to create database");
            let dataset_service = DatasetService::new(db.clone()).expect("Failed to create dataset service");
            let export = ExportService::new(db, dataset_service);

            assert_eq!(export.escape_csv_field("normal"), "normal");
        }

        #[test]
        fn test_escape_csv_field_with_comma() {
            let db = DatabaseService::new(None).expect("Failed to create database");
            let dataset_service = DatasetService::new(db.clone()).expect("Failed to create dataset service");
            let export = ExportService::new(db, dataset_service);

            assert_eq!(export.escape_csv_field("has,comma"), "\"has,comma\"");
        }

        #[test]
        fn test_escape_csv_field_with_quotes() {
            let db = DatabaseService::new(None).expect("Failed to create database");
            let dataset_service = DatasetService::new(db.clone()).expect("Failed to create dataset service");
            let export = ExportService::new(db, dataset_service);

            assert_eq!(export.escape_csv_field("has\"quotes"), "\"has\"\"quotes\"");
        }

        #[test]
        fn test_escape_csv_field_with_newline() {
            let db = DatabaseService::new(None).expect("Failed to create database");
            let dataset_service = DatasetService::new(db.clone()).expect("Failed to create dataset service");
            let export = ExportService::new(db, dataset_service);

            assert_eq!(export.escape_csv_field("has\nnewline"), "\"has\nnewline\"");
        }

        #[test]
        fn test_escape_csv_field_with_carriage_return() {
            let db = DatabaseService::new(None).expect("Failed to create database");
            let dataset_service = DatasetService::new(db.clone()).expect("Failed to create dataset service");
            let export = ExportService::new(db, dataset_service);

            assert_eq!(export.escape_csv_field("has\rreturn"), "\"has\rreturn\"");
        }

        #[test]
        fn test_escape_csv_field_multiple_special_chars() {
            let db = DatabaseService::new(None).expect("Failed to create database");
            let dataset_service = DatasetService::new(db.clone()).expect("Failed to create dataset service");
            let export = ExportService::new(db, dataset_service);

            assert_eq!(
                export.escape_csv_field("has\"quotes\"and,commas"),
                "\"has\"\"quotes\"\"and,commas\""
            );
        }
    }

    mod file_operations {
        use super::*;

        #[test]
        fn test_write_to_file_success() {
            let db = DatabaseService::new(None).expect("Failed to create database");
            let dataset_service = DatasetService::new(db.clone()).expect("Failed to create dataset service");
            let export = ExportService::new(db, dataset_service);

            let temp_dir = tempdir().expect("Failed to create temp directory");
            let file_path = temp_dir.path().join("test.txt");
            let file_path_str = file_path.to_str().expect("Failed to get file path");

            let content = "test content";
            let result = export.write_to_file(file_path_str, content);
            assert!(result.is_ok(), "Should write file successfully");

            let written_content = fs::read_to_string(&file_path).expect("Failed to read file");
            assert_eq!(written_content, content);
        }

        #[test]
        fn test_write_to_file_error() {
            let db = DatabaseService::new(None).expect("Failed to create database");
            let dataset_service = DatasetService::new(db.clone()).expect("Failed to create dataset service");
            let export = ExportService::new(db, dataset_service);

            let invalid_path = "/nonexistent/directory/test.txt";
            let result = export.write_to_file(invalid_path, "content");
            assert!(result.is_err(), "Should fail with invalid path");

            if let Err(error) = result {
                assert!(error.to_string().starts_with("File system error:"));
            }
        }
    }

    mod export_integration {
        use crate::services::{DatasetService, ModelService};

        use super::*;

        #[test]
        fn test_export_to_csv_success() {
            let db = DatabaseService::new(None).expect("Failed to create database");
            let dataset_service = DatasetService::new(db.clone()).expect("Failed to create dataset service");
            let export = ExportService::new(db, dataset_service);
            let _ = ModelService::new(None, export.db.clone()).expect("Failed to create model service");

            {
                let conn = export.db.conn.lock().unwrap();

                conn.execute(
                    "INSERT INTO datasets_metadata (table_name, name, description) VALUES (?, ?, ?)",
                    ["test_dataset", "Test Dataset", "Test Description"],
                )
                .expect("Failed to insert dataset metadata");

                conn.execute(
                    "INSERT INTO columns (dataset_id, table_name, name, column_type, rules, position) VALUES (?, ?, ?, ?, ?, ?)",
                    ["1", "test_dataset", "name", "TEXT", "Name column", "1"],
                ).expect("Failed to insert column");

                conn.execute(
                    "CREATE TABLE test_dataset (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        data JSON DEFAULT '{}' CHECK(json_valid(data)),
                        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                    )",
                    [],
                )
                .expect("Failed to create dataset table");

                conn.execute(
                    "INSERT INTO test_dataset (data) VALUES (?)",
                    [r#"[{"column_id": "1", "value": "John"}]"#],
                )
                .expect("Failed to insert row");
            }

            let temp_dir = tempdir().expect("Failed to create temp directory");
            let csv_path = temp_dir.path().join("test_export.csv");
            let csv_path_str = csv_path.to_str().expect("Failed to get CSV path");

            let result = export.export_to_csv(1, csv_path_str);
            assert!(result.is_ok(), "CSV export should succeed");
            assert!(csv_path.exists(), "CSV file should exist");
        }

        #[test]
        fn test_export_to_csv_invalid_dataset_id() {
            let db = DatabaseService::new(None).expect("Failed to create database");
            let dataset_service = DatasetService::new(db.clone()).expect("Failed to create dataset service");
            let export = ExportService::new(db, dataset_service);

            let temp_dir = tempdir().expect("Failed to create temp directory");
            let csv_path = temp_dir.path().join("test_export.csv");
            let csv_path_str = csv_path.to_str().expect("Failed to get CSV path");

            let result = export.export_to_csv(0, csv_path_str);
            assert!(result.is_err(), "Export should fail with invalid dataset ID");

            if let Err(error) = result {
                assert!(error.to_string().contains("Dataset ID must be a positive integer"));
            }
        }
    }
}
