//! Tauri command interface for Qwen3-ASR engine

use crate::qwen_engine::qwen_engine::QwenEngine;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::{AppHandle, Emitter};
use log::{info, warn};

/// Global Qwen engine instance
pub static QWEN_ENGINE: Mutex<Option<Arc<QwenEngine>>> = Mutex::const_new(None);

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
pub async fn qwen_get_available_models() -> Result<Vec<serde_json::Value>, String> {
    let engine = get_engine().await?;

    let models = engine.discover_models()
        .await
        .map_err(|e| format!("Failed to get models: {}", e))?;

    // Convert to serde_json::Value for frontend compatibility
    let json_models: Vec<serde_json::Value> = models
        .into_iter()
        .map(|m| serde_json::json!(m))
        .collect();

    Ok(json_models)
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

    // TODO: Implement download cancellation
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
