//! Qwen3-ASR engine implementation
//!
//! Provides high-level interface for Qwen model management and transcription.

use crate::qwen_engine::model::{QwenModel, QwenError, Result, TranscriptResult, ModelStatus};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

// Convert QwenEngineError to QwenError
impl From<QwenEngineError> for QwenError {
    fn from(err: QwenEngineError) -> Self {
        match err {
            QwenEngineError::ModelNotFound(msg) => QwenError::ModelNotFound(msg),
            QwenEngineError::DownloadFailed(msg) => QwenError::DownloadFailed(msg),
            QwenEngineError::Io(err) => err.into(),
            QwenEngineError::ModelError(err) => err,
        }
    }
}

/// Download progress information
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    pub model_name: String,
    pub progress: u8, // 0-100
}

/// Model information for frontend
#[derive(Debug, Clone, serde::Serialize)]
pub struct ModelInfo {
    pub name: String,
    pub size_mb: u64,
    pub status: crate::qwen_engine::model::ModelStatus,
    pub description: String,
    pub path: Option<String>,
}

/// Qwen Engine errors
#[derive(Debug, thiserror::Error)]
pub enum QwenEngineError {
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Download failed: {0}")]
    DownloadFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Model error: {0}")]
    ModelError(#[from] QwenError),
}

/// Qwen speech recognition engine
pub struct QwenEngine {
    models_dir: PathBuf,
    current_model: Arc<RwLock<Option<QwenModel>>>,
    available_models: Arc<RwLock<Vec<crate::qwen_engine::model::ModelInfo>>>,
    cancel_token: Arc<tokio_util::sync::CancellationToken>,
}

impl QwenEngine {
    /// Create a new Qwen engine instance
    pub fn new() -> std::result::Result<Self, QwenEngineError> {
        let models_dir = Self::get_models_directory()?;

        // Ensure models directory exists
        std::fs::create_dir_all(&models_dir)?;

        Ok(Self {
            models_dir,
            current_model: Arc::new(RwLock::new(None)),
            available_models: Arc::new(RwLock::new(Vec::new())),
            cancel_token: Arc::new(tokio_util::sync::CancellationToken::new()),
        })
    }

    /// Get the models directory path
    fn get_models_directory() -> std::result::Result<PathBuf, QwenEngineError> {
        let mut models_dir = dirs::home_dir()
            .ok_or_else(|| QwenEngineError::ModelNotFound("Home directory not found".to_string()))?;

        models_dir.push(".meetily");
        models_dir.push("qwen_models");

        Ok(models_dir)
    }

    /// Discover available models in the models directory
    pub async fn discover_models(&self) -> Result<Vec<ModelInfo>> {
        let mut models = Vec::new();

        // Define available Qwen models
        let known_models = vec![
            ("qwen3-asr-1.7b", 3200, "Qwen3-ASR 1.7B - 多语言高精度"),
            ("qwen3-asr-0.6b", 1200, "Qwen3-ASR 0.6B - 轻量高效"),
        ];

        for (name, size_mb, description) in known_models {
            let model_path = self.models_dir.join(name);

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

            models.push(crate::qwen_engine::model::ModelInfo {
                name: name.to_string(),
                size_mb,
                status,
                description: description.to_string(),
                path: Some(model_path),
            });
        }

        // Update cached models
        *self.available_models.write().await = models.clone();

        Ok(models
            .into_iter()
            .map(|m| ModelInfo {
                name: m.name,
                size_mb: m.size_mb,
                status: m.status,
                description: m.description,
                path: m.path.map(|p| p.display().to_string()),
            })
            .collect())
    }

    /// Load a model by name
    pub async fn load_model(&self, model_name: &str) -> Result<()> {
        let models = self.available_models.read().await;

        let model_info = models
            .iter()
            .find(|m| m.name == model_name)
            .ok_or_else(|| QwenError::ModelNotFound(format!("Model '{}' not found", model_name)))?;

        if !matches!(model_info.status, ModelStatus::Available) {
            return Err(QwenError::ModelNotFound(format!(
                "Model '{}' is not available. Current status: {:?}",
                model_name, model_info.status
            )));
        }

        let model_path = model_info.path.as_ref()
            .ok_or_else(|| QwenError::ModelNotFound("Model path not set".to_string()))?;

        let mut model = QwenModel::new(model_name.to_string(), model_path.clone());
        model.load()?;

        *self.current_model.write().await = Some(model);

        Ok(())
    }

    /// Transcribe audio samples
    pub async fn transcribe(&self, audio: Vec<f32>, language: Option<String>) -> Result<TranscriptResult> {
        let model_guard = self.current_model.read().await;
        let model = model_guard
            .as_ref()
            .ok_or_else(|| QwenError::ModelNotLoaded)?;

        model.transcribe(&audio, language.as_deref())
    }

    /// Get the currently loaded model name
    pub async fn get_current_model(&self) -> Option<String> {
        let model_guard = self.current_model.read().await;
        model_guard.as_ref().map(|m| m.name().to_string())
    }

    /// Check if a model is currently loaded
    pub async fn is_model_loaded(&self) -> bool {
        let model_guard = self.current_model.read().await;
        model_guard.as_ref().map(|m| m.is_loaded()).unwrap_or(false)
    }

    /// Unload the current model
    pub async fn unload_model(&self) -> Result<()> {
        let mut model_guard = self.current_model.write().await;
        if let Some(mut model) = model_guard.take() {
            model.unload();
        }
        Ok(())
    }

    /// Get model path
    pub fn get_model_path(&self, model_name: &str) -> PathBuf {
        self.models_dir.join(model_name)
    }

    /// Cancel the current download operation
    pub async fn cancel_download(&self) {
        log::info!("🌐 Cancelling current download operation");
        self.cancel_token.cancel();
        // Give the download operation a moment to notice the cancellation
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    /// Delete a model
    pub async fn delete_model(&self, model_name: &str) -> Result<()> {
        let model_path = self.get_model_path(model_name);

        if model_path.exists() {
            std::fs::remove_dir_all(&model_path)?;

            // Update model status
            let mut models = self.available_models.write().await;
            if let Some(model) = models.iter_mut().find(|m| m.name == model_name) {
                model.status = ModelStatus::Missing;
            }
        }

        // Unload if this was the current model
        if self.get_current_model().await.as_deref() == Some(model_name) {
            self.unload_model().await?;
        }

        Ok(())
    }

    /// Download a Qwen model from HuggingFace
    pub async fn download_model(
        &self,
        model_name: &str,
        progress_callback: Option<Box<dyn Fn(u8) + Send>>,
    ) -> Result<()> {
        log::info!("🌐 Starting download for Qwen model: {}", model_name);

        // Create a fresh cancellation token for this download
        let cancel_token = self.cancel_token.child_token();

        // Check if model exists in our known models
        let models = self.available_models.read().await;
        let _model_info = models.iter().find(|m| m.name == model_name)
            .ok_or_else(|| QwenEngineError::ModelNotFound(format!("Model '{}' not found", model_name)))?;

        let model_dir = &self.models_dir.join(model_name);

        // Update status to downloading
        drop(models);
        {
            let mut models = self.available_models.write().await;
            if let Some(model) = models.iter_mut().find(|m| m.name == model_name) {
                model.status = crate::qwen_engine::model::ModelStatus::Downloading { progress: 0 };
            }
        }

        // Create model directory
        tokio::fs::create_dir_all(model_dir).await
            .map_err(|e| QwenEngineError::DownloadFailed(format!("Failed to create directory: {}", e)))?;

        // HuggingFace repository for Qwen3-ASR models
        let repo_name = match model_name {
            "qwen3-asr-1.7b" => "Qwen/Qwen3-ASR-1.7B",
            "qwen3-asr-0.6b" => "Qwen/Qwen3-ASR-0.6B",
            _ => return Err(QwenEngineError::ModelNotFound(format!("Unknown model: {}", model_name)).into()),
        };

        // Try multiple download sources in order
        // 1. hf-mirror.com (fast for China, no auth required)
        // 2. huggingface.co (official, may require auth for some models)
        let download_bases = vec![
            "https://hf-mirror.com",
            "https://huggingface.co",
        ];

        // Files to download
        // Note: tokenizer.json is not available on HuggingFace for Qwen3-ASR models
        // The qwen3-asr crate should work with tokenizer_config.json + vocab.json + merges.txt
        let required_files = vec![
            "config.json",
            "tokenizer_config.json",
            "vocab.json",
            "merges.txt",
            "generation_config.json",
            "preprocessor_config.json",
            "chat_template.json",
            "model.safetensors.index.json",
            "model-00001-of-00002.safetensors",
            "model-00002-of-00002.safetensors",
        ];

        // Optional files that may not exist on HuggingFace
        let optional_files = vec!["tokenizer.json"];

        let total_files = required_files.len();
        let mut completed_files = 0;

        // Calculate file weights for progress: safetensors files get 45% each, others split remaining 10%
        // This makes progress smoother instead of jumping 10% per file
        let get_file_weight = |filename: &str| -> f64 {
            if filename.contains("safetensors") {
                45.0  // Each safetensors file is ~45% of total
            } else {
                1.25   // Small files share the remaining 10% (8 files × 1.25% = 10%)
            }
        };

        let mut total_weight: f64 = required_files.iter().map(|f| get_file_weight(f)).sum();
        let mut completed_weight: f64 = 0.0;

        for (index, filename) in required_files.iter().enumerate() {
            let file_path = model_dir.join(filename);

            log::info!("🔍 Processing file {}/{}: {}", index + 1, total_files, filename);
            log::info!("🔍 Target path: {}", file_path.display());

            // Skip if file already exists and has valid content
            // safetensors files should be large (>100MB), other files just need to exist
            if file_path.exists() {
                let metadata = tokio::fs::metadata(&file_path).await
                    .map_err(|e| QwenEngineError::DownloadFailed(format!("Failed to check file: {}", e)))?;

                let file_size = metadata.len();
                let is_safetensors = filename.contains("safetensors");
                let is_valid = if is_safetensors {
                    // safetensors files should be at least 100MB
                    file_size > 100_000_000
                } else {
                    // Other files just need to exist and have content
                    file_size > 0
                };

                if is_valid {
                    log::info!("✓ Skipping existing valid file: {} ({} MB)", filename, file_size / 1_048_576);
                    let file_weight = get_file_weight(filename);
                    completed_weight += file_weight;
                    completed_files += 1;
                    // Update progress based on weight
                    if let Some(ref cb) = progress_callback {
                        let progress = ((completed_weight / total_weight) * 100.0) as u8;
                        cb(progress);
                        {
                            let mut models = self.available_models.write().await;
                            if let Some(model) = models.iter_mut().find(|m| m.name == model_name) {
                                model.status = crate::qwen_engine::model::ModelStatus::Downloading { progress };
                            }
                        }
                    }
                    continue;
                } else {
                    log::warn!("⚠️ File exists but invalid ({} bytes), re-downloading: {}", file_size, filename);
                }
            }

            // Check for cancellation before starting download
            if cancel_token.is_cancelled() {
                log::warn!("⚠️ Download cancelled by user");
                return Err(QwenEngineError::DownloadFailed("Download cancelled".to_string()).into());
            }

            log::info!("📥 Downloading file {}/{}: {}", index + 1, total_files, filename);

            // Try multiple download sources
            let mut download_success = false;
            let mut last_error = None;

            for base_url in &download_bases {
                // Construct download URL
                let url = format!("{}/{}/resolve/main/{}", base_url, repo_name, filename);
                log::info!("📥 Trying: {}", url);

                // Download with timeout and optional HuggingFace token
                let client = reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(3600)) // 1 hour timeout
                    .build()
                    .map_err(|e| QwenEngineError::DownloadFailed(format!("Failed to create HTTP client: {}", e)))?;

                // Try to get HuggingFace token from environment (only for official HF)
                let hf_token = if *base_url == "https://huggingface.co" {
                    std::env::var("HUGGINGFACE_TOKEN").ok()
                } else {
                    None
                };

                log::info!("📥 Starting HTTP request from {}...", base_url);
                let mut request = client.get(&url);

                // Add authorization header if token is available
                if let Some(token) = &hf_token {
                    log::info!("🔑 Using HuggingFace token for authentication");
                    request = request.header("Authorization", format!("Bearer {}", token));
                }

                match request.send().await {
                    Ok(response) => {
                        if response.status().is_success() {
                            log::info!("✅ Successfully started download from {}", base_url);

                            // Get total file size for progress tracking
                            let total_size = response.content_length().unwrap_or(0);

                            // Download in chunks and write to file
                            let mut file = tokio::fs::File::create(&file_path).await
                                .map_err(|e| QwenEngineError::DownloadFailed(format!("Failed to create file {}: {}", filename, e)))?;

                            let mut downloaded: u64 = 0;
                            let mut stream = response.bytes_stream();

                            use futures_util::StreamExt;
                            let mut download_complete = false;

                            while let Some(chunk_result) = stream.next().await {
                                // Check for cancellation during download
                                if cancel_token.is_cancelled() {
                                    log::warn!("⚠️ Download cancelled during file transfer: {}", filename);
                                    // Clean up partial file
                                    let _ = tokio::fs::remove_file(&file_path).await;
                                    return Err(QwenEngineError::DownloadFailed("Download cancelled".to_string()).into());
                                }

                                let chunk = chunk_result
                                    .map_err(|e| QwenEngineError::DownloadFailed(format!("Download error for {}: {}", filename, e)))?;

                                tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await
                                    .map_err(|e| QwenEngineError::DownloadFailed(format!("Failed to write file {}: {}", filename, e)))?;

                                downloaded += chunk.len() as u64;

                                // Update progress for large files with weight-based calculation
                                if total_size > 0 && total_size > 100_000_000 { // Only for files > 100MB
                                    let file_weight = get_file_weight(filename);
                                    let file_progress = (downloaded as f64 / total_size as f64) * file_weight;
                                    let overall_progress = ((completed_weight + file_progress) / total_weight * 100.0) as u8;

                                    if let Some(ref cb) = progress_callback {
                                        cb(overall_progress);
                                        {
                                            let mut models = self.available_models.write().await;
                                            if let Some(model) = models.iter_mut().find(|m| m.name == model_name) {
                                                model.status = crate::qwen_engine::model::ModelStatus::Downloading { progress: overall_progress };
                                            }
                                        }
                                    }
                                }
                            }

                            download_complete = true;
                            download_success = true;
                            log::info!("✅ Download completed from {}", base_url);
                            break;
                        } else {
                            let status = response.status();
                            log::warn!("⚠️ HTTP {} from {} (trying next source)", status, base_url);
                            last_error = Some(format!("HTTP {}", status));
                        }
                    }
                    Err(e) => {
                        log::warn!("⚠️ Failed to download from {}: {} (trying next source)", base_url, e);
                        last_error = Some(format!("Network error: {}", e));
                    }
                }
            }

            if !download_success {
                return Err(QwenEngineError::DownloadFailed(
                    format!("Failed to download {} from all sources. Last error: {}", filename, last_error.unwrap_or_else(|| "Unknown error".to_string()))
                ).into());
            }

            // Add completed file weight
            let file_weight = get_file_weight(filename);
            completed_weight += file_weight;
            completed_files += 1;

            // Verify file was actually written
            if !file_path.exists() {
                return Err(QwenEngineError::DownloadFailed(
                    format!("File download completed but file not found: {}", filename)
                ).into());
            }

            let metadata = std::fs::metadata(&file_path)
                .map_err(|e| QwenEngineError::DownloadFailed(format!("Failed to verify file {}: {}", filename, e)))?;
            let final_size = metadata.len();
            let downloaded_mb = final_size as f64 / 1_048_576.0;
            log::info!("✓ Downloaded: {} ({:.2} MB) - {}/{} files complete", filename, downloaded_mb, completed_files, total_files);

        }

        // Download optional files (if available on HuggingFace)
        log::info!("📦 Downloading optional files...");
        for filename in optional_files {
            let file_path = model_dir.join(filename);

            // Skip if file already exists
            if file_path.exists() {
                log::info!("✓ Optional file already exists: {}", filename);
                continue;
            }

            log::info!("📥 Attempting optional file: {} (may not exist on HuggingFace)", filename);

            let url = format!("https://huggingface.co/{}/resolve/main/{}", repo_name, filename);

            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(60)) // Shorter timeout for optional files
                .build()
                .map_err(|e| QwenEngineError::DownloadFailed(format!("Failed to create HTTP client: {}", e)))?;

            // Try to get HuggingFace token from environment
            let hf_token = std::env::var("HUGGINGFACE_TOKEN").ok();

            let mut request = client.get(&url);
            if let Some(token) = &hf_token {
                request = request.header("Authorization", format!("Bearer {}", token));
            }

            let response = request.send().await;

            match response {
                Ok(resp) => {
                    if resp.status().is_success() {
                        let _total_size = resp.content_length().unwrap_or(0);

                        let mut file = tokio::fs::File::create(&file_path).await
                            .map_err(|e| QwenEngineError::DownloadFailed(format!("Failed to create file {}: {}", filename, e)))?;

                        let mut downloaded: u64 = 0;
                        let mut stream = resp.bytes_stream();

                        use futures_util::StreamExt;
                        while let Some(chunk_result) = stream.next().await {
                            let chunk = chunk_result
                                .map_err(|e| QwenEngineError::DownloadFailed(format!("Download error for {}: {}", filename, e)))?;

                            tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await
                                .map_err(|e| QwenEngineError::DownloadFailed(format!("Failed to write file {}: {}", filename, e)))?;

                            downloaded += chunk.len() as u64;
                        }

                        log::info!("✓ Downloaded optional file: {} ({:.2} MB)", filename, downloaded as f64 / 1_048_576.0);
                    } else {
                        log::info!("ℹ️ Optional file not found on HuggingFace: {} (status: {})", filename, resp.status());
                    }
                }
                Err(e) => {
                    log::info!("ℹ️ Optional file download failed (this is OK): {} - {}", filename, e);
                }
            }
        }

        // Update model status to Available
        {
            let mut models = self.available_models.write().await;
            if let Some(model) = models.iter_mut().find(|m| m.name == model_name) {
                model.status = crate::qwen_engine::model::ModelStatus::Available;
            }
        }

        log::info!("✅ Model download completed: {}", model_name);
        Ok(())
    }
}

impl Default for QwenEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create Qwen engine")
    }
}
