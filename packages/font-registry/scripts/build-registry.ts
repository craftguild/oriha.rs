import { createHash } from "node:crypto";
import { mkdir, readFile, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { RegistryProvenance } from "./registry-provenance.js";
import type {
  FontRegistryConfig,
  FontRegistryConfigFont,
  FontRegistryFontKind,
  FontRegistryManifest,
} from "../src/registry-types.js";

const PACKAGE_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const VALID_KINDS = new Set<FontRegistryFontKind>(["heading", "body", "code"]);

type BuildArgs = {
  config: string | null;
  dist: string | null;
  cssFixtureDir: string | null;
  fontFixtureDir: string | null;
};

type FontFace = {
  url: string;
  weight: string;
  style: string;
  unicodeRange: string | null;
};

async function main(): Promise<void> {
  const args = parseArgs(process.argv.slice(2));
  const configPath = path.resolve(PACKAGE_ROOT, args.config ?? "registry.config.json");
  const distDir = path.resolve(PACKAGE_ROOT, args.dist ?? "dist");
  const configBytes = await readFile(configPath);
  const config: unknown = JSON.parse(configBytes.toString("utf8"));

  validateConfig(config);

  await mkdir(distDir, { recursive: true });
  for (const name of ["fonts", "sources", "index.json", "provenance.json"]) {
    await rm(path.join(distDir, name), { recursive: true, force: true });
  }
  await mkdir(path.join(distDir, "fonts"), { recursive: true });

  const fonts = [];
  // Local fixture bytes are deliberately outside production font provenance.
  const provenance = args.cssFixtureDir || args.fontFixtureDir
    ? null : new RegistryProvenance(PACKAGE_ROOT, distDir, createHash("sha256").update(configBytes).digest("hex"));
  for (const font of config.fonts) {
    const cssUrl = makeCss2Url(config, font);
    const css = await loadCss(font, cssUrl, args.cssFixtureDir);
    const faces = parseFontFaces(css);

    if (faces.length === 0) {
      throw new Error(`No @font-face blocks found for ${font.id}.`);
    }

    const files = [];
    const sourceFiles = [];
    for (const [index, face] of faces.entries()) {
      const sourceUrl = resolveSourceUrl(face.url, cssUrl, font, args);
      const payload = await loadFontPayload(sourceUrl);
      const sha256 = createHash("sha256").update(payload).digest("hex");
      const filename = outputFilename(sourceUrl, index, sha256);
      const relativeUrl = `fonts/${font.id}/${filename}`;
      const outputPath = path.join(distDir, relativeUrl);

      await mkdir(path.dirname(outputPath), { recursive: true });
      await writeFile(outputPath, payload);
      sourceFiles.push({ source: sourceUrl, url: relativeUrl, sha256, size: payload.byteLength });

      files.push({
        url: relativeUrl,
        weight: face.weight,
        style: face.style,
        format: "woff2",
        unicodeRange: face.unicodeRange,
        size: payload.byteLength,
        sha256,
      });
    }

    await provenance?.retain(font.id, font.family, cssUrl, css, sourceFiles);

    fonts.push({
      id: font.id,
      label: font.label,
      family: font.family,
      kinds: font.kinds,
      version: String(config.version),
      fontFamilySpec: {
        family: quoteFontFamily(font.family),
        fallbacks: fallbackFamiliesForKinds(font.kinds),
      },
      files,
    });
  }

  const registry: FontRegistryManifest = {
    version: config.version,
    generatedAt: new Date().toISOString(),
    source: {
      provider: config.source.provider,
      policy: config.source.policy,
    },
    fonts,
  };

  await writeFile(path.join(distDir, "index.json"), `${JSON.stringify(registry, null, 2)}\n`);
  await provenance?.finish();
  console.log(`wrote ${path.relative(PACKAGE_ROOT, path.join(distDir, "index.json"))}`);
}

function parseArgs(argv: string[]): BuildArgs {
  const result: BuildArgs = {
    config: null,
    dist: null,
    cssFixtureDir: null,
    fontFixtureDir: null,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--config") {
      result.config = requireValue(argv, ++index, "--config");
    } else if (arg.startsWith("--config=")) {
      result.config = arg.slice("--config=".length);
    } else if (arg === "--dist") {
      result.dist = requireValue(argv, ++index, "--dist");
    } else if (arg.startsWith("--dist=")) {
      result.dist = arg.slice("--dist=".length);
    } else if (arg === "--css-fixture-dir") {
      result.cssFixtureDir = requireValue(argv, ++index, "--css-fixture-dir");
    } else if (arg.startsWith("--css-fixture-dir=")) {
      result.cssFixtureDir = arg.slice("--css-fixture-dir=".length);
    } else if (arg === "--font-fixture-dir") {
      result.fontFixtureDir = requireValue(argv, ++index, "--font-fixture-dir");
    } else if (arg.startsWith("--font-fixture-dir=")) {
      result.fontFixtureDir = arg.slice("--font-fixture-dir=".length);
    } else {
      throw new Error(`Unknown argument "${arg}".`);
    }
  }

  return result;
}

function requireValue(argv: string[], index: number, flag: string): string {
  const value = argv[index];
  if (!value || value.startsWith("--")) {
    throw new Error(`${flag} requires a value.`);
  }
  return value;
}

function validateConfig(config: unknown): asserts config is FontRegistryConfig {
  if (!isRecord(config)) {
    throw new Error("registry config must be an object.");
  }

  if (!isRecord(config.source)) {
    throw new Error("registry source must be an object.");
  }

  if (config.source?.provider !== "google-fonts-css2") {
    throw new Error("registry source.provider must be google-fonts-css2.");
  }

  if (typeof config.source.baseUrl !== "string") {
    throw new Error("registry source.baseUrl must be a string.");
  }

  if (!isRecord(config.source.policy)) {
    throw new Error("registry source.policy must be an object.");
  }

  if (config.source?.policy?.allowTextParameter !== false) {
    throw new Error("registry source.policy.allowTextParameter must be false.");
  }

  if (config.source?.policy?.allowPerDocumentSubset !== false) {
    throw new Error("registry source.policy.allowPerDocumentSubset must be false.");
  }

  if (!Array.isArray(config.fonts)) {
    throw new Error("registry fonts must be an array.");
  }

  const seen = new Set<string>();
  for (const font of config.fonts) {
    if (!isRecord(font)) {
      throw new Error("registry font entries must be objects.");
    }

    if (!isRecord(font.source)) {
      throw new Error(`Invalid font entry ${String(font.id ?? "(missing id)")}.`);
    }

    if (
      typeof font.id !== "string" ||
      typeof font.label !== "string" ||
      typeof font.family !== "string" ||
      typeof font.source.css2Family !== "string"
    ) {
      throw new Error(`Invalid font entry ${font.id ?? "(missing id)"}.`);
    }

    if (seen.has(font.id)) {
      throw new Error(`Duplicate font id ${font.id}.`);
    }
    if (!/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(font.id)) {
      throw new Error(`Invalid portable font id: ${font.id}`);
    }
    seen.add(font.id);

    if (!Array.isArray(font.kinds) || font.kinds.length === 0) {
      throw new Error(`Font ${font.id} must declare at least one kind.`);
    }

    for (const kind of font.kinds) {
      if (!isFontKind(kind)) {
        throw new Error(`Font ${font.id} has invalid kind ${kind}.`);
      }
    }
  }
}

function makeCss2Url(config: FontRegistryConfig, font: FontRegistryConfigFont): string {
  const baseUrl = new URL(config.source.baseUrl);
  const family = font.source.css2Family;
  const display = encodeURIComponent(config.source.display ?? "swap");

  if (family.includes("&") || family.includes("?") || family.includes("#")) {
    throw new Error(`Invalid CSS2 family segment for ${font.id}.`);
  }

  if (family.toLowerCase().includes("text=")) {
    throw new Error(`Refusing to generate Google Fonts text= URL for ${font.id}.`);
  }

  baseUrl.search = "";
  baseUrl.hash = "";
  const url = `${baseUrl.href}?family=${family}&display=${display}`;

  if (url.toLowerCase().includes("text=")) {
    throw new Error(`Refusing to generate Google Fonts text= URL for ${font.id}.`);
  }

  return url;
}

async function loadCss(
  font: FontRegistryConfigFont,
  cssUrl: string,
  cssFixtureDir: string | null,
): Promise<string> {
  if (cssFixtureDir) {
    const fixturePath = path.resolve(PACKAGE_ROOT, cssFixtureDir, `${font.id}.css`);
    return readFile(fixturePath, "utf8");
  }

  const response = await fetch(cssUrl, {
    headers: {
      "User-Agent": "Mozilla/5.0 AppleWebKit/537.36 Chrome/123 Safari/537.36",
    },
  });

  if (!response.ok) {
    throw new Error(`Failed to fetch CSS for ${font.id}: ${response.status} ${response.statusText}`);
  }

  return response.text();
}

function parseFontFaces(css: string): FontFace[] {
  const faces: FontFace[] = [];
  const blockPattern = /@font-face\s*\{([\s\S]*?)\}/g;
  let match: RegExpExecArray | null;

  while ((match = blockPattern.exec(css)) !== null) {
    const block = match[1];
    const url = property(block, "src")?.match(/url\((['"]?)(.*?)\1\)/)?.[2];
    if (!url) {
      continue;
    }

    faces.push({
      url,
      weight: property(block, "font-weight") ?? "400",
      style: property(block, "font-style") ?? "normal",
      unicodeRange: property(block, "unicode-range") ?? null,
    });
  }

  return faces;
}

function property(block: string, name: string): string | null {
  const match = block.match(new RegExp(`${escapeRegExp(name)}\\s*:\\s*([^;]+);`, "i"));
  return match ? match[1].trim().replace(/\s+/g, " ") : null;
}

function resolveSourceUrl(
  rawUrl: string,
  cssUrl: string,
  font: FontRegistryConfigFont,
  args: BuildArgs,
): string {
  if (args.fontFixtureDir) {
    return pathToFileURL(path.resolve(PACKAGE_ROOT, args.fontFixtureDir, path.basename(rawUrl))).href;
  }

  try {
    return new URL(rawUrl, cssUrl).href;
  } catch {
    throw new Error(`Invalid font URL for ${font.id}: ${rawUrl}`);
  }
}

async function loadFontPayload(sourceUrl: string): Promise<Buffer> {
  const url = new URL(sourceUrl);

  if (url.protocol === "file:") {
    return readFile(fileURLToPath(url));
  }

  if (url.protocol !== "https:") {
    throw new Error(`Refusing non-HTTPS font URL: ${sourceUrl}`);
  }

  const response = await fetch(sourceUrl);
  if (!response.ok) {
    throw new Error(`Failed to fetch font ${sourceUrl}: ${response.status} ${response.statusText}`);
  }

  return Buffer.from(await response.arrayBuffer());
}

function outputFilename(sourceUrl: string, index: number, sha256: string): string {
  const url = new URL(sourceUrl);
  const basename = path.basename(url.pathname) || `font-${index}.woff2`;
  const extension = path.extname(basename) || ".woff2";
  const stem = path.basename(basename, extension).replace(/[^a-zA-Z0-9._-]/g, "-");
  return `${String(index + 1).padStart(3, "0")}-${stem}-${sha256.slice(0, 12)}${extension}`;
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function quoteFontFamily(family: string): string {
  return `"${String(family).replace(/\\/g, "\\\\").replace(/"/g, '\\"')}"`;
}

function fallbackFamiliesForKinds(kinds: readonly FontRegistryFontKind[]): string[] {
  return kinds.includes("code") ? ["monospace"] : ["sans-serif"];
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function isFontKind(value: unknown): value is FontRegistryFontKind {
  return typeof value === "string" && VALID_KINDS.has(value as FontRegistryFontKind);
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
});
