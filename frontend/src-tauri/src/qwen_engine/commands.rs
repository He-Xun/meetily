//! Tauri command interface for Qwen3-ASR engine

use crate::qwen_engine::qwen_engine::QwenEngine;
use crate::qwen_engine::model::ModelStatus;
use std::sync::{Arc, Mutex as StdMutex};
use std::path::PathBuf;
use tokio::sync::Mutex;
use tauri::{AppHandle, Emitter, Runtime, Manager};
use log::{info, warn};

/// Global Qwen engine instance
pub static QWEN_ENGINE: Mutex<Option<Arc<QwenEngine>>> = Mutex::const_new(None);

/// Global models directory path (set during app initialization)
static MODELS_DIR: StdMutex<Option<PathBuf>> = StdMutex::new(None);

/// Initialize the models directory path using app_data_dir
/// This should be called during app setup before qwen_init
pub fn set_models_directory<R: Runtime>(_app: &AppHandle<R>) {
    // Use the same path as QwenEngine::get_models_directory()
    // ~/.meetily/qwen_models
    let models_dir = dirs::home_dir()
        .expect("Failed to get home directory")
        .join(".meetily")
        .join("qwen_models");

    // Create directory if it doesn't exist
    if !models_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&models_dir) {
            log::error!("Failed to create Qwen models directory: {}", e);
            return;
        }
    }

    log::info!("Qwen models directory set to: {}", models_dir.display());

    let mut guard = MODELS_DIR.lock().unwrap();
    *guard = Some(models_dir);
}

/// Get the configured models directory
fn get_models_directory() -> Option<PathBuf> {
    MODELS_DIR.lock().unwrap().clone()
}

/// Initialize the Qwen engine
#[tauri::command]
pub async fn qwen_init() -> Result<(), String> {
    info!("🌐 Initializing Qwen3-ASR engine...");

    let engine = QwenEngine::new()
        .map_err(|e| format!("Failed to create Qwen engine: {}", e))?;

    // Discover available models
    let models = engine.discover_models()
        .await
        .map_err(|e| format!("Failed to discover models: {}", e))?;

    info!("🌐 Qwen engine initialized with {} models available", models.len());

    // Store the engine globally
    let mut guard = QWEN_ENGINE.lock().await;
    *guard = Some(Arc::new(engine));

    Ok(())
}

/// Get list of available Qwen models
#[tauri::command]
pub async fn qwen_get_available_models() -> Result<Vec<crate::qwen_engine::qwen_engine::ModelInfo>, String> {
    let engine = {
        let guard = QWEN_ENGINE.lock().await;
        guard.as_ref().cloned()
    };

    if let Some(engine) = engine {
        engine.discover_models()
            .await
            .map_err(|e| format!("Failed to get models: {}", e))
    } else {
        // Fallback: scan models directory directly without initialized engine
        log::info!("Qwen engine not initialized, scanning models directory directly");
        discover_models_standalone()
    }
}

/// Discover Qwen models by scanning the models directory directly
/// Used when the Qwen engine isn't initialized
fn discover_models_standalone() -> Result<Vec<crate::qwen_engine::qwen_engine::ModelInfo>, String> {
    let models_dir = get_models_directory()
        .ok_or_else(|| "Qwen models directory not initialized".to_string())?;

    log::info!("Scanning for Qwen models in: {}", models_dir.display());

    // Define available Qwen models
    let known_models = vec![
        ("qwen3-asr-1.7b", 3200u64, "Qwen3-ASR 1.7B - 多语言高精度"),
        ("qwen3-asr-0.6b", 1200u64, "Qwen3-ASR 0.6B - 轻量高效"),
    ];

    let mut models = Vec::new();

    for (name, size_mb, description) in known_models {
        let model_path = models_dir.join(name);

        // Check if model is actually available by verifying key files exist
        let config_file = model_path.join("config.json");
        let safetensors_1 = model_path.join("model-00001-of-00002.safetensors");
        let safetensors_2 = model_path.join("model-00002-of-00002.safetensors");

        let is_available = model_path.exists() &&
                          config_file.exists() &&
                          safetensors_1.exists() &&
                          safetensors_2.exists();

        let status = if is_available {
            ModelStatus::Available
        } else {
            ModelStatus::Missing
        };

        models.push(crate::qwen_engine::qwen_engine::ModelInfo {
            name: name.to_string(),
            size_mb,
            status,
            description: description.to_string(),
            path: Some(model_path.display().to_string()),
        });
    }

    Ok(models)
}

/// Download a Qwen model
#[tauri::command]
pub async fn qwen_download_model(
    app: AppHandle,
    model_name: String
) -> Result<(), String> {
    info!("🌐 Starting download for Qwen model: {}", model_name);

    let engine = get_engine().await?;

    // Clone model_name for use in callback
    let model_name_for_callback = model_name.clone();

    // Clone app handle for progress callback
    let app_clone = app.clone();

    // Create progress callback
    let progress_callback = Box::new(move |progress: u8| {
        info!("📊 Download progress: {}%", progress);

        // Emit progress event to frontend
        let _ = app_clone.emit("qwen-model-download-progress", serde_json::json!({
            "modelName": model_name_for_callback,
            "progress": progress
        }));
    });

    // Perform download
    match engine.download_model(&model_name, Some(progress_callback)).await {
        Ok(()) => {
            info!("✅ Successfully downloaded Qwen model: {}", model_name);

            // Emit completion event
            app.emit("qwen-model-download-complete", serde_json::json!({
                "modelName": model_name
            }))
            .map_err(|e| format!("Failed to emit completion event: {}", e))?;

            Ok(())
        }
        Err(e) => {
            warn!("❌ Failed to download Qwen model: {}", e);

            // Emit error event
            app.emit("qwen-model-download-error", serde_json::json!({
                "modelName": model_name,
                "error": e.to_string()
            }))
            .map_err(|e| format!("Failed to emit error event: {}", e))?;

            Err(format!("Failed to download model: {}", e))
        }
    }
}

/// Cancel model download
#[tauri::command]
pub async fn qwen_cancel_download(model_name: String) -> Result<(), String> {
    info!("🌐 Cancelling download for Qwen model: {}", model_name);

    let engine = get_engine().await?;

    // Trigger cancellation - this will cause download to fail and clean up partial files
    engine.cancel_download().await;

    // Refresh model discovery to update status
    let _ = engine.discover_models().await;

    Ok(())
}

/// Delete a Qwen model
#[tauri::command]
pub async fn qwen_delete_model(model_name: String) -> Result<(), String> {
    info!("🌐 Deleting Qwen model: {}", model_name);

    let engine = get_engine().await?;

    engine.delete_model(&model_name)
        .await
        .map_err(|e| format!("Failed to delete model: {}", e))?;

    info!("✅ Successfully deleted Qwen model: {}", model_name);

    Ok(())
}

/// Load a Qwen model
#[tauri::command]
pub async fn qwen_load_model(model_name: String) -> Result<(), String> {
    info!("🌐 Loading Qwen model: {}", model_name);

    let engine = get_engine().await?;

    engine.load_model(&model_name)
        .await
        .map_err(|e| format!("Failed to load model: {}", e))?;

    info!("✅ Successfully loaded Qwen model: {}", model_name);

    Ok(())
}

/// Unload the current Qwen model
#[tauri::command]
pub async fn qwen_unload_model() -> Result<(), String> {
    info!("🌐 Unloading Qwen model");

    let engine = get_engine().await?;

    engine.unload_model()
        .await
        .map_err(|e| format!("Failed to unload model: {}", e))?;

    info!("✅ Successfully unloaded Qwen model");

    Ok(())
}

/// Check if a model is loaded
#[tauri::command]
pub async fn qwen_is_model_loaded() -> Result<bool, String> {
    let engine = get_engine().await?;
    Ok(engine.is_model_loaded().await)
}

/// Get the currently loaded model name
#[tauri::command]
pub async fn qwen_get_current_model() -> Result<Option<String>, String> {
    let engine = get_engine().await?;
    Ok(engine.get_current_model().await)
}

/// Transcribe audio samples
#[tauri::command]
pub async fn qwen_transcribe(
    audio: Vec<f32>,
    language: Option<String>
) -> Result<String, String> {
    let engine = get_engine().await?;

    let result = engine.transcribe(audio, language)
        .await
        .map_err(|e| format!("Transcription failed: {}", e))?;

    Ok(result.text)
}

/// Helper function to get the engine instance
async fn get_engine() -> Result<Arc<QwenEngine>, String> {
    let guard = QWEN_ENGINE.lock().await;
    guard.as_ref()
        .cloned()
        .ok_or_else(|| "Qwen engine not initialized. Call qwen_init first.".to_string())
}
