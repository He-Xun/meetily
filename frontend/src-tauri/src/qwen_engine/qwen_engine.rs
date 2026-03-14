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
    pub status: String, // Serialized status
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
            let status = if model_path.exists() {
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
                status: serde_json::to_string(&m.status).unwrap_or_else(|_| "Unknown".to_string()),
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

        // Files to download
        // Note: tokenizer.json needs to be generated using Python transformers
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

        let total_files = required_files.len();
        let mut completed_files = 0;

        for (index, filename) in required_files.iter().enumerate() {
            let file_path = model_dir.join(filename);

            // Skip if file already exists and has content
            if file_path.exists() {
                let metadata = tokio::fs::metadata(&file_path).await
                    .map_err(|e| QwenEngineError::DownloadFailed(format!("Failed to check file: {}", e)))?;

                if metadata.len() > 0 {
                    log::info!("✓ Skipping existing file: {}", filename);
                    completed_files += 1;
                    // Update progress
                    if let Some(ref cb) = progress_callback {
                        let progress = ((index + 1) * 100 / total_files) as u8;
                        cb(progress);
                        {
                            let mut models = self.available_models.write().await;
                            if let Some(model) = models.iter_mut().find(|m| m.name == model_name) {
                                model.status = crate::qwen_engine::model::ModelStatus::Downloading { progress };
                            }
                        }
                    }
                    continue;
                }
            }

            log::info!("📥 Downloading file {}/{}: {}", index + 1, total_files, filename);

            // Construct HuggingFace download URL
            let url = format!("https://huggingface.co/{}/resolve/main/{}", repo_name, filename);

            // Download with timeout
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(3600)) // 1 hour timeout
                .build()
                .map_err(|e| QwenEngineError::DownloadFailed(format!("Failed to create HTTP client: {}", e)))?;

            let response = client.get(&url).send().await
                .map_err(|e| QwenEngineError::DownloadFailed(format!("Failed to download {}: {}", filename, e)))?;

            if !response.status().is_success() {
                return Err(QwenEngineError::DownloadFailed(
                    format!("Failed to download {}: HTTP {}", filename, response.status())
                ).into());
            }

            // Get total file size for progress tracking
            let total_size = response.content_length().unwrap_or(0);

            // Download in chunks and write to file
            let mut file = tokio::fs::File::create(&file_path).await
                .map_err(|e| QwenEngineError::DownloadFailed(format!("Failed to create file {}: {}", filename, e)))?;

            let mut downloaded: u64 = 0;
            let mut stream = response.bytes_stream();

            use futures_util::StreamExt;
            while let Some(chunk_result) = stream.next().await {
                let chunk = chunk_result
                    .map_err(|e| QwenEngineError::DownloadFailed(format!("Download error for {}: {}", filename, e)))?;

                tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await
                    .map_err(|e| QwenEngineError::DownloadFailed(format!("Failed to write file {}: {}", filename, e)))?;

                downloaded += chunk.len() as u64;

                // Update progress for large files
                if total_size > 0 && total_size > 100_000_000 { // Only for files > 100MB
                    let file_progress = ((downloaded as f64 / total_size as f64) * 100.0) as u8;
                    let overall_progress = ((completed_files * 100 + (index * 100 + file_progress as usize)) / total_files) as u8;

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

            completed_files += 1;
            log::info!("✓ Downloaded: {} ({:.2} MB)", filename, downloaded as f64 / 1_048_576.0);

            // Update progress after each file
            if let Some(ref cb) = progress_callback {
                let progress = ((index + 1) * 100 / total_files) as u8;
                cb(progress);
                {
                    let mut models = self.available_models.write().await;
                    if let Some(model) = models.iter_mut().find(|m| m.name == model_name) {
                        model.status = crate::qwen_engine::model::ModelStatus::Downloading { progress };
                    }
                }
            }
        }

        // Generate tokenizer.json if it doesn't exist
        let tokenizer_path = model_dir.join("tokenizer.json");
        if !tokenizer_path.exists() {
            log::info!("⚠️ Note: tokenizer.json needs to be generated using Python transformers");
            log::info!("⚠️ Please run: python -c \"from transformers import AutoTokenizer; tok = AutoTokenizer.from_pretrained('{}', trust_remote_code=True); tok.backend_tokenizer.save('{}')\"",
                repo_name, model_dir.display());
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
