import type { FontFamilySpec } from "../types";
import type { InstalledFontFileRuntimeRecord } from "./fontRegistryStorage";

export type InstalledFontFaceCssFile = Pick<
  InstalledFontFileRuntimeRecord,
  "fileName" | "format" | "style" | "unicodeRange" | "uri" | "weight"
>;

export type InstalledFontFaceCssFont = {
  fontFamilySpec: FontFamilySpec;
  files: readonly InstalledFontFaceCssFile[];
};

export type ReadInstalledFontFileBase64 = (
  file: InstalledFontFaceCssFile,
) => Promise<string>;

export async function buildInstalledFontFaceCssRules(
  installedFonts: readonly InstalledFontFaceCssFont[],
  selectedFontSpecs: readonly (FontFamilySpec | undefined)[] | undefined,
  readFileBase64: ReadInstalledFontFileBase64,
): Promise<string> {
  const selectedFamilies = selectedFontFamilies(selectedFontSpecs);
  const rules: string[] = [];

  for (const font of installedFonts) {
    if (selectedFamilies && !selectedFamilies.has(font.fontFamilySpec.family)) {
      continue;
    }
    for (const file of font.files) {
      rules.push(buildInstalledFontFaceRule(font, file, await readFileBase64(file)));
    }
  }

  return rules.join("\n");
}

export function buildInstalledFontFaceRule(
  font: InstalledFontFaceCssFont,
  file: InstalledFontFaceCssFile,
  base64: string,
): string {
  const mimeType = fontMimeType(file);
  return [
    "@font-face {",
    `  font-family: ${cssFontFamily(font.fontFamilySpec.family)};`,
    `  src: url("data:${mimeType};base64,${base64}")${file.format ? ` format("${escapeCssString(file.format)}")` : ""};`,
    `  font-weight: ${file.weight ?? "400"};`,
    `  font-style: ${file.style ?? "normal"};`,
    ...(file.unicodeRange ? [`  unicode-range: ${file.unicodeRange};`] : []),
    "  font-display: swap;",
    "}",
  ].join("\n");
}

export function selectedFontFamilies(
  selectedFontSpecs?: readonly (FontFamilySpec | undefined)[],
): Set<string> | null {
  return selectedFontSpecs
    ? new Set(
        selectedFontSpecs
          .map((spec) => spec?.family)
          .filter((family): family is string => Boolean(family)),
      )
    : null;
}

export function fontMimeType(file: InstalledFontFaceCssFile): string {
  const format = file.format?.trim().toLowerCase();
  if (format === "woff2") {
    return "font/woff2";
  }
  if (format === "woff") {
    return "font/woff";
  }
  if (format === "opentype" || format === "otf") {
    return "font/otf";
  }
  if (format === "truetype" || format === "ttf") {
    return "font/ttf";
  }

  const lowerFileName = file.fileName.toLowerCase();
  if (lowerFileName.endsWith(".woff2")) {
    return "font/woff2";
  }
  if (lowerFileName.endsWith(".woff")) {
    return "font/woff";
  }
  if (lowerFileName.endsWith(".otf")) {
    return "font/otf";
  }
  if (lowerFileName.endsWith(".ttf")) {
    return "font/ttf";
  }
  return "font/ttf";
}

export function cssFontFamily(family: string): string {
  const trimmed = family.trim();
  if (
    (trimmed.startsWith("\"") && trimmed.endsWith("\"")) ||
    (trimmed.startsWith("'") && trimmed.endsWith("'"))
  ) {
    return trimmed;
  }
  return `"${escapeCssString(trimmed)}"`;
}

function escapeCssString(value: string): string {
  return value.replace(/\\/g, "\\\\").replace(/"/g, '\\"');
}
