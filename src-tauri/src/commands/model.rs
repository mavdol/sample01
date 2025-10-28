use crate::error::{AppError, AppResult};
use crate::models::SuccessResponse;
use crate::services::model::{DownloadProgress, ModelInfo};
use crate::services::ModelService;
use crate::utils::detect_optimal_gpu_layers;

use tauri::{AppHandle, Emitter, Manager, State, Window};
use tokio_util::sync::CancellationToken;

#[tauri::command]
pub async fn download_model(
    app_handle: AppHandle,
    window: Window,
    model_url: String,
    filename: String,
    quantization: String,
    label: String,
    model_type: String,
    model_service: State<'_, ModelService>,
) -> AppResult<SuccessResponse<String>> {
    let models_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Io(e.to_string()))?
        .join("models");

    let download_id = format!("{}_{}", filename, quantization);
    let download_id_return = download_id.clone();

    let model_service_clone = model_service.inner().clone();
    let cancel_token = CancellationToken::new();

    model_service.register_download(&filename, &quantization, cancel_token.clone());

    tokio::spawn(async move {
        let _ = window.emit(
            "download-progress",
            DownloadProgress {
                download_id: download_id.clone(),
                progress: 0.0,
                status: "downloading".to_string(),
            },
        );

        let result = model_service_clone
            .download_model(
                &models_dir,
                &filename,
                &quantization,
                &label,
                &model_type,
                &model_url,
                cancel_token,
                |progress: f64| {
                    let _ = window.emit(
                        "download-progress",
                        DownloadProgress {
                            download_id: download_id.clone(),
                            progress,
                            status: "downloading".to_string(),
                        },
                    );
                },
            )
            .await;

        model_service_clone.unregister_download(&filename, &quantization);

        match result {
            Ok(_) => {
                let _ = window.emit(
                    "download-progress",
                    DownloadProgress {
                        download_id: download_id.clone(),
                        progress: 100.0,
                        status: "completed".to_string(),
                    },
                );
            }
            Err(e) => {
                let status = if e.to_string().contains("cancelled") {
                    "cancelled"
                } else {
                    "failed"
                };
                let _ = window.emit(
                    "download-progress",
                    DownloadProgress {
                        download_id: download_id.clone(),
                        progress: 0.0,
                        status: status.to_string(),
                    },
                );
            }
        }
    });

    Ok(SuccessResponse::new(download_id_return))
}

#[tauri::command]
pub fn cancel_download(
    app_handle: AppHandle,
    filename: String,
    quantization: String,
    model_service: State<'_, ModelService>,
) -> AppResult<SuccessResponse<String>> {
    let models_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Io(e.to_string()))?
        .join("models");

    model_service
        .cancel_download(&models_dir, &filename, &quantization)
        .map_err(|e| AppError::Io(e.to_string()))?;

    Ok(SuccessResponse::new("Download cancelled".to_string()))
}

#[tauri::command]
pub fn list_models(model_service: State<'_, ModelService>) -> AppResult<SuccessResponse<Vec<ModelInfo>>> {
    let models = model_service.list_models().map_err(|e| AppError::Io(e.to_string()))?;

    Ok(SuccessResponse::new(models))
}

#[tauri::command]
pub fn delete_model(
    app_handle: AppHandle,
    filename: String,
    model_service: State<'_, ModelService>,
) -> AppResult<SuccessResponse<String>> {
    let models_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Io(e.to_string()))?
        .join("models");

    let model_path = models_dir.join(&filename);

    model_service
        .delete_model_file(&model_path, filename)
        .map_err(|e| AppError::Io(e.to_string()))?;

    Ok(SuccessResponse::new("Model deleted".to_string()))
}

#[tauri::command]
pub fn get_default_gpu_layers() -> AppResult<SuccessResponse<u32>> {
    let default = detect_optimal_gpu_layers();
    Ok(SuccessResponse::new(default))
}
