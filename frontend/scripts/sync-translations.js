/**
 * 同步翻译文件
 * 1. 以 en.json 为基准，统一所有语言文件的键顺序
 * 2. 同步缺失的键（使用 en.json 的值作为占位符）
 * 3. 移除其他语言文件中多余的键
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

// 递归同步对象结构（保持 en.json 的顺序）
function syncObjectStructure(enObj, targetObj, path = '') {
  if (typeof enObj !== 'object' || enObj === null) {
    return enObj;
  }

  if (Array.isArray(enObj)) {
    // 对于数组，如果目标有值则保留，否则使用 en 的值
    if (Array.isArray(targetObj)) {
      return targetObj.map((item, index) => 
        syncObjectStructure(enObj[index], item, `${path}[${index}]`)
      );
    }
    return [...enObj];
  }

  const result = {};
  
  // 按照 en.json 的顺序遍历键
  for (const key of Object.keys(enObj)) {
    const currentPath = path ? `${path}.${key}` : key;
    
    if (typeof enObj[key] === 'object' && enObj[key] !== null && !Array.isArray(enObj[key])) {
      // 嵌套对象，递归同步
      const targetValue = targetObj && typeof targetObj === 'object' ? targetObj[key] : undefined;
      result[key] = syncObjectStructure(enObj[key], targetValue, currentPath);
    } else {
      // 叶子节点（字符串值）
      if (targetObj && typeof targetObj === 'object' && key in targetObj) {
        // 目标文件有这个键，保留其值
        result[key] = targetObj[key];
      } else {
        // 目标文件没有这个键，使用 en.json 的值
        result[key] = enObj[key];
        log('yellow', `  添加缺失键: ${currentPath}`);
      }
    }
  }

  return result;
}

// 统计键数量
function countKeys(obj, prefix = '') {
  let count = 0;
  
  for (const key of Object.keys(obj)) {
    const currentKey = prefix ? `${prefix}.${key}` : key;
    
    if (typeof obj[key] === 'object' && obj[key] !== null && !Array.isArray(obj[key])) {
      count += countKeys(obj[key], currentKey);
    } else {
      count++;
    }
  }
  
  return count;
}

// 主函数
function main() {
  log('cyan', '翻译文件同步工具');
  log('cyan', '================\n');

  // 加载 en.json 作为基准
  const enPath = path.join(MESSAGES_DIR, 'en.json');
  const enData = loadJson(enPath);
  
  if (!enData) {
    log('red', '无法加载 en.json，退出');
    return;
  }

  const enKeyCount = countKeys(enData);
  log('blue', `已加载 en.json (${enKeyCount} 个键)\n`);

  // 处理每个语言文件
  for (const lang of LANGUAGES) {
    const filePath = path.join(MESSAGES_DIR, `${lang}.json`);
    log('blue', `\n处理 ${lang}.json...`);
    
    const data = loadJson(filePath);
    if (!data) {
      log('red', `  跳过 ${lang}.json (加载失败)`);
      continue;
    }

    const beforeKeyCount = countKeys(data);
    log('cyan', `  同步前: ${beforeKeyCount} 个键`);

    // 同步结构
    const syncedData = syncObjectStructure(enData, data);
    const afterKeyCount = countKeys(syncedData);
    log('cyan', `  同步后: ${afterKeyCount} 个键`);

    // 保存同步后的文件
    if (saveJson(filePath, syncedData)) {
      log('green', `  ✓ 已保存同步后的 ${lang}.json`);
    } else {
      log('red', `  ✗ 保存 ${lang}.json 失败`);
    }
  }

  log('cyan', '\n\n同步完成！');
  log('blue', '所有语言文件现在具有相同的键顺序和结构');
}

main();
