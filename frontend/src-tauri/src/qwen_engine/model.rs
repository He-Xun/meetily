//! Qwen3-ASR model wrapper and inference logic.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use log::{info, debug, warn};
use qwen3_asr::{AsrInference, TranscribeOptions};

/// Model status for tracking availability and download state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelStatus {
    /// Model is downloaded and ready to use
    Available,
    /// Model is not downloaded
    Missing,
    /// Model is being downloaded (progress percentage 0-100)
    Downloading { progress: u8 },
    /// Model download or loading failed
    Error { message: String },
    /// Model file is corrupted and needs to be re-downloaded
    Corrupted { message: String },
}

/// Information about a Qwen model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub size_mb: u64,
    pub status: ModelStatus,
    pub description: String,
    pub path: Option<PathBuf>,
}

/// Transcript result from Qwen model
#[derive(Debug, Clone)]
pub struct TranscriptResult {
    pub text: String,
    pub language: Option<String>,
    pub confidence: Option<f32>,
}

/// Errors that can occur during Qwen model operations
#[derive(Debug, thiserror::Error)]
pub enum QwenError {
    #[error("Model not loaded")]
    ModelNotLoaded,

    #[error("Audio too short: {samples} samples (minimum {minimum})")]
    AudioTooShort { samples: usize, minimum: usize },

    #[error("Inference failed: {0}")]
    InferenceFailed(String),

    #[error("Model file not found: {0}")]
    ModelNotFound(String),

    #[error("Model file corrupted: {0}")]
    ModelCorrupted(String),

    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Download failed: {0}")]
    DownloadFailed(String),

    #[error("Qwen ASR error: {0}")]
    AsrError(String),
}

impl From<qwen3_asr::AsrError> for QwenError {
    fn from(err: qwen3_asr::AsrError) -> Self {
        QwenError::AsrError(err.to_string())
    }
}

/// Result type for Qwen operations
pub type Result<T> = std::result::Result<T, QwenError>;

/// Qwen ASR model wrapper
pub struct QwenModel {
    name: String,
    model_path: PathBuf,
    loaded: bool,
    inference: Option<AsrInference>,
}

impl QwenModel {
    /// Create a new Qwen model instance
    pub fn new(name: String, model_path: PathBuf) -> Self {
        Self {
            name,
            model_path,
            loaded: false,
            inference: None,
        }
    }

    /// Load the model into memory
    pub fn load(&mut self) -> Result<()> {
        info!("🌐 Loading Qwen3-ASR model: {} from {}", self.name, self.model_path.display());

        // Check if model directory exists
        if !self.model_path.exists() {
            return Err(QwenError::ModelNotFound(self.model_path.display().to_string()));
        }

        // Check for required model files
        let config_file = self.model_path.join("config.json");
        let tokenizer_file = self.model_path.join("tokenizer.json");

        if !config_file.exists() {
            return Err(QwenError::ModelCorrupted(
                format!("Missing config.json in {}", self.model_path.display())
            ));
        }

        if !tokenizer_file.exists() {
            return Err(QwenError::ModelCorrupted(
                format!("Missing tokenizer.json in {}", self.model_path.display())
            ));
        }

        // Try to load the model using qwen3-asr crate
        // Use best available device (Metal/CUDA/CPU)
        let device = qwen3_asr::best_device();
        debug!("🔧 Using device: {:?}", device);

        match AsrInference::load(&self.model_path, device) {
            Ok(inference) => {
                self.inference = Some(inference);
                self.loaded = true;
                info!("✅ Qwen3-ASR model '{}' loaded successfully", self.name);
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to load Qwen model: {}", e);
                warn!("❌ {}", error_msg);
                Err(QwenError::InferenceFailed(error_msg))
            }
        }
    }

    /// Transcribe audio samples to text
    ///
    /// # Arguments
    /// * `audio` - Audio samples (16kHz mono, f32 format)
    /// * `language` - Optional language hint (e.g., "zh", "en", "yue")
    ///
    /// # Returns
    /// * `TranscriptResult` with text, detected language, and confidence score
    pub fn transcribe(&self, audio: &[f32], language: Option<&str>) -> Result<TranscriptResult> {
        if !self.loaded {
            return Err(QwenError::ModelNotLoaded);
        }

        if audio.len() < 1600 { // Minimum 100ms at 16kHz
            return Err(QwenError::AudioTooShort {
                samples: audio.len(),
                minimum: 1600,
            });
        }

        let inference = self.inference.as_ref()
            .ok_or_else(|| QwenError::ModelNotLoaded)?;

        debug!("🎙️ Transcribing {} samples with language hint: {:?}", audio.len(), language);

        // Build transcription options
        let mut options = TranscribeOptions::default();
        if let Some(lang) = language {
            options.language = Some(lang.to_string());
        }

        // Perform transcription
        match inference.transcribe_samples(audio, options) {
            Ok(result) => {
                debug!("✅ Transcription successful: '{}' (language: {:?})", result.text, result.language);
                Ok(TranscriptResult {
                    text: result.text,
                    language: Some(result.language),
                    confidence: None, // Qwen3-ASR doesn't provide confidence scores
                })
            }
            Err(e) => {
                let error_msg = format!("Transcription failed: {}", e);
                warn!("❌ {}", error_msg);
                Err(QwenError::InferenceFailed(error_msg))
            }
        }
    }

    /// Unload the model from memory
    pub fn unload(&mut self) {
        if self.loaded {
            info!("🌐 Unloading Qwen3-ASR model: {}", self.name);
            self.inference = None;
            self.loaded = false;
        }
    }

    /// Check if model is loaded
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    /// Get model name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get model path
    pub fn path(&self) -> &PathBuf {
        &self.model_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_too_short() {
        let model = QwenModel::new(
            "test-model".to_string(),
            PathBuf::from("/fake/path"),
        );

        // Model not loaded, but audio too short check happens first
        let audio = vec![0.0f32; 100]; // Too short
        let result = model.transcribe(&audio, None);

        assert!(matches!(result, Err(QwenError::AudioTooShort { .. })));
    }
}
