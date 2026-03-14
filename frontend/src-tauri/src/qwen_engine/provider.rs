//! Qwen3-ASR provider implementation for TranscriptionProvider trait

use async_trait::async_trait;
use std::sync::Arc;
use crate::audio::transcription::provider::{TranscriptionProvider, TranscriptionError, TranscriptResult};
use super::{QwenEngine, QwenError};

/// Qwen transcription provider wrapper
/// Implements TranscriptionProvider trait for QwenEngine
pub struct QwenProvider {
    engine: Arc<QwenEngine>,
}

impl QwenProvider {
    /// Create a new Qwen provider
    pub fn new(engine: Arc<QwenEngine>) -> Self {
        Self { engine }
    }

    /// Get the underlying Qwen engine
    pub fn engine(&self) -> &Arc<QwenEngine> {
        &self.engine
    }
}

/// Convert QwenError to TranscriptionError
impl From<QwenError> for TranscriptionError {
    fn from(err: QwenError) -> Self {
        match err {
            QwenError::ModelNotLoaded => TranscriptionError::ModelNotLoaded,
            QwenError::AudioTooShort { samples, minimum } => {
                TranscriptionError::AudioTooShort { samples, minimum }
            }
            QwenError::InferenceFailed(msg) => TranscriptionError::EngineFailed(msg),
            QwenError::UnsupportedLanguage(lang) => TranscriptionError::UnsupportedLanguage(lang),
            QwenError::ModelNotFound(msg) => TranscriptionError::EngineFailed(msg),
            QwenError::ModelCorrupted(msg) => TranscriptionError::EngineFailed(msg),
            QwenError::Io(err) => TranscriptionError::EngineFailed(err.to_string()),
            QwenError::DownloadFailed(msg) => TranscriptionError::EngineFailed(msg),
            QwenError::AsrError(msg) => TranscriptionError::EngineFailed(msg),
        }
    }
}

#[async_trait]
impl TranscriptionProvider for QwenProvider {
    async fn transcribe(
        &self,
        audio: Vec<f32>,
        language: Option<String>,
    ) -> std::result::Result<TranscriptResult, TranscriptionError> {
        let qwen_result = self.engine.transcribe(audio, language).await?;

        Ok(TranscriptResult {
            text: qwen_result.text,
            confidence: qwen_result.confidence,
            is_partial: false, // Qwen doesn't support partial results in current implementation
        })
    }

    async fn is_model_loaded(&self) -> bool {
        self.engine.is_model_loaded().await
    }

    async fn get_current_model(&self) -> Option<String> {
        self.engine.get_current_model().await
    }

    fn provider_name(&self) -> &'static str {
        "Qwen3-ASR"
    }
}
