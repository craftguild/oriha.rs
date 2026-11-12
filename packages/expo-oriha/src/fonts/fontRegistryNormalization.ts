// Registry, installed-font, file and URL validation and normalization.
import type { FontRegistryManifest } from "./fontRegistryTypes";
import type { FontFamilySpec } from "../types";
import { normalizeInstalledFontFileRecord } from "./fontRegistryStorage";
import { DEFAULT_STORAGE_NAMESPACE, REGISTRY_MANIFEST_FILE } from "./fontRegistryModel";
import type {
  OrihaFontRegistryConfig,
  NormalizedFontRegistryConfig,
  OrihaInstallableFont,
  OrihaFontFile,
  FontRegistryStorage,
  OrihaInstalledFont,
  OrihaInstalledFontFile,
  OrihaFontKind,
} from "./fontRegistryModel";

export function normalizeConfig(
  config?: OrihaFontRegistryConfig,
): NormalizedFontRegistryConfig {
  return {
    baseUrl: config?.baseUrl,
    storageNamespace: sanitizeNamespace(
      config?.storageNamespace ?? DEFAULT_STORAGE_NAMESPACE,
    ),
  };
}

export function normalizeRegistryManifest(
  manifest: FontRegistryManifest | unknown[],
  baseUrl?: string,
): OrihaInstallableFont[] {
  const fonts = Array.isArray(manifest)
    ? manifest
    : Array.isArray(manifest.fonts)
      ? manifest.fonts
      : [];
  return fonts
    .map((font) => normalizeInstallableFont(font, baseUrl))
    .filter((font): font is OrihaInstallableFont => font !== null);
}

export function normalizeInstallableFont(
  value: unknown,
  baseUrl?: string,
): OrihaInstallableFont | null {
  if (!value || typeof value !== "object") {
    return null;
  }
  const font = value as Partial<OrihaInstallableFont>;
  if (
    !font.id ||
    !font.label ||
    !font.family ||
    !font.version ||
    !font.fontFamilySpec ||
    !Array.isArray(font.kinds) ||
    !Array.isArray(font.files)
  ) {
    return null;
  }

  const files = font.files
    .map((file) => normalizeInstallableFile(file, baseUrl))
    .filter((file): file is OrihaFontFile => file !== null);
  if (files.length === 0) {
    return null;
  }

  return {
    id: font.id,
    label: font.label,
    family: font.family,
    kinds: normalizeFontKinds(font.kinds),
    fontFamilySpec: normalizeFontFamilySpec(font.fontFamilySpec),
    version: String(font.version),
    files,
    description: font.description,
  };
}

export function normalizeInstalledFont(
  value: unknown,
  storage: FontRegistryStorage,
): OrihaInstalledFont | null {
  if (!value || typeof value !== "object") {
    return null;
  }
  const font = value as Partial<OrihaInstalledFont>;
  if (
    !font.id ||
    !font.label ||
    !font.family ||
    !font.version ||
    !font.installedAt ||
    !font.fontFamilySpec ||
    !Array.isArray(font.kinds) ||
    !Array.isArray(font.files)
  ) {
    return null;
  }

  const files = font.files
    .map((file) => normalizeInstalledFile(file, storage))
    .filter((file): file is OrihaInstalledFontFile => file !== null);
  if (files.length === 0) {
    return null;
  }

  return {
    id: font.id,
    label: font.label,
    family: font.family,
    kinds: normalizeFontKinds(font.kinds),
    fontFamilySpec: normalizeFontFamilySpec(font.fontFamilySpec),
    installedAt: font.installedAt,
    version: font.version,
    files,
    description: font.description,
  };
}

export function normalizeInstallableFile(
  value: unknown,
  baseUrl?: string,
): OrihaFontFile | null {
  if (!value || typeof value !== "object") {
    return null;
  }
  const file = value as Partial<OrihaFontFile> & { path?: string };
  const rawUrl = file.url ?? file.path;
  if (!rawUrl || !file.sha256) {
    return null;
  }
  return {
    url: resolveFontFileUrl(rawUrl, baseUrl),
    fileName: file.fileName,
    sha256: normalizeSha256(file.sha256),
    weight: file.weight,
    style: file.style,
    format: file.format,
    unicodeRange: file.unicodeRange,
  };
}

export function normalizeInstalledFile(
  value: unknown,
  storage: FontRegistryStorage,
): OrihaInstalledFontFile | null {
  const file = normalizeInstalledFontFileRecord(value, {
    storageRootUri: storage.rootDir,
    storageNamespace: storage.config.storageNamespace,
  });
  if (!file) {
    return null;
  }
  return {
    uri: file.uri,
    relativePath: file.relativePath,
    fileName: file.fileName,
    sha256: normalizeSha256(file.sha256),
    weight: file.weight,
    style: file.style,
    format: file.format,
    unicodeRange: file.unicodeRange,
  };
}

export function normalizeFontFamilySpec(value: FontFamilySpec): FontFamilySpec {
  return {
    family: value.family,
    fallbacks: value.fallbacks,
  };
}

export function normalizeFontKinds(kinds: readonly unknown[]): OrihaFontKind[] {
  const normalized = kinds.filter(
    (kind): kind is OrihaFontKind =>
      kind === "heading" || kind === "body" || kind === "code",
  );
  return normalized.length > 0 ? normalized : ["body"];
}

export function resolveRegistryManifestUrl(baseUrl?: string): string {
  if (!baseUrl) {
    throw new Error("fontRegistry.baseUrl is required to refresh fonts");
  }
  if (/\.json(?:[?#].*)?$/i.test(baseUrl)) {
    return baseUrl;
  }
  return new URL(REGISTRY_MANIFEST_FILE, ensureTrailingSlash(baseUrl)).toString();
}

export function resolveFontFileUrl(rawUrl: string, baseUrl?: string): string {
  if (/^[a-z][a-z\d+.-]*:/i.test(rawUrl)) {
    return rawUrl;
  }
  if (!baseUrl) {
    throw new Error(`Relative font file URL requires fontRegistry.baseUrl: ${rawUrl}`);
  }
  return new URL(rawUrl, resolveFontAssetBaseUrl(baseUrl)).toString();
}

export function resolveFontAssetBaseUrl(baseUrl: string): string {
  if (/\.json(?:[?#].*)?$/i.test(baseUrl)) {
    return new URL(".", baseUrl).toString();
  }
  return ensureTrailingSlash(baseUrl);
}

export function safeFileName(value: string): string {
  const lastSegment = value.split(/[/?#]/).filter(Boolean).pop() ?? "font";
  const safe = lastSegment.replace(/[^A-Za-z0-9._-]/g, "-");
  return safe.length > 0 ? safe : "font";
}

export function sanitizeNamespace(value: string): string {
  const safe = value.trim().replace(/[^A-Za-z0-9._-]/g, "-");
  return safe.length > 0 ? safe : DEFAULT_STORAGE_NAMESPACE;
}

export function ensureTrailingSlash(value: string): string {
  return value.endsWith("/") ? value : `${value}/`;
}

export function normalizeSha256(value: string): string {
  return value.toLowerCase().replace(/^sha256[:-]/, "");
}
