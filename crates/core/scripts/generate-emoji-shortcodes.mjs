#!/usr/bin/env node
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";
import https from "node:https";

const GITHUB_GEMOJI_COMMIT = "0eca75db9301421efc8710baf7a7576793ae452a";
const CLDR_JSON_VERSION = "48.2.0";
const UNICODE_EMOJI_VERSION = "16.0";
const LOCALE = "en";
const SOURCE_PRIORITY = ["github-gemoji", "unicode-cldr"];

const scriptDir = dirname(fileURLToPath(import.meta.url));
const crateDir = join(scriptDir, "..");
const dataDir = join(crateDir, "emoji-data");
const outputIndex = process.argv.indexOf("--output-dir");
if (outputIndex >= 0 && !process.argv[outputIndex + 1]) {
  throw new Error("--output-dir requires a destination directory");
}
const generatedDirectory = outputIndex >= 0
  ? process.argv[outputIndex + 1]
  : join(crateDir, "src", "emoji", "generated");
const generatedPath = join(generatedDirectory, "shortcodes.rs");

const fetchMode = process.argv.includes("--fetch");

function dataPath(path) {
  return join(dataDir, path);
}

async function download(url) {
  return new Promise((resolve, reject) => {
    https
      .get(url, (response) => {
        if (
          response.statusCode >= 300 &&
          response.statusCode < 400 &&
          response.headers.location
        ) {
          response.resume();
          download(response.headers.location).then(resolve, reject);
          return;
        }
        if (response.statusCode !== 200) {
          response.resume();
          reject(new Error(`GET ${url} failed with ${response.statusCode}`));
          return;
        }
        response.setEncoding("utf8");
        let body = "";
        response.on("data", (chunk) => {
          body += chunk;
        });
        response.on("end", () => resolve(body));
      })
      .on("error", reject);
  });
}

async function fetchSourceData() {
  const sources = [
    {
      path: `github-gemoji-${GITHUB_GEMOJI_COMMIT}/emoji.json`,
      url: `https://raw.githubusercontent.com/github/gemoji/${GITHUB_GEMOJI_COMMIT}/db/emoji.json`,
    },
    {
      path: `github-gemoji-${GITHUB_GEMOJI_COMMIT}/LICENSE`,
      url: `https://raw.githubusercontent.com/github/gemoji/${GITHUB_GEMOJI_COMMIT}/LICENSE`,
    },
    {
      path: `unicode-cldr-json-${CLDR_JSON_VERSION}/annotations/${LOCALE}/annotations.json`,
      url: `https://raw.githubusercontent.com/unicode-org/cldr-json/${CLDR_JSON_VERSION}/cldr-json/cldr-annotations-full/annotations/${LOCALE}/annotations.json`,
    },
    {
      path: `unicode-cldr-json-${CLDR_JSON_VERSION}/annotationsDerived/${LOCALE}/annotations.json`,
      url: `https://raw.githubusercontent.com/unicode-org/cldr-json/${CLDR_JSON_VERSION}/cldr-json/cldr-annotations-derived-full/annotationsDerived/${LOCALE}/annotations.json`,
    },
    {
      path: `unicode-cldr-json-${CLDR_JSON_VERSION}/LICENSE`,
      url: `https://raw.githubusercontent.com/unicode-org/cldr-json/${CLDR_JSON_VERSION}/LICENSE`,
    },
    {
      path: `unicode-emoji-${UNICODE_EMOJI_VERSION}/emoji-test.txt`,
      url: `https://unicode.org/Public/emoji/${UNICODE_EMOJI_VERSION}/emoji-test.txt`,
    },
    {
      path: `unicode-emoji-${UNICODE_EMOJI_VERSION}/LICENSE`,
      url: "https://www.unicode.org/license.txt",
    },
  ];
  for (const source of sources) {
    const body = await download(source.url);
    const target = dataPath(source.path);
    mkdirSync(dirname(target), { recursive: true });
    writeFileSync(target, `${body.trimEnd()}\n`);
    console.error(`Fetched ${source.url} -> ${relative(process.cwd(), target)}`);
  }
}

function readJson(path) {
  return JSON.parse(readFileSync(dataPath(path), "utf8"));
}

function readText(path) {
  return readFileSync(dataPath(path), "utf8");
}

function parseEmojiTest(text) {
  const emojis = new Set();
  for (const line of text.split(/\r?\n/)) {
    const match = line.match(/^([0-9A-F ]+)\s*;\s*(fully-qualified|minimally-qualified|unqualified|component)\s*#/);
    if (!match) {
      continue;
    }
    const emoji = match[1]
      .trim()
      .split(/\s+/)
      .map((codepoint) => String.fromCodePoint(parseInt(codepoint, 16)))
      .join("");
    emojis.add(emoji);
  }
  return emojis;
}

function shortcodeFromCldrTts(value) {
  return value
    .normalize("NFKD")
    .replace(/[\u0300-\u036f]/g, "")
    .toLowerCase()
    .replace(/&/g, " and ")
    .replace(/['’]/g, "")
    .replace(/[^a-z0-9]+/g, "_")
    .replace(/^_+|_+$/g, "");
}

function validateShortcode(shortcode) {
  return /^[A-Za-z0-9_+\-]+$/.test(shortcode);
}

function rustString(value) {
  let out = '"';
  for (const ch of value) {
    switch (ch) {
      case "\\":
        out += "\\\\";
        break;
      case '"':
        out += '\\"';
        break;
      case "\n":
        out += "\\n";
        break;
      case "\r":
        out += "\\r";
        break;
      case "\t":
        out += "\\t";
        break;
      default:
        out += ch;
    }
  }
  return `${out}"`;
}

function addEntry(entries, conflicts, shortcode, emoji, source) {
  if (!shortcode || !validateShortcode(shortcode)) {
    throw new Error(`Unsupported shortcode syntax from ${source}: ${shortcode}`);
  }
  const existing = entries.get(shortcode);
  if (!existing) {
    entries.set(shortcode, { emoji, source });
    return;
  }
  if (existing.emoji !== emoji) {
    conflicts.push({ shortcode, kept: existing, skipped: { emoji, source } });
  }
}

function addGithubEntries(entries, conflicts) {
  const records = readJson(`github-gemoji-${GITHUB_GEMOJI_COMMIT}/emoji.json`);
  for (const record of records) {
    if (!record.emoji) {
      continue;
    }
    for (const alias of record.aliases ?? []) {
      addEntry(entries, conflicts, alias, record.emoji, "github-gemoji");
    }
  }
}

function addCldrEntries(entries, conflicts, emojiAllowlist) {
  const annotationRoots = [
    readJson(`unicode-cldr-json-${CLDR_JSON_VERSION}/annotations/${LOCALE}/annotations.json`)
      .annotations.annotations,
    readJson(
      `unicode-cldr-json-${CLDR_JSON_VERSION}/annotationsDerived/${LOCALE}/annotations.json`,
    ).annotationsDerived.annotations,
  ];
  for (const root of annotationRoots) {
    for (const [emoji, annotation] of Object.entries(root)) {
      if (!emojiAllowlist.has(emoji) || !annotation.tts?.[0]) {
        continue;
      }
      addEntry(
        entries,
        conflicts,
        shortcodeFromCldrTts(annotation.tts[0]),
        emoji,
        "unicode-cldr",
      );
    }
  }
}

function buildEntries() {
  const entries = new Map();
  const conflicts = [];
  addGithubEntries(entries, conflicts);
  const emojiAllowlist = parseEmojiTest(
    readText(`unicode-emoji-${UNICODE_EMOJI_VERSION}/emoji-test.txt`),
  );
  addCldrEntries(entries, conflicts, emojiAllowlist);

  if (conflicts.length > 0) {
    const sample = conflicts
      .slice(0, 10)
      .map(
        (conflict) =>
          `${conflict.shortcode}: kept ${conflict.kept.emoji}/${conflict.kept.source}, skipped ${conflict.skipped.emoji}/${conflict.skipped.source}`,
      )
      .join("; ");
    console.warn(
      `Resolved ${conflicts.length} shortcode conflicts by source priority (${SOURCE_PRIORITY.join(
        " > ",
      )}). ${sample}`,
    );
  }

  return Array.from(entries.entries()).sort(([left], [right]) => {
    if (left < right) {
      return -1;
    }
    if (left > right) {
      return 1;
    }
    return 0;
  });
}

function writeGenerated(entries) {
  const rows = entries
    .map(([shortcode, { emoji }]) => `    (${rustString(shortcode)}, ${rustString(emoji)}),`)
    .join("\n");
  const output = `// @generated by scripts/generate-emoji-shortcodes.mjs. Do not edit.
// Sources:
// - github/gemoji ${GITHUB_GEMOJI_COMMIT} db/emoji.json aliases.
// - unicode-org/cldr-json ${CLDR_JSON_VERSION} ${LOCALE} annotations tts names.
// - Unicode emoji-test.txt ${UNICODE_EMOJI_VERSION} emoji allowlist.
// Source priority: ${SOURCE_PRIORITY.join(" > ")}.

#[rustfmt::skip]
pub(super) const EMOJI_SHORTCODES: &[(&str, &str)] = &[
${rows}
];
`;
  mkdirSync(dirname(generatedPath), { recursive: true });
  writeFileSync(generatedPath, output);
  console.error(
    `Generated ${entries.length} emoji shortcodes -> ${relative(process.cwd(), generatedPath)}`,
  );
}

if (fetchMode) {
  await fetchSourceData();
}

writeGenerated(buildEntries());
