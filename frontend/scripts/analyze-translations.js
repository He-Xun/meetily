/**
 * 翻译文件分析脚本
 * 用于检测翻译文件中的各种问题
 */

const fs = require('fs');
const path = require('path');

const LANGUAGES = ['en', 'zh', 'fr', 'es', 'ru', 'ar'];
const MESSAGES_DIR = path.join(__dirname, '../src/i18n/messages');

// 颜色输出
const colors = {
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  magenta: '\x1b[35m',
  cyan: '\x1b[36m',
  reset: '\x1b[0m'
};

function log(color, message) {
  console.log(`${colors[color]}${message}${colors.reset}`);
}

// 加载所有语言文件
function loadTranslations() {
  const translations = {};
  for (const lang of LANGUAGES) {
    const filePath = path.join(MESSAGES_DIR, `${lang}.json`);
    try {
      const content = fs.readFileSync(filePath, 'utf-8');
      translations[lang] = JSON.parse(content);
    } catch (error) {
      log('red', `Error loading ${lang}.json: ${error.message}`);
    }
  }
  return translations;
}

// 获取所有键（递归）
function getAllKeys(obj, prefix = '') {
  const keys = [];
  for (const [key, value] of Object.entries(obj)) {
    const fullKey = prefix ? `${prefix}.${key}` : key;
    keys.push(fullKey);
    if (typeof value === 'object' && value !== null) {
      keys.push(...getAllKeys(value, fullKey));
    }
  }
  return keys;
}

// 获取值（通过键路径）
function getValue(obj, keyPath) {
  const parts = keyPath.split('.');
  let current = obj;
  for (const part of parts) {
    if (current === null || current === undefined) return undefined;
    current = current[part];
  }
  return current;
}

// 检查重复键（在同一层级）
function findDuplicateKeys(obj, prefix = '') {
  const duplicates = [];
  const seen = new Set();
  const seenLower = new Set();

  for (const key of Object.keys(obj)) {
    const fullKey = prefix ? `${prefix}.${key}` : key;

    // 检查完全重复的键
    if (seen.has(key)) {
      duplicates.push({ key: fullKey, type: 'exact' });
    }
    seen.add(key);

    // 检查大小写不敏感的重复
    const lowerKey = key.toLowerCase();
    if (seenLower.has(lowerKey)) {
      duplicates.push({ key: fullKey, type: 'caseInsensitive' });
    }
    seenLower.add(lowerKey);

    // 递归检查子对象
    if (typeof obj[key] === 'object' && obj[key] !== null) {
      duplicates.push(...findDuplicateKeys(obj[key], fullKey));
    }
  }

  return duplicates;
}

// 分析键顺序差异
function analyzeKeyOrder(translations) {
  log('cyan', '\n=== 键顺序分析 ===');

  const enKeys = getAllKeys(translations.en);
  const enTopLevelKeys = Object.keys(translations.en);

  log('blue', `\n英语文件一级键顺序 (${enTopLevelKeys.length}个):`);
  enTopLevelKeys.forEach((key, i) => {
    console.log(`  ${i + 1}. ${key}`);
  });

  for (const lang of LANGUAGES.slice(1)) {
    if (!translations[lang]) continue;

    const langTopLevelKeys = Object.keys(translations[lang]);
    const differences = [];

    // 检查顺序差异
    for (let i = 0; i < Math.max(enTopLevelKeys.length, langTopLevelKeys.length); i++) {
      const enKey = enTopLevelKeys[i];
      const langKey = langTopLevelKeys[i];

      if (enKey !== langKey) {
        differences.push({
          position: i + 1,
          expected: enKey,
          actual: langKey
        });
      }
    }

    if (differences.length > 0) {
      log('yellow', `\n${lang}.json 一级键顺序差异:`);
      differences.forEach(diff => {
        console.log(`  位置 ${diff.position}: 期望 "${diff.expected}", 实际 "${diff.actual}"`);
      });
    } else {
      log('green', `\n${lang}.json 一级键顺序与英语文件一致 ✓`);
    }
  }
}

// 分析重复键
function analyzeDuplicateKeys(translations) {
  log('cyan', '\n=== 重复键分析 ===');

  for (const lang of LANGUAGES) {
    if (!translations[lang]) continue;

    const duplicates = findDuplicateKeys(translations[lang]);

    if (duplicates.length > 0) {
      log('red', `\n${lang}.json 发现 ${duplicates.length} 个重复键:`);
      duplicates.forEach(dup => {
        const typeStr = dup.type === 'exact' ? '完全重复' : '大小写不敏感重复';
        console.log(`  - ${dup.key} (${typeStr})`);
      });
    } else {
      log('green', `\n${lang}.json 没有发现重复键 ✓`);
    }
  }
}

// 分析键内容差异
function analyzeContentDifferences(translations) {
  log('cyan', '\n=== 键内容差异分析 ===');

  const enKeys = getAllKeys(translations.en);

  for (const lang of LANGUAGES.slice(1)) {
    if (!translations[lang]) continue;

    const langKeys = getAllKeys(translations[lang]);
    const enSet = new Set(enKeys);
    const langSet = new Set(langKeys);

    const missingInLang = enKeys.filter(k => !langSet.has(k));
    const extraInLang = langKeys.filter(k => !enSet.has(k));

    if (missingInLang.length > 0) {
      log('red', `\n${lang}.json 缺失 ${missingInLang.length} 个键:`);
      missingInLang.slice(0, 20).forEach(key => {
        console.log(`  - ${key}`);
      });
      if (missingInLang.length > 20) {
        console.log(`  ... 还有 ${missingInLang.length - 20} 个`);
      }
    }

    if (extraInLang.length > 0) {
      log('yellow', `\n${lang}.json 多出 ${extraInLang.length} 个键 (英语文件中没有):`);
      extraInLang.slice(0, 20).forEach(key => {
        console.log(`  - ${key}`);
      });
      if (extraInLang.length > 20) {
        console.log(`  ... 还有 ${extraInLang.length - 20} 个`);
      }
    }

    if (missingInLang.length === 0 && extraInLang.length === 0) {
      log('green', `\n${lang}.json 键结构与英语文件完全一致 ✓`);
    }
  }
}

// 检查特定重复键（根据用户反馈）
function checkSpecificDuplicates(translations) {
  log('cyan', '\n=== 特定重复键检查 ===');

  // 检查 permissionWarning 中的重复
  const permissionWarningKeys = [
    'installVirtualAudio',
    'grantScreenRecording',
    'configureAudioRouting',
    'micCheck1',
    'micCheck2',
    'micCheck3'
  ];

  for (const lang of LANGUAGES) {
    if (!translations[lang]) continue;

    const pw = translations[lang].permissionWarning;
    if (!pw) continue;

    const duplicates = [];
    const seen = new Set();

    for (const key of Object.keys(pw)) {
      // 检查是否有重复的值
      const value = pw[key];
      if (typeof value === 'string') {
        const normalized = value.toLowerCase().trim();
        if (seen.has(normalized)) {
          duplicates.push({ key, value });
        }
        seen.add(normalized);
      }
    }

    if (duplicates.length > 0) {
      log('yellow', `\n${lang}.json permissionWarning 中发现可能的重复内容:`);
      duplicates.forEach(dup => {
        console.log(`  - ${dup.key}: "${dup.value}"`);
      });
    }
  }
}

// 主函数
function main() {
  log('magenta', '翻译文件分析报告');
  log('magenta', '================');

  const translations = loadTranslations();

  // 1. 分析键顺序
  analyzeKeyOrder(translations);

  // 2. 分析重复键
  analyzeDuplicateKeys(translations);

  // 3. 分析内容差异
  analyzeContentDifferences(translations);

  // 4. 检查特定重复
  checkSpecificDuplicates(translations);

  log('cyan', '\n=== 总结 ===');
  log('blue', '建议修复步骤:');
  console.log('1. 统一所有语言文件的键顺序，以 en.json 为基准');
  console.log('2. 删除重复的键（保留第一个出现的）');
  console.log('3. 同步缺失的键到其他语言文件');
  console.log('4. 检查并修复重复的值（特别是 permissionWarning 部分）');
}

main();
