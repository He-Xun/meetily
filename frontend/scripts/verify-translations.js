/**
 * 验证翻译文件
 * 检查所有语言文件是否与 en.json 有相同的键结构和数量
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

// 递归获取所有键路径
function getAllKeys(obj, prefix = '') {
  const keys = [];
  
  for (const key of Object.keys(obj)) {
    const currentKey = prefix ? `${prefix}.${key}` : key;
    
    if (typeof obj[key] === 'object' && obj[key] !== null && !Array.isArray(obj[key])) {
      keys.push(...getAllKeys(obj[key], currentKey));
    } else {
      keys.push(currentKey);
    }
  }
  
  return keys;
}

// 统计键数量
function countKeys(obj) {
  let count = 0;
  
  for (const key of Object.keys(obj)) {
    if (typeof obj[key] === 'object' && obj[key] !== null && !Array.isArray(obj[key])) {
      count += countKeys(obj[key]);
    } else {
      count++;
    }
  }
  
  return count;
}

// 主函数
function main() {
  log('cyan', '翻译文件验证工具');
  log('cyan', '================\n');

  // 加载 en.json 作为基准
  const enPath = path.join(MESSAGES_DIR, 'en.json');
  const enData = loadJson(enPath);
  
  if (!enData) {
    log('red', '无法加载 en.json，退出');
    return;
  }

  const enKeys = getAllKeys(enData);
  const enKeyCount = enKeys.length;
  
  log('blue', `en.json: ${enKeyCount} 个键\n`);

  let allValid = true;

  // 验证每个语言文件
  for (const lang of LANGUAGES) {
    const filePath = path.join(MESSAGES_DIR, `${lang}.json`);
    log('blue', `\n验证 ${lang}.json...`);
    
    const data = loadJson(filePath);
    if (!data) {
      log('red', `  ✗ 无法加载 ${lang}.json`);
      allValid = false;
      continue;
    }

    const langKeys = getAllKeys(data);
    const langKeyCount = langKeys.length;
    
    log('cyan', `  键数量: ${langKeyCount}/${enKeyCount}`);

    // 检查缺失的键
    const missingKeys = enKeys.filter(key => !langKeys.includes(key));
    if (missingKeys.length > 0) {
      log('red', `  ✗ 缺失 ${missingKeys.length} 个键:`);
      missingKeys.slice(0, 5).forEach(key => log('red', `    - ${key}`));
      if (missingKeys.length > 5) {
        log('red', `    ... 还有 ${missingKeys.length - 5} 个`);
      }
      allValid = false;
    }

    // 检查多余的键
    const extraKeys = langKeys.filter(key => !enKeys.includes(key));
    if (extraKeys.length > 0) {
      log('yellow', `  ⚠ 多余 ${extraKeys.length} 个键:`);
      extraKeys.slice(0, 5).forEach(key => log('yellow', `    - ${key}`));
      if (extraKeys.length > 5) {
        log('yellow', `    ... 还有 ${extraKeys.length - 5} 个`);
      }
    }

    // 检查键顺序
    const enOrder = enKeys.join(',');
    const langOrder = langKeys.join(',');
    if (enOrder !== langOrder) {
      log('yellow', `  ⚠ 键顺序与 en.json 不一致`);
    }

    if (missingKeys.length === 0 && extraKeys.length === 0 && enOrder === langOrder) {
      log('green', `  ✓ 验证通过`);
    }
  }

  log('cyan', '\n\n验证完成！');
  if (allValid) {
    log('green', '所有语言文件验证通过 ✓');
  } else {
    log('red', '发现一些问题，请检查上方输出');
  }
}

main();
