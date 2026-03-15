pub fn format_timestamp(seconds: f64) -> String {
    let total_seconds = seconds as u64;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let secs = total_seconds % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, secs)
}

/// Text processing utilities
pub mod text {
    /// Clean up Qwen3-ASR output format artifacts
    /// 
    /// Some versions of Qwen3-ASR output format markers like "onghua<asr_text>" before actual text.
    /// This function removes these artifacts to get clean transcription text.
    /// 
    /// Examples:
    /// - "onghua<asr_text>这是中文" -> "这是中文"
    /// - "<asr_text>This is English" -> "This is English"
    /// - "onghuaThis is text" -> "This is text"
    pub fn clean_qwen_asr_output(text: &str) -> String {
        let trimmed = text.trim();
        
        // Pattern 1: Remove "onghua<asr_text>" or "onghua<asr_text|...>" prefix
        if let Some(pos) = trimmed.find("<asr_text>") {
            // Check if there's "onghua" or similar before it
            let before_tag = &trimmed[..pos];
            if before_tag.ends_with("onghua") || before_tag.ends_with("nghua") {
                return trimmed[pos + "<asr_text>".len()..].trim().to_string();
            }
            // Just remove the <asr_text> tag itself
            return trimmed[..pos].to_string() + &trimmed[pos + "<asr_text>".len()..];
        }
        
        // Pattern 2: Remove standalone "onghua" prefix (common artifact)
        if trimmed.starts_with("onghua") {
            return trimmed["onghua".len()..].trim().to_string();
        }
        
        // Pattern 3: Remove "nghua" if it's at the start (partial artifact)
        if trimmed.starts_with("nghua") {
            return trimmed["nghua".len()..].trim().to_string();
        }
        
        // Pattern 4: Check for any XML-like tags at the start
        if trimmed.starts_with("<") {
            if let Some(end_pos) = trimmed.find(">") {
                // Check if this looks like an ASR tag
                let tag_content = &trimmed[1..end_pos];
                if tag_content.contains("asr") || tag_content.contains("text") {
                    return trimmed[end_pos + 1..].trim().to_string();
                }
            }
        }
        
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::text::clean_qwen_asr_output;

    #[test]
    fn test_clean_qwen_asr_output_with_full_artifact() {
        let input = "onghua<asr_text>这是中文转写结果";
        assert_eq!(clean_qwen_asr_output(input), "这是中文转写结果");
    }

    #[test]
    fn test_clean_qwen_asr_output_with_just_tag() {
        let input = "<asr_text>This is English";
        assert_eq!(clean_qwen_asr_output(input), "This is English");
    }

    #[test]
    fn test_clean_qwen_asr_output_with_onghua_only() {
        let input = "onghuaSome text here";
        assert_eq!(clean_qwen_asr_output(input), "Some text here");
    }

    #[test]
    fn test_clean_qwen_asr_output_clean_text() {
        let input = "这是干净的转写结果";
        assert_eq!(clean_qwen_asr_output(input), "这是干净的转写结果");
    }

    #[test]
    fn test_clean_qwen_asr_output_with_whitespace() {
        let input = "  onghua<asr_text>  中文结果  ";
        assert_eq!(clean_qwen_asr_output(input), "中文结果");
    }
}

/// Opens macOS System Settings to a specific privacy preference pane
#[cfg(target_os = "macos")]
#[tauri::command]
pub async fn open_system_settings(preference_pane: String) -> Result<(), String> {
    use std::process::Command;

    // Construct the URL for System Settings
    let url = format!("x-apple.systempreferences:com.apple.preference.security?{}", preference_pane);

    // Use the 'open' command on macOS to open the URL
    Command::new("open")
        .arg(&url)
        .spawn()
        .map_err(|e| format!("Failed to open system settings: {}", e))?;

    Ok(())
} 