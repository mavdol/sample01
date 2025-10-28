use crate::error::{AppError, AppResult};
use crate::models::SuccessResponse;
use crate::services::dataset::{Column, PaginatedResponse, Row, UpdatableColumnFields};
use crate::services::{
    DatasetMetadata, DatasetService, ExportService, GenerationService, RowGenerationProgress, RowGenerationStatus,
};
use crate::utils::detect_optimal_gpu_layers;
use std::collections::HashMap;
use tauri::{Emitter, State, Window};
use tokio_util::sync::CancellationToken;

#[tauri::command]
pub async fn create_dataset(
    name: String,
    description: String,
    dataset_service: State<'_, DatasetService>,
) -> AppResult<SuccessResponse<DatasetMetadata>> {
    let dataset = dataset_service
        .create(&name, &description)
        .map_err(|e| AppError::Io(e.to_string()))?;

    Ok(SuccessResponse::new(dataset))
}

#[tauri::command]
pub async fn list_datasets(
    dataset_service: State<'_, DatasetService>,
) -> AppResult<SuccessResponse<Vec<DatasetMetadata>>> {
    let datasets = dataset_service.find_all().map_err(|e| AppError::Io(e.to_string()))?;

    Ok(SuccessResponse::new(datasets))
}

#[tauri::command]
pub async fn update_dataset(
    id: i64,
    name: Option<String>,
    description: Option<String>,
    dataset_service: State<'_, DatasetService>,
) -> AppResult<SuccessResponse<DatasetMetadata>> {
    let dataset = dataset_service
        .update(id, name.as_deref(), description.as_deref())
        .map_err(|e| AppError::Io(e.to_string()))?;
    Ok(SuccessResponse::new(dataset))
}

#[tauri::command]
pub async fn delete_dataset(id: i64, dataset_service: State<'_, DatasetService>) -> AppResult<SuccessResponse<()>> {
    dataset_service.delete(id).map_err(|e| AppError::Io(e.to_string()))?;

    Ok(SuccessResponse::new(()))
}

#[tauri::command]
pub async fn get_columns(
    dataset_id: i64,
    dataset_service: State<'_, DatasetService>,
) -> AppResult<SuccessResponse<Vec<Column>>> {
    let columns = dataset_service
        .get_columns(dataset_id)
        .map_err(|e| AppError::Io(e.to_string()))?;
    Ok(SuccessResponse::new(columns))
}

#[tauri::command]
pub async fn create_column(
    dataset_id: i64,
    name: String,
    column_type: String,
    column_type_details: Option<String>,
    rules: String,
    dataset_service: State<'_, DatasetService>,
) -> AppResult<SuccessResponse<Vec<Column>>> {
    let dataset_metadata = dataset_service
        .find_by_id(dataset_id)
        .map_err(|e| AppError::Io(e.to_string()))?;

    let columns = dataset_service
        .add_columns(
            dataset_id,
            &[Column {
                id: None,
                table_name: dataset_metadata.table_name,
                dataset_id,
                name,
                column_type,
                column_type_details,
                rules,
                position: 0,
            }],
        )
        .map_err(|e| AppError::Io(e.to_string()))?;
    Ok(SuccessResponse::new(columns))
}

#[tauri::command]
pub async fn update_column(
    id: i64,
    name: Option<String>,
    column_type: Option<String>,
    column_type_details: Option<String>,
    rules: Option<String>,
    position: Option<String>,
    dataset_service: State<'_, DatasetService>,
) -> AppResult<SuccessResponse<Column>> {
    let column = dataset_service
        .update_column(
            id,
            UpdatableColumnFields {
                name,
                column_type,
                column_type_details,
                rules,
                position,
            },
        )
        .map_err(|e| AppError::Io(e.to_string()))?;
    Ok(SuccessResponse::new(column))
}

#[tauri::command]
pub async fn delete_column(id: i64, dataset_service: State<'_, DatasetService>) -> AppResult<SuccessResponse<()>> {
    dataset_service
        .delete_column(id)
        .map_err(|e| AppError::Io(e.to_string()))?;
    Ok(SuccessResponse::new(()))
}

#[tauri::command]
pub async fn fetch_rows(
    dataset_id: i64,
    page: i64,
    page_size: i64,
    dataset_service: State<'_, DatasetService>,
) -> AppResult<SuccessResponse<PaginatedResponse>> {
    let paginated_rows = dataset_service
        .get_rows(dataset_id, page, page_size)
        .map_err(|e| AppError::Io(e.to_string()))?;

    Ok(SuccessResponse::new(paginated_rows))
}

#[tauri::command]
pub async fn update_row(
    dataset_id: i64,
    row_id: i64,
    data: HashMap<i64, String>,
    dataset_service: State<'_, DatasetService>,
) -> AppResult<SuccessResponse<Row>> {
    let row = dataset_service
        .update_row(dataset_id, row_id, &data)
        .map_err(|e| AppError::Io(e.to_string()))?;

    Ok(SuccessResponse::new(row))
}

#[tauri::command]
pub async fn delete_row(
    dataset_id: i64,
    row_id: i64,
    dataset_service: State<'_, DatasetService>,
) -> AppResult<SuccessResponse<()>> {
    dataset_service
        .delete_row(dataset_id, row_id)
        .map_err(|e| AppError::Io(e.to_string()))?;

    Ok(SuccessResponse::new(()))
}

#[tauri::command]
pub async fn generate_rows(
    dataset_id: i64,
    model_id: i64,
    total_rows_to_generate: i64,
    gpu_layers: Option<u32>,
    window: Window,
    generation_service: State<'_, GenerationService>,
    dataset_service: State<'_, DatasetService>,
) -> AppResult<SuccessResponse<String>> {
    let gpu_layers = gpu_layers.unwrap_or_else(|| {
        let optimal = detect_optimal_gpu_layers();
        optimal
    });

    let generation_id = format!(
        "gen_{}_{}",
        dataset_id,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let generation_id_return = generation_id.clone();

    let cancel_token = CancellationToken::new();

    generation_service.register_generation(&generation_id, cancel_token.clone());

    let generation_service_clone = generation_service.inner().clone();
    let dataset_service_clone = dataset_service.inner().clone();
    let window_clone = window.clone();

    tokio::spawn(async move {
        let _ = window_clone.emit(
            "generation-status",
            RowGenerationStatus {
                generation_id: generation_id.clone(),
                status: "started".to_string(),
                message: None,
            },
        );

        let generation_service_inner = generation_service_clone.clone();
        let cancel_token_inner = cancel_token.clone();
        let generation_id_inner = generation_id.clone();
        let window_inner = window_clone.clone();
        let dataset_service_inner = dataset_service_clone.clone();

        let result = tokio::task::spawn_blocking(move || {
            generation_service_inner.generate(
                dataset_id,
                model_id,
                total_rows_to_generate,
                gpu_layers,
                cancel_token_inner,
                move |last_row_generated, total_rows_generated, total_rows_to_generate| {
                    let row = match dataset_service_inner.add_row(dataset_id, &last_row_generated) {
                        Ok(row) => row,
                        Err(e) => {
                            let _ = window_inner.emit(
                                "generation-status",
                                RowGenerationStatus {
                                    generation_id: generation_id_inner.clone(),
                                    status: "failed".to_string(),
                                    message: Some(e.to_string()),
                                },
                            );
                            return;
                        }
                    };

                    let _ = window_inner.emit(
                        "generation-progress",
                        RowGenerationProgress {
                            dataset_id,
                            generation_id: generation_id_inner.clone(),
                            last_row_generated: row,
                            total_rows_generated,
                            total_rows_to_generate,
                            status: "generating".to_string(),
                        },
                    );
                },
            )
        })
        .await;

        match result {
            Ok(Ok(())) => {
                let _ = window_clone.emit(
                    "generation-status",
                    RowGenerationStatus {
                        generation_id: generation_id.clone(),
                        status: "completed".to_string(),
                        message: Some("All rows generated successfully".to_string()),
                    },
                );
            }
            Ok(Err(e)) => {
                let status = if e.to_string().contains("cancelled") {
                    "cancelled"
                } else {
                    "failed"
                };
                let _ = window_clone.emit(
                    "generation-status",
                    RowGenerationStatus {
                        generation_id: generation_id.clone(),
                        status: status.to_string(),
                        message: Some(e.to_string()),
                    },
                );
            }
            Err(e) => {
                let _ = window_clone.emit(
                    "generation-status",
                    RowGenerationStatus {
                        generation_id: generation_id.clone(),
                        status: "failed".to_string(),
                        message: Some(format!("Task panicked: {}", e)),
                    },
                );
            }
        }

        generation_service_clone.unregister_generation(&generation_id);
    });

    Ok(SuccessResponse::new(generation_id_return))
}

#[tauri::command]
pub fn cancel_generation(
    generation_id: String,
    generation_service: State<'_, GenerationService>,
) -> AppResult<SuccessResponse<String>> {
    generation_service
        .cancel_generation(&generation_id)
        .map_err(|e| AppError::Io(e.to_string()))?;

    Ok(SuccessResponse::new("Generation cancelled".to_string()))
}

#[tauri::command]
#[allow(dead_code)]
pub fn get_optimal_gpu_layers() -> AppResult<SuccessResponse<u32>> {
    let optimal = detect_optimal_gpu_layers();
    Ok(SuccessResponse::new(optimal))
}

#[tauri::command]
pub fn export_to_csv(
    dataset_id: i64,
    file_path: String,
    export_service: State<'_, ExportService>,
) -> AppResult<SuccessResponse<String>> {
    export_service
        .export_to_csv(dataset_id, &file_path)
        .map_err(|e| AppError::Io(e.to_string()))?;
    Ok(SuccessResponse::new("Dataset exported".to_string()))
}
