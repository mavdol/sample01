use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tauri::{AppHandle, Manager};

use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

use crate::error::AppError;
use crate::services::{DatabaseError, DatabaseService};
use rusqlite::Result as SqliteResult;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub download_id: String,
    pub progress: f64,
    pub status: String,
}

#[derive(Debug)]
pub enum ModelError {
    DatabaseError(String),
    HttpError(String),
    FsError(String),
    Cancelled(String),
    NotFound(String),
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModelError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            ModelError::HttpError(msg) => write!(f, "HTTP error: {}", msg),
            ModelError::FsError(msg) => write!(f, "File system error: {}", msg),
            ModelError::Cancelled(msg) => write!(f, "Download cancelled: {}", msg),
            ModelError::NotFound(msg) => write!(f, "Not found: {}", msg),
        }
    }
}

impl std::error::Error for ModelError {}

impl From<rusqlite::Error> for ModelError {
    fn from(err: rusqlite::Error) -> Self {
        ModelError::DatabaseError(err.to_string())
    }
}

impl From<std::io::Error> for ModelError {
    fn from(err: std::io::Error) -> Self {
        ModelError::FsError(err.to_string())
    }
}

impl From<reqwest::Error> for ModelError {
    fn from(err: reqwest::Error) -> Self {
        ModelError::HttpError(err.to_string())
    }
}

impl From<DatabaseError> for ModelError {
    fn from(err: DatabaseError) -> Self {
        ModelError::DatabaseError(err.to_string())
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub id: Option<i64>,
    pub filename: String,
    pub quantization: Option<String>,
    pub label: String,
    pub size: u64,
    pub model_type: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone)]
pub struct ModelService {
    pub db: DatabaseService,
    pub client: Client,
    pub models_dir: PathBuf,
    active_downloads: Arc<Mutex<HashMap<String, CancellationToken>>>,
}

impl ModelService {
    pub fn new(app: Option<&AppHandle>, db: DatabaseService) -> Result<Self, AppError> {
        let client = Client::new();
        let mut model = Self {
            db,
            client,
            models_dir: PathBuf::new(),
            active_downloads: Arc::new(Mutex::new(HashMap::new())),
        };

        model
            .create_models_default_table()
            .map_err(|e| AppError::Io(e.to_string()))?;

        if let Some(app) = app {
            let app_data_dir = app.path().app_data_dir().map_err(|e| AppError::Io(e.to_string()))?;
            let models_dir = app_data_dir.join("models");

            std::fs::create_dir_all(&models_dir)
                .map_err(|e| AppError::Io(format!("Failed to create models directory: {}", e)))?;

            model.models_dir = models_dir.clone();
            model
                .check_model_files_integrity(&model.db, models_dir)
                .map_err(|e| AppError::Io(e.to_string()))?;
        }

        Ok(model)
    }

    pub fn create_models_default_table(&self) -> SqliteResult<(), DatabaseError> {
        let conn = self
            .db
            .conn
            .lock()
            .map_err(|_| DatabaseError::SqliteError("Failed to acquire mutex lock".to_string()))?;

        conn.execute(
            "
            CREATE TABLE IF NOT EXISTS models (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                filename TEXT NOT NULL,
                quantization TEXT,
                label TEXT NOT NULL,
                model_type TEXT NOT NULL,
                size INTEGER NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
        ",
            [],
        )?;

        Ok(())
    }

    pub fn get_model_info(&self, id: i64) -> Result<ModelInfo, ModelError> {
        let model = self.db.query("SELECT id, filename, quantization, label, model_type, size, created_at, updated_at FROM models WHERE id = ?", [id], |row| {
            Ok(ModelInfo {
                id: row.get::<_, Option<i64>>(0)?,
                filename: row.get::<_, String>(1)?,
                quantization: row.get::<_, Option<String>>(2)?,
                label: row.get::<_, String>(3)?,
                model_type: row.get::<_, String>(4)?,
                size: row.get::<_, u64>(5)?,
                created_at: row.get::<_, String>(6)?,
                updated_at: row.get::<_, String>(7)?,
            })
        })?.into_iter().next().ok_or(ModelError::DatabaseError("Model not found".to_string()))?;

        Ok(model)
    }

    pub fn list_models(&self) -> Result<Vec<ModelInfo>, ModelError> {
        let models = self.db.query(
            "SELECT id, filename, quantization, label, model_type, size, created_at, updated_at FROM models",
            [],
            |row| {
                Ok(ModelInfo {
                    id: row.get::<_, Option<i64>>(0)?,
                    filename: row.get::<_, String>(1)?,
                    quantization: row.get::<_, Option<String>>(2)?,
                    label: row.get::<_, String>(3)?,
                    model_type: row.get::<_, String>(4)?,
                    size: row.get::<_, u64>(5)?,
                    created_at: row.get::<_, String>(6)?,
                    updated_at: row.get::<_, String>(7)?,
                })
            },
        )?;

        Ok(models)
    }

    pub async fn download_model(
        &self,
        models_dir: &PathBuf,
        filename: &str,
        quantization: &str,
        label: &str,
        model_type: &str,
        model_url: &str,
        cancel_token: CancellationToken,
        progress_callback: impl Fn(f64),
    ) -> Result<(), ModelError> {
        let model_path = models_dir.join(filename);

        let mut downloaded: u64 = 0;
        let file_exists = model_path.exists();

        if file_exists {
            let metadata = std::fs::metadata(&model_path)?;
            downloaded = metadata.len();
        }

        let mut request = self.client.get(model_url);
        if downloaded > 0 {
            request = request.header("Range", format!("bytes={}-", downloaded));
        }

        let response = request.send().await?;

        if !response.status().is_success() && response.status() != reqwest::StatusCode::PARTIAL_CONTENT {
            return Err(ModelError::HttpError(format!("HTTP error: {}", response.status())));
        }

        let mut file = if file_exists {
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&model_path)?
        } else {
            File::create(&model_path)?
        };

        let total_size = response.content_length().unwrap_or(0) + downloaded;
        let mut stream = response.bytes_stream();

        let mut last_progress_reported = 0.0;
        let mut last_progress_time = Instant::now();
        let progress_threshold = 1.0;
        let time_threshold = Duration::from_millis(100);

        while let Some(chunk) = stream.next().await {
            if cancel_token.is_cancelled() {
                return Err(ModelError::Cancelled("Download was cancelled".to_string()));
            }

            let chunk = chunk.map_err(|e| ModelError::FsError(e.to_string()))?;

            file.write_all(&chunk)?;

            downloaded += chunk.len() as u64;
            if total_size > 0 {
                let progress = (downloaded as f64 / total_size as f64) * 100.0;
                let time_elapsed = last_progress_time.elapsed();

                if (progress - last_progress_reported).abs() >= progress_threshold || time_elapsed >= time_threshold {
                    progress_callback(progress);
                    last_progress_reported = progress;
                    last_progress_time = Instant::now();
                }
            }
        }

        if total_size > 0 {
            progress_callback(100.0);
        }

        let file_size = std::fs::metadata(&model_path)?.len();

        let conn = self.db.conn.lock().unwrap();
        let existing_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM models WHERE filename = ?", [filename], |row| {
                row.get(0)
            })
            .unwrap_or(0);

        if existing_count == 0 {
            drop(conn);
            self.db
                .execute(
                    "INSERT INTO models (filename, quantization, label, model_type, size) VALUES (?, ?, ?, ?, ?)",
                    [filename, quantization, label, model_type, &file_size.to_string()],
                )
                .map_err(|e| ModelError::DatabaseError(e.to_string()))?;
        }

        Ok(())
    }

    pub fn register_download(&self, filename: &str, quantization: &str, cancel_token: CancellationToken) {
        let download_id = format!("{}_{}", filename, quantization);
        let mut active_downloads = self.active_downloads.lock().unwrap();
        active_downloads.insert(download_id, cancel_token);
    }

    pub fn unregister_download(&self, filename: &str, quantization: &str) {
        let download_id = format!("{}_{}", filename, quantization);
        let mut active_downloads = self.active_downloads.lock().unwrap();
        active_downloads.remove(&download_id);
    }

    pub fn cancel_download(&self, models_dir: &PathBuf, filename: &str, quantization: &str) -> Result<(), ModelError> {
        let download_id = format!("{}_{}", filename, quantization);

        let cancel_token = {
            let active_downloads = self.active_downloads.lock().unwrap();
            active_downloads.get(&download_id).cloned()
        };

        if let Some(token) = cancel_token {
            token.cancel();

            let model_path = models_dir.join(filename);
            if model_path.exists() {
                std::fs::remove_file(&model_path)?;
            }

            Ok(())
        } else {
            Err(ModelError::NotFound(format!(
                "No active download found for: {}",
                download_id
            )))
        }
    }

    pub fn delete_model_file(&self, model_path: &PathBuf, filename: String) -> Result<(), ModelError> {
        let model = self.db.query(
            "SELECT filename FROM models WHERE filename = ?",
            [&filename.to_string()],
            |row| Ok(row.get::<_, String>(0)?),
        )?;

        if model.is_empty() {
            return Err(ModelError::DatabaseError(format!(
                "Model with filename {} not found",
                filename
            )));
        }

        self.db
            .execute("DELETE FROM models WHERE filename = ?", [&filename.to_string()])?;

        std::fs::remove_file(model_path)?;

        Ok(())
    }

    pub fn check_model_files_integrity(&self, db: &DatabaseService, models_dir: PathBuf) -> Result<(), ModelError> {
        let models = db.query(
            "SELECT id, filename, quantization, label, size, model_type, created_at, updated_at FROM models",
            [],
            |row| {
                Ok(ModelInfo {
                    id: row.get::<_, Option<i64>>(0)?,
                    filename: row.get::<_, String>(1)?,
                    quantization: row.get::<_, Option<String>>(2)?,
                    label: row.get::<_, String>(3)?,
                    size: row.get::<_, u64>(4)?,
                    model_type: row.get::<_, String>(5)?,
                    created_at: row.get::<_, String>(6)?,
                    updated_at: row.get::<_, String>(7)?,
                })
            },
        )?;

        let mut existing_files = std::collections::HashSet::new();

        let files = std::fs::read_dir(models_dir)?;

        for file in files {
            let path = file?.path();

            if path.is_file() {
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    existing_files.insert(filename.to_string());
                }
            }
        }

        let mut models_to_delete: Vec<i64> = Vec::new();

        for model in models {
            if let Some(id) = model.id {
                if !existing_files.contains(&model.filename) {
                    models_to_delete.push(id);
                }
            }
        }

        if !models_to_delete.is_empty() {
            let delete_query = "DELETE FROM models WHERE id = ?";
            let params_list: Vec<_> = models_to_delete.iter().map(|&id| (id,)).collect();

            db.execute_batch(delete_query, &params_list)
                .map_err(|e| ModelError::DatabaseError(e.to_string()))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::Arc;
    use tokio;
    use tokio::time::{sleep, Duration};
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    mod creation {
        use super::*;

        #[test]
        fn test_new_model_service() {
            let db = DatabaseService::new(None).expect("Failed to create database");

            let model = ModelService::new(None, db.clone());
            assert!(model.is_ok(), "Failed to create model service");
        }

        #[test]
        fn test_create_models_default_table() {
            let db = DatabaseService::new(None).expect("Failed to create database");
            let model = ModelService::new(None, db.clone()).expect("Failed to create model service");

            {
                let conn = db.conn.lock().unwrap();
                conn.execute("DROP TABLE IF EXISTS models", [])
                    .expect("Failed to delete models table");
                conn.execute("DROP TABLE IF EXISTS columns", [])
                    .expect("Failed to delete columns table");
                conn.execute("DROP TABLE IF EXISTS datasets_metadata", [])
                    .expect("Failed to delete datasets_metadata table");
            }

            model
                .create_models_default_table()
                .expect("Failed to create models table");

            let conn = db.conn.lock().unwrap();

            let mut models_stmt = conn
                .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='models'")
                .expect("Failed to prepare query");

            let models_exists: bool = models_stmt.exists([]).expect("Failed to check if table exists");

            assert!(models_exists, "models table was not created");
        }
    }

    mod file_operations {
        use super::*;

        #[test]
        fn test_model_get_model_info() {
            let db = DatabaseService::new(None).expect("Failed to create database");
            let _ = ModelService::new(None, db.clone()).expect("Failed to create model service");

            {
                let conn = db.conn.lock().unwrap();

                conn.execute(
                    "INSERT INTO models (filename, quantization, label, model_type, size) VALUES (?, ?, ?, ?, ?)",
                    ["model1.gguf", "Q4_K_M", "Test Model 1", "llm", "1000"],
                )
                .expect("Failed to insert model1");

                conn.execute(
                    "INSERT INTO models (filename, quantization, label, model_type, size) VALUES (?, ?, ?, ?, ?)",
                    ["missing_model.gguf", "Q5_K_M", "Missing Model", "llm", "2000"],
                )
                .expect("Failed to insert model2");
            }

            let model_service = ModelService::new(None, db.clone()).expect("Failed to create model service");
            let model_info = model_service.get_model_info(1).expect("Failed to get model info");
            assert_eq!(model_info.filename, "model1.gguf");
            assert_eq!(model_info.quantization, Some("Q4_K_M".to_string()));
            assert_eq!(model_info.label, "Test Model 1");
        }

        #[test]
        fn test_model_check_files_integrity() {
            let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
            let models_path = temp_dir.path().to_path_buf();

            let db = DatabaseService::new(None).expect("Failed to create database");
            let _ = ModelService::new(None, db.clone()).expect("Failed to create model service");

            {
                let conn = db.conn.lock().unwrap();

                conn.execute(
                    "INSERT INTO models (filename, quantization, label, model_type, size) VALUES (?, ?, ?, ?, ?)",
                    ["model1.gguf", "Q4_K_M", "Test Model 1", "llm", "1000"],
                )
                .expect("Failed to insert model1");

                conn.execute(
                    "INSERT INTO models (filename, quantization, label, model_type, size) VALUES (?, ?, ?, ?, ?)",
                    ["missing_model.gguf", "Q5_K_M", "Missing Model", "llm", "2000"],
                )
                .expect("Failed to insert model2");
            }

            let model1_path = models_path.join("model1.gguf");
            let mut file = File::create(&model1_path).expect("Failed to create test file");
            file.write_all(b"fake model content")
                .expect("Failed to write to test file");

            let model_service = ModelService::new(None, db.clone()).expect("Failed to create model service");

            let result = model_service.check_model_files_integrity(&db, models_path);
            assert!(result.is_ok(), "Integrity check failed: {:?}", result.err());

            let conn = db.conn.lock().unwrap();

            let mut models_stmt = conn.prepare("SELECT * FROM models").expect("Failed to prepare query");

            let models = models_stmt
                .query_map([], |row| {
                    Ok(ModelInfo {
                        id: row.get::<_, Option<i64>>(0)?,
                        filename: row.get::<_, String>(1)?,
                        quantization: row.get::<_, Option<String>>(2)?,
                        label: row.get::<_, String>(3)?,
                        model_type: row.get::<_, String>(4)?,
                        size: row.get::<_, u64>(5)?,
                        created_at: row.get::<_, String>(6)?,
                        updated_at: row.get::<_, String>(7)?,
                    })
                })
                .expect("Failed to query columns")
                .collect::<Result<Vec<_>, _>>()
                .expect("Failed to collect models");

            assert_eq!(models.len(), 1, "Should have 1 model remaining");
            assert_eq!(models[0].filename, "model1.gguf", "Wrong model remained");
        }

        #[test]
        fn test_model_list_models() {
            let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
            let models_path = temp_dir.path().to_path_buf();

            let db = DatabaseService::new(None).expect("Failed to create database");
            let _ = ModelService::new(None, db.clone()).expect("Failed to create model service");

            {
                let conn = db.conn.lock().unwrap();

                conn.execute(
                    "INSERT INTO models (filename, quantization, label, model_type, size) VALUES (?, ?, ?, ?, ?)",
                    ["model1.gguf", "Q4_K_M", "Test Model 1", "llm", "1000"],
                )
                .expect("Failed to insert model1");

                conn.execute(
                    "INSERT INTO models (filename, quantization, label, model_type, size) VALUES (?, ?, ?, ?, ?)",
                    ["model2.gguf", "Q5_K_M", "Missing Model", "llm", "2000"],
                )
                .expect("Failed to insert model2");
            }

            let model1_path = models_path.join("model1.gguf");
            let model2_path = models_path.join("model2.gguf");
            let mut file = File::create(&model1_path).expect("Failed to create test file");
            let mut file2 = File::create(&model2_path).expect("Failed to create test file");
            file.write_all(b"fake model content")
                .expect("Failed to write to test file");
            file2
                .write_all(b"fake model content")
                .expect("Failed to write to test file");

            let model_service = ModelService::new(None, db.clone()).expect("Failed to create model service");

            let models = model_service.list_models().expect("Failed to list models");

            assert_eq!(models.len(), 2, "Should have 2 models");
            assert_eq!(models[0].filename, "model1.gguf", "Wrong model");
            assert_eq!(models[1].filename, "model2.gguf", "Wrong model");
        }

        #[test]
        fn test_model_delete_model_file() {
            let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
            let models_path = temp_dir.path().to_path_buf();

            let db = DatabaseService::new(None).expect("Failed to create database");
            let _ = ModelService::new(None, db.clone()).expect("Failed to create model service");

            {
                let conn = db.conn.lock().unwrap();

                conn.execute(
                    "INSERT INTO models (filename, quantization, label, model_type, size) VALUES (?, ?, ?, ?, ?)",
                    ["model1.gguf", "Q4_K_M", "Test Model 1", "llm", "1000"],
                )
                .expect("Failed to insert model1");

                conn.execute(
                    "INSERT INTO models (filename, quantization, label, model_type, size) VALUES (?, ?, ?, ?, ?)",
                    ["model2.gguf", "Q5_K_M", "Missing Model", "llm", "2000"],
                )
                .expect("Failed to insert model2");
            }

            let model1_path = models_path.join("model1.gguf");
            let model2_path = models_path.join("model2.gguf");
            let mut file = File::create(&model1_path).expect("Failed to create test file");
            let mut file2 = File::create(&model2_path).expect("Failed to create test file");
            file.write_all(b"fake model content")
                .expect("Failed to write to test file");
            file2
                .write_all(b"fake model content")
                .expect("Failed to write to test file");

            let model_service = ModelService::new(None, db.clone()).expect("Failed to create model service");

            model_service
                .delete_model_file(&model1_path, "model1.gguf".to_string())
                .expect("Failed to delete model file");

            let conn = db.conn.lock().unwrap();

            let mut models_stmt = conn.prepare("SELECT * FROM models").expect("Failed to prepare query");

            let models = models_stmt
                .query_map([], |row| {
                    Ok(ModelInfo {
                        id: row.get::<_, Option<i64>>(0)?,
                        filename: row.get::<_, String>(1)?,
                        quantization: row.get::<_, Option<String>>(2)?,
                        label: row.get::<_, String>(3)?,
                        model_type: row.get::<_, String>(4)?,
                        size: row.get::<_, u64>(5)?,
                        created_at: row.get::<_, String>(6)?,
                        updated_at: row.get::<_, String>(7)?,
                    })
                })
                .expect("Failed to query columns")
                .collect::<Result<Vec<_>, _>>()
                .expect("Failed to collect models");

            let model1_exists = std::fs::exists(&model1_path).expect("Failed to check if model1 file exists");

            assert!(!model1_exists, "Model1 file still exists");
            assert_eq!(models.len(), 1, "Should have only 1 models");
        }
    }

    mod download {
        use super::*;

        #[tokio::test]
        async fn test_download_model_success() {
            let mock_server = MockServer::start().await;

            let test_content = b"fake model content for testing";
            let test_filename = "test_model.gguf";
            let test_quantization = "Q4_K_M";
            let test_label = "Test Model";
            let test_model_type = "llm";

            let content_length = test_content.len().to_string();
            Mock::given(method("GET"))
                .and(path("/download"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_raw(test_content, "application/octet-stream")
                        .insert_header("content-length", content_length.as_str()),
                )
                .mount(&mock_server)
                .await;

            let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
            let models_path = temp_dir.path().to_path_buf();

            let db = DatabaseService::new(None).expect("Failed to create database");

            let model_service = ModelService::new(None, db.clone()).expect("Failed to create model service");

            let progress_calls = Arc::new(AtomicUsize::new(0));
            let progress_calls_clone = progress_calls.clone();

            let cancel_token = CancellationToken::new();

            let result = model_service
                .download_model(
                    &models_path,
                    test_filename,
                    test_quantization,
                    test_label,
                    test_model_type,
                    &format!("{}/download", mock_server.uri()),
                    cancel_token,
                    move |progress| {
                        progress_calls_clone.fetch_add(1, Ordering::Relaxed);
                        assert!(
                            progress >= 0.0 && progress <= 100.0,
                            "Progress should be between 0 and 100"
                        );
                    },
                )
                .await;

            assert!(result.is_ok(), "Download should succeed: {:?}", result.err());

            let model_path = models_path.join(test_filename);
            assert!(model_path.exists(), "Model file should exist");

            let file_content = std::fs::read(&model_path).expect("Failed to read model file");
            assert_eq!(
                file_content, test_content,
                "File content should match downloaded content"
            );

            let conn = db.conn.lock().unwrap();
            let mut stmt = conn.prepare("SELECT id, filename, quantization, label, model_type, size, created_at, updated_at FROM models WHERE filename = ?")
                .expect("Failed to prepare query");

            let model_info: Result<ModelInfo, _> = stmt.query_row([test_filename], |row| {
                Ok(ModelInfo {
                    id: row.get::<_, Option<i64>>(0)?,
                    filename: row.get::<_, String>(1)?,
                    quantization: row.get::<_, Option<String>>(2)?,
                    label: row.get::<_, String>(3)?,
                    model_type: row.get::<_, String>(4)?,
                    size: row.get::<_, u64>(5)?,
                    created_at: row.get::<_, String>(6)?,
                    updated_at: row.get::<_, String>(7)?,
                })
            });

            assert!(
                model_info.is_ok(),
                "Model should be in database: {:?}",
                model_info.err()
            );
            let model_info = model_info.unwrap();
            assert_eq!(model_info.filename, test_filename);
            assert_eq!(model_info.quantization, Some(test_quantization.to_string()));
            assert_eq!(model_info.label, test_label);
            assert_eq!(model_info.model_type, test_model_type);
            assert_eq!(model_info.size, test_content.len() as u64);

            assert!(
                progress_calls.load(Ordering::Relaxed) > 0,
                "Progress callback should be called"
            );
        }

        #[tokio::test]
        async fn test_download_model_network_error() {
            let mock_server = MockServer::start().await;

            Mock::given(method("GET"))
                .and(path("/download"))
                .respond_with(ResponseTemplate::new(404))
                .mount(&mock_server)
                .await;

            let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
            let models_path = temp_dir.path().to_path_buf();

            let db = DatabaseService::new(None).expect("Failed to create database");
            let model_service = ModelService::new(None, db.clone()).expect("Failed to create model service");

            let cancel_token = CancellationToken::new();

            let result = model_service
                .download_model(
                    &models_path,
                    "test_model.gguf",
                    "Q4_K_M",
                    "Test Model",
                    "llm",
                    &format!("{}/download", mock_server.uri()),
                    cancel_token,
                    |_| {},
                )
                .await;

            assert!(result.is_err(), "Download should fail with 404");

            let model_path = models_path.join("test_model.gguf");
            assert!(!model_path.exists(), "Model file should not exist");

            let conn = db.conn.lock().unwrap();
            let mut stmt = conn
                .prepare("SELECT COUNT(*) FROM models WHERE filename = ?")
                .expect("Failed to prepare query");

            let count: i64 = stmt
                .query_row(["test_model.gguf"], |row| row.get(0))
                .expect("Failed to query count");

            assert_eq!(count, 0, "No model should be in database");
        }

        #[tokio::test]
        async fn test_download_model_file_creation_error() {
            let mock_server = MockServer::start().await;

            let test_content = b"test content";

            Mock::given(method("GET"))
                .and(path("/download"))
                .respond_with(ResponseTemplate::new(200).set_body_raw(test_content, "application/octet-stream"))
                .mount(&mock_server)
                .await;

            let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
            let models_path = temp_dir.path().join("nonexistent").join("models");

            let db = DatabaseService::new(None).expect("Failed to create database");
            let model_service = ModelService::new(None, db.clone()).expect("Failed to create model service");

            let cancel_token = CancellationToken::new();

            let result = model_service
                .download_model(
                    &models_path,
                    "test_model.gguf",
                    "Q4_K_M",
                    "Test Model",
                    "llm",
                    &format!("{}/download", mock_server.uri()),
                    cancel_token,
                    |_| {},
                )
                .await;

            assert!(result.is_err(), "Download should fail due to file creation error");

            let error_msg = result.unwrap_err().to_string();
            assert!(
                error_msg.starts_with("File system error:"),
                "Error should mention file creation failure"
            );
        }

        #[tokio::test]
        async fn test_download_model_progress_callback() {
            let mock_server = MockServer::start().await;

            let test_content = vec![0u8; 1000];

            let content_length = test_content.len().to_string();
            Mock::given(method("GET"))
                .and(path("/download"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_raw(test_content.clone(), "application/octet-stream")
                        .insert_header("content-length", content_length.as_str()),
                )
                .mount(&mock_server)
                .await;

            let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
            let models_path = temp_dir.path().to_path_buf();

            let db = DatabaseService::new(None).expect("Failed to create database");
            let model_service = ModelService::new(None, db.clone()).expect("Failed to create model service");

            let progress_values = Arc::new(std::sync::Mutex::new(Vec::new()));
            let progress_values_clone = progress_values.clone();

            let cancel_token = CancellationToken::new();

            let result = model_service
                .download_model(
                    &models_path,
                    "test_model.gguf",
                    "Q4_K_M",
                    "Test Model",
                    "llm",
                    &format!("{}/download", mock_server.uri()),
                    cancel_token,
                    move |progress| {
                        progress_values_clone.lock().unwrap().push(progress);
                    },
                )
                .await;

            assert!(result.is_ok(), "Download should succeed");

            let values = progress_values.lock().unwrap();
            assert!(!values.is_empty(), "Progress callback should be called");

            for i in 1..values.len() {
                assert!(values[i] >= values[i - 1], "Progress should be non-decreasing");
            }

            if let Some(&final_progress) = values.last() {
                assert!(
                    (final_progress - 100.0).abs() < 0.1,
                    "Final progress should be close to 100%"
                );
            }
        }

        #[tokio::test]
        async fn test_cancel_download() {
            let mock_server = MockServer::start().await;

            let test_content = vec![0u8; 50_000_000]; // 50 MB to ensure streaming takes time and cancellation can be tested
            let test_filename = "test_model.gguf";
            let test_quantization = "Q4_K_M";

            let content_length = test_content.len().to_string();
            Mock::given(method("GET"))
                .and(path("/download"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_raw(test_content.clone(), "application/octet-stream")
                        .insert_header("content-length", content_length.as_str()),
                )
                .mount(&mock_server)
                .await;

            let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
            let models_path = temp_dir.path().to_path_buf();

            let db = DatabaseService::new(None).expect("Failed to create database");
            let model_service = ModelService::new(None, db.clone()).expect("Failed to create model service");

            let model_service_clone = model_service.clone();
            let models_path_clone = models_path.clone();
            let model_path = models_path.join(test_filename);

            let cancel_token = CancellationToken::new();
            let cancel_token_clone = cancel_token.clone();

            model_service.register_download(test_filename, test_quantization, cancel_token.clone());

            let download_started = Arc::new(AtomicBool::new(false));
            let download_started_clone = download_started.clone();

            let download_handle = tokio::spawn(async move {
                let result = model_service_clone
                    .download_model(
                        &models_path_clone,
                        test_filename,
                        test_quantization,
                        "Test Model",
                        "llm",
                        &format!("{}/download", mock_server.uri()),
                        cancel_token_clone,
                        move |_| {
                            download_started_clone.store(true, Ordering::Relaxed);
                        },
                    )
                    .await;

                model_service_clone.unregister_download(test_filename, test_quantization);
                result
            });

            let mut attempts = 0;
            while !download_started.load(Ordering::Relaxed) && attempts < 100 {
                sleep(Duration::from_millis(10)).await;
                attempts += 1;
            }

            if !download_started.load(Ordering::Relaxed) {
                let _ = download_handle.await;
                return;
            }

            let _cancel_result = model_service.cancel_download(&models_path, test_filename, test_quantization);
            let download_result = download_handle.await.expect("Task should complete");

            assert!(download_result.is_err(), "Download should be cancelled");
            let error_msg = download_result.unwrap_err().to_string();
            assert!(
                error_msg.contains("cancelled") || error_msg.contains("Cancelled"),
                "Error should mention cancellation: {}",
                error_msg
            );

            assert!(!model_path.exists(), "Partial file should be deleted");
        }

        #[tokio::test]
        async fn test_cancel_download_not_found() {
            let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
            let models_path = temp_dir.path().to_path_buf();

            let db = DatabaseService::new(None).expect("Failed to create database");
            let model_service = ModelService::new(None, db.clone()).expect("Failed to create model service");

            let result = model_service.cancel_download(&models_path, "nonexistent.gguf", "Q4_K_M");

            assert!(result.is_err(), "Should fail for non-existent download");
            let error_msg = result.unwrap_err().to_string();
            assert!(
                error_msg.contains("Not found") || error_msg.contains("not found"),
                "Error should mention not found: {}",
                error_msg
            );
        }

        #[tokio::test]
        async fn test_download_resume() {
            let mock_server = MockServer::start().await;

            let full_content = b"This is the full file content for resume testing";
            let partial_size = 20;
            let remaining_content = &full_content[partial_size..];

            let test_filename = "test_resume.gguf";
            let test_quantization = "Q4_K_M";

            let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
            let models_path = temp_dir.path().to_path_buf();
            let model_path = models_path.join(test_filename);

            let mut partial_file = File::create(&model_path).expect("Failed to create partial file");
            partial_file
                .write_all(&full_content[..partial_size])
                .expect("Failed to write partial content");
            drop(partial_file);

            Mock::given(method("GET"))
                .and(path("/download"))
                .and(header("Range", format!("bytes={}-", partial_size).as_str()))
                .respond_with(
                    ResponseTemplate::new(206)
                        .set_body_raw(remaining_content, "application/octet-stream")
                        .insert_header("content-length", remaining_content.len().to_string().as_str()),
                )
                .mount(&mock_server)
                .await;

            let db = DatabaseService::new(None).expect("Failed to create database");
            let model_service = ModelService::new(None, db.clone()).expect("Failed to create model service");

            let cancel_token = CancellationToken::new();

            let result = model_service
                .download_model(
                    &models_path,
                    test_filename,
                    test_quantization,
                    "Test Resume Model",
                    "llm",
                    &format!("{}/download", mock_server.uri()),
                    cancel_token,
                    |_| {},
                )
                .await;

            assert!(result.is_ok(), "Resume download should succeed: {:?}", result.err());

            let file_content = std::fs::read(&model_path).expect("Failed to read resumed file");
            assert_eq!(file_content, full_content, "Resumed file should have complete content");

            let conn = db.conn.lock().unwrap();
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM models WHERE filename = ?",
                    [test_filename],
                    |row| row.get(0),
                )
                .expect("Failed to query count");

            assert_eq!(count, 1, "Model should be in database");
        }
    }
}
