/**
 * 修复翻译文件中的重复键
 * 以 en.json 为基准，修复其他语言文件中的重复键问题
 */

const fs = require('fs');
const path = require('path');

const LANGUAGES = ['zh', 'fr', 'es', 'ru', 'ar'];
const MESSAGES_DIR = path.join(__dirname, '../src/i18n/messages');

// 颜色输出
const colors = {
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  cyan: '\x1b[36m',
  reset: '\x1b[0m'
};

function log(color, message) {
  console.log(`${colors[color]}${message}${colors.reset}`);
}

// 加载 JSON 文件
function loadJson(filePath) {
  try {
    const content = fs.readFileSync(filePath, 'utf-8');
    return JSON.parse(content);
  } catch (error) {
    log('red', `Error loading ${filePath}: ${error.message}`);
    return null;
  }
}

// 保存 JSON 文件
function saveJson(filePath, data) {
  try {
    const content = JSON.stringify(data, null, 2);
    fs.writeFileSync(filePath, content, 'utf-8');
    return true;
  } catch (error) {
    log('red', `Error saving ${filePath}: ${error.message}`);
    return false;
  }
}

// 递归移除重复键（保留第一个出现的）
function removeDuplicateKeys(obj, path = '') {
  if (typeof obj !== 'object' || obj === null) {
    return obj;
  }

  if (Array.isArray(obj)) {
    return obj.map((item, index) => removeDuplicateKeys(item, `${path}[${index}]`));
  }

  const seen = new Set();
  const result = {};
  let duplicatesFound = 0;

  for (const key of Object.keys(obj)) {
    const currentPath = path ? `${path}.${key}` : key;
    
    if (seen.has(key)) {
      log('yellow', `  删除重复键: ${currentPath}`);
      duplicatesFound++;
      continue;
    }
    
    seen.add(key);
    result[key] = removeDuplicateKeys(obj[key], currentPath);
  }

  return result;
}

// 主函数
function main() {
  log('cyan', '翻译文件重复键修复工具');
  log('cyan', '=====================\n');

  // 加载 en.json 作为基准
  const enPath = path.join(MESSAGES_DIR, 'en.json');
  const enData = loadJson(enPath);
  
  if (!enData) {
    log('red', '无法加载 en.json，退出');
    return;
  }

  log('blue', `已加载 en.json 作为基准\n`);

  // 处理每个语言文件
  for (const lang of LANGUAGES) {
    const filePath = path.join(MESSAGES_DIR, `${lang}.json`);
    log('blue', `\n处理 ${lang}.json...`);
    
    const data = loadJson(filePath);
    if (!data) {
      log('red', `  跳过 ${lang}.json (加载失败)`);
      continue;
    }

    // 移除重复键
    const cleanedData = removeDuplicateKeys(data);
    
    // 保存修复后的文件
    if (saveJson(filePath, cleanedData)) {
      log('green', `  ✓ 已保存修复后的 ${lang}.json`);
    } else {
      log('red', `  ✗ 保存 ${lang}.json 失败`);
    }
  }

  log('cyan', '\n\n修复完成！');
  log('blue', '建议：运行 verify-translations.js 验证修复结果');
}

main();
