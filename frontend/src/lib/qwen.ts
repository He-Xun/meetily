// Qwen3-ASR Model Types and Utilities

export type ModelStatus =
  | 'Available'
  | 'Missing'
  | { Downloading: number }
  | { Error: string }
  | { Corrupted: string };

export interface QwenModelInfo {
  name: string;
  size_mb: number;
  status: ModelStatus;
  description?: string;
  path?: string;
}

// Model display information
export interface ModelDisplayInfo {
  friendlyName: string;
  icon: string;
  tagline: string;
  description: string;
  languages: string[];
}

// Model display configurations
const MODEL_DISPLAY_INFO: Record<string, ModelDisplayInfo> = {
  'qwen3-asr-1.7b': {
    friendlyName: 'Qwen3-ASR 1.7B',
    icon: '🌐',
    tagline: '多语言高精度',
    description: '支持30种语言和22种中文方言',
    languages: ['中文', '英文', '粤语', '日语', '韩语', '法语', '德语', '西班牙语', '俄语', '阿拉伯语', '...']
  },
  'qwen3-asr-0.6b': {
    friendlyName: 'Qwen3-ASR 0.6B',
    icon: '🎯',
    tagline: '轻量高效',
    description: '快速多语言识别',
    languages: ['中文', '英文', '粤语', '日语', '韩语']
  }
};

export function getModelDisplayInfo(modelName: string): ModelDisplayInfo | undefined {
  return MODEL_DISPLAY_INFO[modelName];
}

export function getModelDisplayName(modelName: string): string {
  const info = getModelDisplayInfo(modelName);
  return info?.friendlyName || modelName;
}

export function formatFileSize(sizeMb: number): string {
  if (sizeMb < 1024) {
    return `${sizeMb.toFixed(0)} MB`;
  } else {
    return `${(sizeMb / 1024).toFixed(1)} GB`;
  }
}

// Qwen API utilities
export const QwenAPI = {
  async init(): Promise<void> {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke('qwen_init');
  },

  async getAvailableModels(): Promise<QwenModelInfo[]> {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke('qwen_get_available_models');
  },

  async downloadModel(modelName: string): Promise<void> {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke('qwen_download_model', { modelName });
  },

  async cancelDownload(modelName: string): Promise<void> {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke('qwen_cancel_download', { modelName });
  },

  async deleteModel(modelName: string): Promise<void> {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke('qwen_delete_model', { modelName });
  },

  async loadModel(modelName: string): Promise<void> {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke('qwen_load_model', { modelName });
  },

  async unloadModel(): Promise<void> {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke('qwen_unload_model');
  }
};
