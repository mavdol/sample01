mod commands;
mod config;
mod error;
mod models;
pub mod services;
mod utils;

use services::database::DatabaseService;
use services::dataset::DatasetService;
use services::export::ExportService;
use services::model::ModelService;

use tauri::Manager;

use crate::services::GenerationService;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            // Model commands
            commands::model::download_model,
            commands::model::cancel_download,
            commands::model::list_models,
            commands::model::delete_model,
            commands::model::get_default_gpu_layers,
            // Dataset commands
            commands::dataset::create_dataset,
            commands::dataset::list_datasets,
            commands::dataset::update_dataset,
            commands::dataset::delete_dataset,
            commands::dataset::get_columns,
            commands::dataset::create_column,
            commands::dataset::update_column,
            commands::dataset::delete_column,
            commands::dataset::fetch_rows,
            commands::dataset::update_row,
            commands::dataset::delete_row,
            commands::dataset::generate_rows,
            commands::dataset::cancel_generation,
            commands::dataset::get_optimal_gpu_layers,
            // export commands
            commands::dataset::export_to_csv,
        ])
        .setup(|app| {
            let db = DatabaseService::new(Some(app.handle()))
                .map_err(|e| format!("Failed to initialize database: {}", e))?;

            let dataset_service = DatasetService::new(db.clone())?;
            let export_service = ExportService::new(db.clone(), dataset_service.clone());
            let model_service = ModelService::new(Some(app.handle()), db.clone())?;
            let generation_service =
                GenerationService::new(db.clone(), dataset_service.clone(), model_service.clone())?;

            app.manage(db);
            app.manage(dataset_service);
            app.manage(export_service);
            app.manage(model_service);
            app.manage(generation_service);

            let window = app
                .get_webview_window("main")
                .ok_or_else(|| "Failed to get sample window".to_string())?;
            config::configure_window_size(&window)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
