export const VENDOR_STORAGE_DIR = "co.progressiveworks.expo-oriha";
export const FONT_FILES_DIR = "files";

export type InstalledFontFileRuntimeRecord = {
  uri: string;
  relativePath: string;
  fileName: string;
  sha256: string;
  weight?: string | number;
  style?: string;
  format?: string;
  unicodeRange?: string | null;
  size?: number;
};

export type StoredInstalledFontFile = Omit<InstalledFontFileRuntimeRecord, "uri">;

export type InstalledFontLike<
  File extends { uri?: string; relativePath?: string } = InstalledFontFileRuntimeRecord,
> = {
  files: readonly File[];
};

export type NormalizeInstalledFontFileOptions = {
  storageRootUri: string;
  storageNamespace: string;
};

export function buildFontRelativeDir(fontId: string): string {
  return `${FONT_FILES_DIR}/${encodeURIComponent(fontId)}/`;
}

export function buildFontRelativePath(fontId: string, fileName: string): string {
  return `${buildFontRelativeDir(fontId)}${fileName}`;
}

export function resolveFontFileUri(
  storageRootUri: string,
  relativePath: string,
): string {
  return `${ensureTrailingSlash(storageRootUri)}${relativePath}`;
}

export function normalizeInstalledFontFileRecord(
  value: unknown,
  options: NormalizeInstalledFontFileOptions,
): InstalledFontFileRuntimeRecord | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  const file = value as Partial<InstalledFontFileRuntimeRecord>;
  if (!file.fileName || !file.sha256) {
    return null;
  }

  const relativePath =
    typeof file.relativePath === "string" && file.relativePath.length > 0
      ? normalizeStoredFontRelativePath(file.relativePath)
      : typeof file.uri === "string"
        ? relativePathFromLegacyFontUri(file.uri, options.storageNamespace)
        : null;

  if (!relativePath) {
    return null;
  }

  return {
    uri: resolveFontFileUri(options.storageRootUri, relativePath),
    relativePath,
    fileName: file.fileName,
    sha256: file.sha256,
    weight: file.weight,
    style: file.style,
    format: file.format,
    unicodeRange: file.unicodeRange,
    size: file.size,
  };
}

export function serializeInstalledFontForIndex<
  Font extends InstalledFontLike<InstalledFontFileRuntimeRecord>,
>(
  font: Font,
): Omit<Font, "files"> & { files: StoredInstalledFontFile[] } {
  return {
    ...font,
    files: font.files.map(({ uri: _uri, ...file }) => file),
  };
}

export function installedFontIndexEntryNeedsRewrite(value: unknown): boolean {
  if (!value || typeof value !== "object") {
    return true;
  }

  const font = value as { files?: unknown };
  if (!Array.isArray(font.files)) {
    return true;
  }

  return font.files.some((file) => {
    if (!file || typeof file !== "object") {
      return true;
    }
    const installedFile = file as { relativePath?: unknown; uri?: unknown };
    return (
      typeof installedFile.uri === "string" ||
      typeof installedFile.relativePath !== "string" ||
      normalizeStoredFontRelativePath(installedFile.relativePath) === null
    );
  });
}

export function normalizeStoredFontRelativePath(value: string): string | null {
  const path = value;
  const segments = path.split("/");
  if (
    !path ||
    !path.startsWith(`${FONT_FILES_DIR}/`) ||
    segments.some((segment) => !segment || segment === "." || segment === "..") ||
    path.startsWith("file:") ||
    /^[a-z][a-z\d+.-]*:/i.test(path)
  ) {
    return null;
  }
  return path;
}

export function relativePathFromLegacyFontUri(
  uri: string,
  storageNamespace: string,
): string | null {
  const storageMarker = `/${VENDOR_STORAGE_DIR}/${storageNamespace}/`;
  const markerIndex = uri.indexOf(storageMarker);
  if (markerIndex < 0) {
    return null;
  }
  return normalizeStoredFontRelativePath(uri.slice(markerIndex + storageMarker.length));
}

function ensureTrailingSlash(value: string): string {
  return value.endsWith("/") ? value : `${value}/`;
}
