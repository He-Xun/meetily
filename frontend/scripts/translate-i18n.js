const fs = require("fs");
const axios = require("axios");
const pLimit = require("p-limit");
const path = require("path");

// ===== 配置 =====
const CONFIG = {
  sourceFile: path.join(__dirname, "../src/i18n/messages/en.json"),
  targetDir: path.join(__dirname, "../src/i18n/messages"),

  languages: {
    zh: "Chinese (Simplified)",
    fr: "French",
    ru: "Russian",
    es: "Spanish",
    ar: "Arabic"
  },

  model: process.env.LLM_MODEL || "qwen-turbo",
  baseURL: process.env.LLM_BASE_URL || "https://coding.dashscope.aliyuncs.com/v1",
  apiKey: process.env.LLM_API_KEY || "sk-sp-8c0e5abe4b2649639071cf673c1fca62",

  batchSize: 20,
  concurrency: 2
};

// ===== HTTP =====
const client = axios.create({
  baseURL: CONFIG.baseURL,
  headers: {
    Authorization: `Bearer ${CONFIG.apiKey}`,
    "Content-Type": "application/json"
  },
  timeout: 60000
});

// ===== JSON 扁平化 =====
function flatten(obj, prefix = "", res = {}) {
  for (const k in obj) {
    const v = obj[k];
    const key = prefix ? `${prefix}.${k}` : k;

    if (typeof v === "object" && v !== null && !Array.isArray(v)) {
      flatten(v, key, res);
    } else {
      res[key] = v;
    }
  }
  return res;
}

// ===== JSON 还原 =====
function unflatten(data) {
  const result = {};

  for (const k in data) {
    const keys = k.split(".");
    let cur = result;

    keys.forEach((part, i) => {
      if (i === keys.length - 1) {
        cur[part] = data[k];
      } else {
        cur[part] = cur[part] || {};
        cur = cur[part];
      }
    });
  }

  return result;
}

// ===== LLM 翻译 =====
async function translateBatch(texts, lang) {
  const prompt = `Translate the following UI texts into ${lang}. Return ONLY a JSON array of translated strings. Keep placeholders like {name} {count} {formats}.

Texts to translate:
${texts.map((t, i) => `${i + 1}. ${t}`).join('\n')}

Return format example:
["translation 1", "translation 2", ...]
`;

  const res = await client.post("/chat/completions", {
    model: CONFIG.model,
    temperature: 0.1,
    messages: [
      { role: "system", content: "You are a professional i18n translator. Return ONLY valid JSON arrays." },
      { role: "user", content: prompt }
    ]
  });

  let text = res.data.choices[0].message.content.trim();
  
  // Remove markdown code blocks if present
  text = text.replace(/```json\s*/g, '').replace(/```\s*/g, '').trim();

  try {
    return JSON.parse(text);
  } catch {
    const match = text.match(/\[.*\]/s);
    if (match) return JSON.parse(match[0]);
    throw new Error("Translation parse failed");
  }
}

// ===== 主流程 =====
async function processLanguage(langCode, langName, sourceFlat) {
  const file = path.join(CONFIG.targetDir, `${langCode}.json`);

  let target = {};
  if (fs.existsSync(file)) {
    const content = fs.readFileSync(file, "utf8");
    if (content.trim()) {
      try {
        target = flatten(JSON.parse(content));
      } catch (e) {
        console.error(`${langCode}: failed to parse existing file, starting fresh`);
      }
    }
  }

  const missingKeys = [];
  const missingTexts = [];

  for (const k in sourceFlat) {
    if (!target[k]) {
      missingKeys.push(k);
      missingTexts.push(sourceFlat[k]);
    }
  }

  if (missingTexts.length === 0) {
    console.log(`${langCode}: nothing to translate`);
    return;
  }

  console.log(`${langCode}: ${missingTexts.length} new texts`);

  const batches = [];
  for (let i = 0; i < missingTexts.length; i += CONFIG.batchSize) {
    batches.push(missingTexts.slice(i, i + CONFIG.batchSize));
  }

  const limit = pLimit(CONFIG.concurrency);
  const translated = [];

  await Promise.all(
    batches.map((batch, i) =>
      limit(async () => {
        console.log(`${langCode} batch ${i + 1}/${batches.length}`);
        const res = await translateBatch(batch, langName);
        translated.push(...res);
      })
    )
  );

  missingKeys.forEach((k, i) => {
    target[k] = translated[i];
  });

  const result = unflatten(target);

  fs.writeFileSync(file, JSON.stringify(result, null, 2));

  console.log(`${langCode} updated`);
}

// ===== MAIN =====
async function main() {
  if (!CONFIG.apiKey) {
    console.error("Missing LLM_API_KEY");
    process.exit(1);
  }

  const source = JSON.parse(fs.readFileSync(CONFIG.sourceFile, "utf8"));
  const sourceFlat = flatten(source);

  console.log(`Source texts: ${Object.keys(sourceFlat).length}`);

  for (const [code, name] of Object.entries(CONFIG.languages)) {
    await processLanguage(code, name, sourceFlat);
  }

  console.log("All languages updated.");
}

main();
