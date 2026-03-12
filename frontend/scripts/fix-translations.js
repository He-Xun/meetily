const fs = require('fs');
const path = require('path');

const messagesDir = path.join(__dirname, '../src/i18n/messages');
const languages = ['en', 'zh', 'fr', 'es', 'ru', 'ar'];

// 需要添加到所有语言文件的缺失键
const missingKeys = {
  // RecordingSettings.tsx 中的硬编码
  'recordingSettings.recordingSettings': {
    en: 'Recording Settings',
    zh: '录音设置',
    fr: 'Paramètres d\'enregistrement',
    es: 'Configuración de grabación',
    ru: 'Настройки записи',
    ar: 'إعدادات التسجيل'
  },
  'recordingSettings.description': {
    en: 'Configure how your audio recordings are saved during meetings.',
    zh: '配置会议期间音频录制的保存方式。',
    fr: 'Configurez comment vos enregistrements audio sont sauvegardés pendant les réunions.',
    es: 'Configure cómo se guardan sus grabaciones de audio durante las reuniones.',
    ru: 'Настройте, как сохраняются ваши аудиозаписи во время встреч.',
    ar: 'تكوين كيفية حفظ تسجيلات الصوت أثناء الاجتماعات.'
  },
  'recordingSettings.saveLocation': {
    en: 'Save Location',
    zh: '保存位置',
    fr: 'Emplacement de sauvegarde',
    es: 'Ubicación de guardado',
    ru: 'Место сохранения',
    ar: 'موقع الحفظ'
  },
  'recordingSettings.openFolder': {
    en: 'Open Folder',
    zh: '打开文件夹',
    fr: 'Ouvrir le dossier',
    es: 'Abrir carpeta',
    ru: 'Открыть папку',
    ar: 'فتح المجلد'
  },
  'recordingSettings.fileFormat': {
    en: 'File Format: {format} files',
    zh: '文件格式：{format} 文件',
    fr: 'Format de fichier : fichiers {format}',
    es: 'Formato de archivo: archivos {format}',
    ru: 'Формат файла: файлы {format}',
    ar: 'تنسيق الملف: ملفات {format}'
  },
  'recordingSettings.fileFormatDescription': {
    en: 'Recordings are saved with timestamp: recording_YYYYMMDD_HHMMSS.{format}',
    zh: '录音以时间戳保存：recording_YYYYMMDD_HHMMSS.{format}',
    fr: 'Les enregistrements sont sauvegardés avec un horodatage : recording_YYYYMMDD_HHMMSS.{format}',
    es: 'Las grabaciones se guardan con marca de tiempo: recording_YYYYMMDD_HHMMSS.{format}',
    ru: 'Записи сохраняются с временной меткой: recording_YYYYMMDD_HHMMSS.{format}',
    ar: 'يتم حفظ التسجيلات مع الطابع الزمني: recording_YYYYMMDD_HHMMSS.{format}'
  },
  'recordingSettings.recordingStartNotification': {
    en: 'Recording Start Notification',
    zh: '录音开始通知',
    fr: 'Notification de début d\'enregistrement',
    es: 'Notificación de inicio de grabación',
    ru: 'Уведомление о начале записи',
    ar: 'إشعار بدء التسجيل'
  },
  'recordingSettings.recordingStartNotificationDescription': {
    en: 'Show reminder to inform participants when recording starts',
    zh: '显示提醒以通知参与者录音开始',
    fr: 'Afficher un rappel pour informer les participants du début de l\'enregistrement',
    es: 'Mostrar recordatorio para informar a los participantes cuando comienza la grabación',
    ru: 'Показывать напоминание для информирования участников о начале записи',
    ar: 'إظهار تذكير لإعلام المشاركين عند بدء التسجيل'
  },
  'recordingSettings.defaultAudioDevices': {
    en: 'Default Audio Devices',
    zh: '默认音频设备',
    fr: 'Périphériques audio par défaut',
    es: 'Dispositivos de audio predeterminados',
    ru: 'Устройства аудио по умолчанию',
    ar: 'أجهزة الصوت الافتراضية'
  },
  'recordingSettings.defaultAudioDevicesDescription': {
    en: 'Set your preferred microphone and system audio devices for recording. These will be automatically selected when starting new recordings.',
    zh: '设置您偏好的麦克风和系统音频设备进行录音。开始新录音时将自动选择这些设备。',
    fr: 'Définissez vos périphériques audio préférés pour l\'enregistrement. Ils seront automatiquement sélectionnés lors du démarrage de nouveaux enregistrements.',
    es: 'Configure sus dispositivos de micrófono y audio del sistema preferidos para grabar. Se seleccionarán automáticamente al iniciar nuevas grabaciones.',
    ru: 'Установите предпочитаемые микрофон и системное аудиоустройство для записи. Они будут автоматически выбраны при запуске новых записей.',
    ar: 'قم بتعيين أجهزة الميكروفون وصوت النظام المفضلة لديك للتسجيل. سيتم تحديدها تلقائيًا عند بدء تسجيلات جديدة.'
  },

  // TranscriptSettings.tsx 中的硬编码
  'settings.apiKey': {
    en: 'API Key',
    zh: 'API 密钥',
    fr: 'Clé API',
    es: 'Clave API',
    ru: 'API ключ',
    ar: 'مفتاح API'
  },

  // ImportAudioDialog.tsx 中的硬编码
  'import.selectModel': {
    en: 'Select model',
    zh: '选择模型',
    fr: 'Sélectionner le modèle',
    es: 'Seleccionar modelo',
    ru: 'Выбрать модель',
    ar: 'اختيار النموذج'
  },
  'import.loadingModels': {
    en: 'Loading models...',
    zh: '加载模型中...',
    fr: 'Chargement des modèles...',
    es: 'Cargando modelos...',
    ru: 'Загрузка моделей...',
    ar: 'جاري تحميل النماذج...'
  },

  // EmptyStateSummary.tsx 中的硬编码
  'summary.noSummaryGenerated': {
    en: 'No Summary Generated Yet',
    zh: '尚未生成摘要',
    fr: 'Aucun résumé généré pour le moment',
    es: 'Aún no se ha generado ningún resumen',
    ru: 'Резюме еще не создано',
    ar: 'لم يتم إنشاء ملخص بعد'
  },
  'summary.generateSummaryDescription': {
    en: 'Generate an AI-powered summary of your meeting transcript to get key points, action items, and decisions.',
    zh: '生成 AI 驱动的会议摘要，获取要点、行动项目和决策。',
    fr: 'Générez un résumé alimenté par l\'IA de votre transcription de réunion pour obtenir les points clés, les éléments d\'action et les décisions.',
    es: 'Genere un resumen impulsado por IA de su transcripción de la reunión para obtener puntos clave, elementos de acción y decisiones.',
    ru: 'Создайте резюме на основе ИИ для вашей стенограммы встречи, чтобы получить ключевые моменты, задачи и решения.',
    ar: 'قم بإنشاء ملخص مدعوم بالذكاء الاصطناعي لمحضر اجتماعك للحصول على النقاط الرئيسية وعناصر العمل والقرارات.'
  },
  'summary.generating': {
    en: 'Generating...',
    zh: '生成中...',
    fr: 'Génération en cours...',
    es: 'Generando...',
    ru: 'Создание...',
    ar: 'جاري الإنشاء...'
  },
  'summary.selectModelFirst': {
    en: 'Please select a model in Settings first',
    zh: '请先在设置中选择一个模型',
    fr: 'Veuillez d\'abord sélectionner un modèle dans les paramètres',
    es: 'Por favor, seleccione primero un modelo en Configuración',
    ru: 'Пожалуйста, сначала выберите модель в Настройках',
    ar: 'يرجى تحديد نموذج أولاً في الإعدادات'
  },

  // ModelSettings.tsx 中的硬编码
  'settings.testingConnection': {
    en: 'Testing Connection...',
    zh: '正在测试连接...',
    fr: 'Test de la connexion...',
    es: 'Probando conexión...',
    ru: 'Проверка соединения...',
    ar: 'جاري اختبار الاتصال...'
  }
};

// 读取 JSON 文件
function readJson(filename) {
  const content = fs.readFileSync(path.join(messagesDir, filename), 'utf8');
  return JSON.parse(content);
}

// 写入 JSON 文件
function writeJson(filename, data) {
  fs.writeFileSync(path.join(messagesDir, filename), JSON.stringify(data, null, 2) + '\n');
}

// 设置嵌套键值
function setNestedKey(obj, keyPath, value) {
  const keys = keyPath.split('.');
  let current = obj;
  for (let i = 0; i < keys.length - 1; i++) {
    if (!current[keys[i]]) {
      current[keys[i]] = {};
    }
    current = current[keys[i]];
  }
  current[keys[keys.length - 1]] = value;
}

// 检查键是否存在
function hasNestedKey(obj, keyPath) {
  const keys = keyPath.split('.');
  let current = obj;
  for (let i = 0; i < keys.length; i++) {
    if (current === null || current === undefined || typeof current !== 'object') {
      return false;
    }
    if (!(keys[i] in current)) {
      return false;
    }
    current = current[keys[i]];
  }
  return true;
}

// 主函数
function main() {
  console.log('Fixing translations...\n');

  // 处理每种语言
  languages.forEach(lang => {
    const filename = `${lang}.json`;
    const data = readJson(filename);
    let addedCount = 0;

    // 添加缺失的键
    Object.entries(missingKeys).forEach(([keyPath, translations]) => {
      if (!hasNestedKey(data, keyPath)) {
        const value = translations[lang] || translations['en']; // 如果没有特定语言翻译，使用英文
        setNestedKey(data, keyPath, value);
        addedCount++;
        console.log(`Added to ${lang}.json: ${keyPath} = ${value.substring(0, 50)}...`);
      }
    });

    // 写入文件
    writeJson(filename, data);
    
    if (addedCount > 0) {
      console.log(`✓ ${lang}.json: Added ${addedCount} missing keys\n`);
    } else {
      console.log(`✓ ${lang}.json: No missing keys\n`);
    }
  });

  console.log('Done!');
}

main();