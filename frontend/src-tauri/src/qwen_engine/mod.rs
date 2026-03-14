//! Qwen3-ASR speech recognition engine module.
//!
//! This module provides multi-language speech recognition using Qwen3-ASR models.
//! Supports 30+ languages and 22 Chinese dialects with GPU acceleration.
//!
//! # Features
//!
//! - **Multi-language**: Supports 30 languages including Chinese, English, Japanese, Korean, etc.
//! - **Dialect Support**: 22 Chinese dialects (Cantonese, Hakka, etc.)
//! - **GPU Acceleration**: Metal (macOS), CUDA (NVIDIA), CPU fallback
//! - **Unified API**: Compatible interface with Whisper and Parakeet engines
//!
//! # Module Structure
//!
//! - `qwen_engine`: Main engine implementation
//! - `model`: Qwen model wrapper and inference logic
//! - `commands`: Tauri command interface for frontend integration
//! - `provider`: TranscriptionProvider trait implementation

pub mod qwen_engine;
pub mod model;
pub mod commands;
pub mod provider;

pub use qwen_engine::{QwenEngine, QwenEngineError, ModelInfo as QwenEngineModelInfo, DownloadProgress};
pub use model::{QwenModel, QwenError, TranscriptResult as QwenTranscriptResult, ModelStatus};
pub use commands::*;
pub use provider::QwenProvider;
