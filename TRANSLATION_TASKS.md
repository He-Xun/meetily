# Meetily 翻译任务清单

## 第一阶段：Permission & Device 组件

### 1. PermissionWarning.tsx
- [ ] `permissions.micCheck1` = "Your microphone is connected and powered on"
- [ ] `permissions.micCheck2` = "Microphone permission is granted in System Settings"
- [ ] `permissions.micCheck3` = "No other app is exclusively using the microphone"
- [ ] `permissions.installVirtualAudio` = "Install a virtual audio device (e.g., BlackHole 2ch)"
- [ ] `permissions.grantScreenRecording` = "Grant Screen Recording permission to Meetily"
- [ ] `permissions.configureAudioRouting` = "Configure your audio routing in Audio MIDI Setup"

### 2. DeviceSelection.tsx
- [ ] `deviceSelection.audioDevices` = "Audio Devices"
- [ ] `deviceSelection.defaultMicrophone` = "Default Microphone"
- [ ] `deviceSelection.defaultSystemAudio` = "Default System Audio"
- [ ] `deviceSelection.noMicrophoneFound` = "No microphone devices found"
- [ ] `deviceSelection.noSystemAudioFound` = "No system audio devices found"
- [ ] `deviceSelection.selectMicrophone` = "Select Microphone" (placeholder)
- [ ] `deviceSelection.selectSystemAudio` = "Select System Audio" (placeholder)

---

## 第二阶段：Meeting Details 组件

### 3. MeetingDetails/SummaryPanel.tsx
- [ ] `summary.meetingSummary` = "Meeting Summary"
- [ ] `summary.keyPoints` = "Key Points"
- [ ] `summary.actionItems` = "Action Items"
- [ ] `summary.decisions` = "Decisions"
- [ ] `summary.mainTopics` = "Main Topics"
- [ ] `summary.fullSummary` = "Full Summary"

### 4. MeetingDetails/TranscriptButtonGroup.tsx
- [ ] `transcript.copy` = "Copy"
- [ ] `transcript.recording` = "Recording"
- [ ] `transcript.enhance` = "Enhance"

### 5. MeetingDetails/SummaryUpdaterButtonGroup.tsx
- [ ] `summary.save` = "Save"
- [ ] `summary.copy` = "Copy"
- [ ] `summary.find` = "Find"

### 6. MeetingDetails/RetranscribeDialog.tsx
- [ ] `retranscribe.language` = "Language"
- [ ] `retranscribe.model` = "Model"
- [ ] `retranscribe.selectLanguage` = "Select language" (placeholder)

---

## 第三阶段：UI 状态组件

### 7. ChunkProgressDisplay.tsx
- [ ] `progress.completed` = "Completed"
- [ ] `progress.processing` = "Processing"
- [ ] `progress.pending` = "Pending"
- [ ] `progress.failed` = "Failed"

### 8. UpdateNotification.tsx
- [ ] `update.updateAvailable` = "Update Available"

### 9. DownloadProgressToast.tsx
- [ ] `download.downloadComplete` = "Download complete"
- [ ] `download.downloadCancelled` = "Download cancelled"

---

## 第四阶段：导入和恢复组件

### 10. ImportAudio/ImportDropOverlay.tsx
- [ ] `import.dropAudioFile` = "Drop audio file to import"

### 11. TranscriptRecovery/TranscriptRecovery.tsx
- [ ] `recovery.recoverInterruptedMeetings` = "Recover Interrupted Meetings"
- [ ] `recovery.interruptedMeetings` = "Interrupted Meetings"
- [ ] `recovery.preview` = "Preview"
- [ ] `recovery.audioAvailable` = "Audio available"
- [ ] `recovery.noAudio` = "No audio"

### 12. DatabaseImport/LegacyDatabaseImport.tsx
- [ ] `database.browseForDatabase` = "Browse for Database"
- [ ] `database.importDatabase` = "Import Database"

---

## 第五阶段：AI 和模型组件

### 13. AISummary/index.tsx
- [ ] `aiSummary.errorGeneratingSummary` = "Error Generating Summary"
- [ ] `aiSummary.copy` = "Copy"
- [ ] `aiSummary.regenerate` = "Regenerate"
- [ ] `aiSummary.undo` = "Undo"
- [ ] `aiSummary.redo` = "Redo"
- [ ] `aiSummary.addNewSection` = "Add new section"
- [ ] `aiSummary.regenerateSummary` = "Regenerate Summary"

### 14. WhisperModelManager.tsx & ParakeetModelManager.tsx
- [ ] `models.failedToLoadModels` = "Failed to load models"
- [ ] `models.ready` = "Ready"
- [ ] `models.deleteModel` = "Delete model"
- [ ] `models.deleteModelToFreeSpace` = "Delete model to free up space"

### 15. ModelSettingsModal.tsx
- [ ] `modelSettings.searchModels` = "Search models..." (placeholder)
- [ ] `modelSettings.customEndpointPlaceholder` = "http://localhost:8000/v1" (placeholder)
- [ ] `modelSettings.customModelPlaceholder` = "gpt-4, llama-3-70b, etc." (placeholder)
- [ ] `modelSettings.ollamaPlaceholder` = "http://localhost:11434" (placeholder)

---

## 第六阶段：Onboarding 组件

### 16. onboarding/steps/WelcomeStep.tsx
- [ ] `onboarding.welcomeTitle` = "Welcome to Meetily"
- [ ] `onboarding.welcomeDescription` = "Record. Transcribe. Summarize. All on your device."

### 17. onboarding/steps/SetupOverviewStep.tsx
- [ ] `onboarding.setupOverviewTitle` = "Setup Overview"
- [ ] `onboarding.setupOverviewDescription` = "Meetily requires that you download the Transcription & Summarization AI models for the software to work."

### 18. onboarding/steps/PermissionsStep.tsx
- [ ] `onboarding.grantPermissionsTitle` = "Grant Permissions"
- [ ] `onboarding.grantPermissionsDescription` = "Meetily needs access to your microphone and system audio to record meetings"
- [ ] `onboarding.microphone` = "Microphone"
- [ ] `onboarding.microphoneDescription` = "Required to capture your voice during meetings"
- [ ] `onboarding.systemAudio` = "System Audio"
- [ ] `onboarding.systemAudioDescription` = "Click Enable to grant Audio Capture permission"

### 19. onboarding/steps/DownloadProgressStep.tsx
- [ ] `onboarding.gettingThingsReady` = "Getting things ready"
- [ ] `onboarding.downloadDescription` = "You can start using Meetily after downloading the Transcription Engine."
- [ ] `onboarding.failed` = "Failed"
- [ ] `onboarding.downloadError` = "Download Error"
- [ ] `onboarding.continueWhileFinishes` = "You can continue while this finishes"

---

## 第七阶段：其他组件

### 20. ConfirmationModel/confirmation-modal.tsx
- [ ] `confirm.deleteTitle` = "Confirm Delete"

### 21. EditableTitle.tsx
- [ ] `editable.editSectionTitle` = "Edit section title"
- [ ] `editable.deleteSection` = "Delete section"

### 22. Info.tsx
- [ ] `info.about` = "About"
- [ ] `info.aboutMeetily` = "About Meetily"

### 23. Logo.tsx
- [ ] `app.name` = "Meetily" (品牌名，可选翻译)

### 24. Shared components (sheet.tsx, dialog.tsx)
- [ ] `common.close` = "Close" (aria-label)

### 25. BlockNoteEditor/BasicBlockNoteTest.tsx
- [ ] `editor.markdownInput` = "Markdown Input"
- [ ] `editor.editorOutput` = "Editor Output"

---

## 统计

- 总计：约 70+ 个翻译键
- 组件数：约 25 个文件需要修改
- 语言文件：6 个（en, zh, fr, es, ru, ar）

## 执行顺序

1. 先更新 en.json 添加所有新键
2. 同步到其他语言文件
3. 逐个组件替换硬编码文本
4. 运行验证脚本确认
