import * as FileSystem from "expo-file-system/legacy";
import type { FontRegistryManifest } from "./fontRegistryTypes";
import type { FontFamilySpec } from "../types";
import {
  FONT_FILES_DIR,
  VENDOR_STORAGE_DIR,
  buildFontRelativeDir,
  buildFontRelativePath,
  installedFontIndexEntryNeedsRewrite,
  serializeInstalledFontForIndex,
} from "./fontRegistryStorage";
import {
  deleteEntryIfExists,
  ensureDirectoryEntry,
  fileEntryExists,
  readJsonEntry,
  readPrunedIndexEntries,
  writeJsonEntry,
} from "./fontRegistryStoragePort";
import type { FontRegistryStorageEncoding, FontRegistryStoragePort } from "./fontRegistryStoragePort";
import { buildInstalledFontFaceCssRules } from "./fontRegistryCss";
import { INSTALLED_INDEX_FILE, REGISTRY_CACHE_FILE } from "./fontRegistryModel";
import type {
  OrihaFontRegistryConfig,
  OrihaInstalledFont,
  OrihaInstallableFont,
  FontRegistryCache,
  OrihaInstallFontOptions,
  OrihaInstalledFontFile,
  OrihaFontInstallPhase,
  FontRegistryStorage,
  InstalledFontIndex,
} from "./fontRegistryModel";
import {
  resolveRegistryManifestUrl,
  normalizeRegistryManifest,
  safeFileName,
  normalizeSha256,
  normalizeConfig,
  ensureTrailingSlash,
  normalizeInstalledFont,
  normalizeInstallableFont,
} from "./fontRegistryNormalization";
import {
  totalKnownBytes,
  progressBytes,
  overallProgress,
  fetchFontPayload,
  sha256Bytes,
  bytesToBase64,
} from "./fontRegistryTransfer";
export type {
  OrihaFontKind,
  OrihaFontFile,
  OrihaInstalledFontFile,
  OrihaInstallableFont,
  OrihaInstalledFont,
  OrihaFontRegistryConfig,
  OrihaFontInstallPhase,
  OrihaFontInstallProgress,
  OrihaInstallFontOptions,
} from "./fontRegistryModel";

const expoFontRegistryStoragePort: FontRegistryStoragePort = {
  get documentDirectory() {
    return FileSystem.documentDirectory;
  },
  getInfo(uri) {
    return FileSystem.getInfoAsync(uri);
  },
  readString(uri, options) {
    return FileSystem.readAsStringAsync(uri, toExpoStringOptions(options));
  },
  writeString(uri, value, options) {
    return FileSystem.writeAsStringAsync(uri, value, toExpoStringOptions(options));
  },
  makeDirectory(uri, options) {
    return FileSystem.makeDirectoryAsync(uri, options);
  },
  delete(uri, options) {
    return FileSystem.deleteAsync(uri, options);
  },
};

function toExpoStringOptions(options?: { encoding?: FontRegistryStorageEncoding }) {
  return options?.encoding === "base64"
    ? { encoding: FileSystem.EncodingType.Base64 }
    : undefined;
}

export async function listInstalledFonts(
  config?: OrihaFontRegistryConfig,
): Promise<OrihaInstalledFont[]> {
  const storage = requireStorage(config, expoFontRegistryStoragePort);
  return readInstalledIndex(storage, expoFontRegistryStoragePort);
}

export async function listInstallableFonts(
  config?: OrihaFontRegistryConfig,
): Promise<OrihaInstallableFont[]> {
  const storage = requireStorage(config, expoFontRegistryStoragePort);
  const [installedFonts, cachedFonts] = await Promise.all([
    readInstalledIndex(storage, expoFontRegistryStoragePort),
    readRegistryCache(storage, expoFontRegistryStoragePort),
  ]);
  const installedIds = new Set(installedFonts.map((font) => font.id));
  return cachedFonts.filter((font) => !installedIds.has(font.id));
}

export async function refreshFontRegistry(
  config?: OrihaFontRegistryConfig,
): Promise<OrihaInstallableFont[]> {
  const storage = requireStorage(config, expoFontRegistryStoragePort);
  await ensureDirectoryEntry(expoFontRegistryStoragePort, storage.rootDir);
  const manifestUrl = resolveRegistryManifestUrl(storage.config.baseUrl);
  const response = await fetch(manifestUrl);
  if (!response.ok) {
    throw new Error(
      `Font registry refresh failed with HTTP ${response.status}: ${manifestUrl}`,
    );
  }

  const manifest = normalizeRegistryManifest(
    (await response.json()) as FontRegistryManifest | unknown[],
    storage.config.baseUrl,
  );
  await writeJsonEntry(
    expoFontRegistryStoragePort,
    storage.registryCacheUri,
    {
      refreshedAt: new Date().toISOString(),
      fonts: manifest,
    } satisfies FontRegistryCache,
  );

  const installedFonts = await readInstalledIndex(storage, expoFontRegistryStoragePort);
  const installedIds = new Set(installedFonts.map((font) => font.id));
  return manifest.filter((font) => !installedIds.has(font.id));
}

export async function installFont(
  fontId: string,
  config?: OrihaFontRegistryConfig,
  options?: OrihaInstallFontOptions,
): Promise<OrihaInstalledFont> {
  const storage = requireStorage(config, expoFontRegistryStoragePort);
  const cachedFonts = await readRegistryCache(storage, expoFontRegistryStoragePort);
  const font = cachedFonts.find((entry) => entry.id === fontId);
  if (!font) {
    throw new Error(`Font is not available in the local registry cache: ${fontId}`);
  }

  const installedFonts = await readInstalledIndex(storage, expoFontRegistryStoragePort);
  const existing = installedFonts.find((entry) => entry.id === fontId);
  if (existing && existing.version === font.version) {
    return existing;
  }

  const fontRelativeDir = buildFontRelativeDir(font.id);
  const fontDir = `${storage.rootDir}${fontRelativeDir}`;
  await ensureDirectoryEntry(expoFontRegistryStoragePort, fontDir);

  const installedFiles: OrihaInstalledFontFile[] = [];
  const totalBytes = totalKnownBytes(font.files);
  let installedBytes = 0;
  for (const [fileOffset, file] of font.files.entries()) {
    const fileName = safeFileName(file.fileName ?? file.url);
    const fileIndex = fileOffset + 1;
    const relativePath = buildFontRelativePath(font.id, fileName);
    const destination = `${fontDir}${fileName}`;
    const reportProgress = (
      phase: OrihaFontInstallPhase,
      fileReceivedBytes?: number,
      fileTotalBytes?: number,
    ) => {
      options?.onProgress?.({
        fontId: font.id,
        fontLabel: font.label,
        fileIndex,
        fileCount: font.files.length,
        fileName,
        fileUrl: file.url,
        phase,
        fileReceivedBytes,
        fileTotalBytes: fileTotalBytes ?? file.size,
        installedBytes: installedBytes + progressBytes(fileReceivedBytes, fileTotalBytes ?? file.size),
        totalBytes,
        overallProgress: overallProgress(
          fileIndex,
          font.files.length,
          installedBytes,
          fileReceivedBytes,
          fileTotalBytes ?? file.size,
          totalBytes,
          phase,
        ),
      });
    };

    const payload = await fetchFontPayload(file.url, (receivedBytes, totalFileBytes) => {
      reportProgress("download", receivedBytes, totalFileBytes);
    });
    reportProgress("verify", payload.byteLength, payload.byteLength);
    const actualSha256 = await sha256Bytes(payload);
    const expectedSha256 = normalizeSha256(file.sha256);
    if (actualSha256 !== expectedSha256) {
      throw new Error(
        `Font file checksum mismatch for ${font.id}/${fileName}: expected ${expectedSha256}, got ${actualSha256}`,
      );
    }
    reportProgress("write", payload.byteLength, payload.byteLength);
    await expoFontRegistryStoragePort.writeString(
      destination,
      bytesToBase64(payload),
      { encoding: "base64" },
    );
    const info = await expoFontRegistryStoragePort.getInfo(destination);
    if (!info.exists) {
      throw new Error(`Font file write failed for ${font.id}/${fileName}: ${destination}`);
    }
    installedFiles.push({
      fileName,
      uri: destination,
      relativePath,
      sha256: expectedSha256,
      weight: file.weight,
      style: file.style,
      format: file.format,
      unicodeRange: file.unicodeRange,
    });
    reportProgress("complete", payload.byteLength, payload.byteLength);
    installedBytes += payload.byteLength;
  }

  const installedFont: OrihaInstalledFont = {
    id: font.id,
    label: font.label,
    family: font.family,
    kinds: font.kinds,
    fontFamilySpec: font.fontFamilySpec,
    installedAt: new Date().toISOString(),
    version: font.version,
    files: installedFiles,
    description: font.description,
  };

  const nextInstalledFonts = [
    ...installedFonts.filter((entry) => entry.id !== font.id),
    installedFont,
  ].sort((a, b) => a.label.localeCompare(b.label));
  await writeInstalledIndex(
    storage,
    expoFontRegistryStoragePort,
    nextInstalledFonts,
  );
  return installedFont;
}

export async function uninstallFont(
  fontId: string,
  config?: OrihaFontRegistryConfig,
): Promise<void> {
  const storage = requireStorage(config, expoFontRegistryStoragePort);
  const installedFonts = await readInstalledIndex(storage, expoFontRegistryStoragePort);
  const nextInstalledFonts = installedFonts.filter((font) => font.id !== fontId);
  if (nextInstalledFonts.length === installedFonts.length) {
    return;
  }
  await writeInstalledIndex(
    storage,
    expoFontRegistryStoragePort,
    nextInstalledFonts,
  );
  await deleteEntryIfExists(
    expoFontRegistryStoragePort,
    `${storage.filesDir}${encodeURIComponent(fontId)}/`,
  );
}

export async function buildInstalledFontFaceCss(
  installedFonts: readonly OrihaInstalledFont[],
  selectedFontSpecs?: readonly (FontFamilySpec | undefined)[],
): Promise<string> {
  return buildInstalledFontFaceCssRules(
    installedFonts,
    selectedFontSpecs,
    (file) =>
      expoFontRegistryStoragePort.readString(file.uri, {
        encoding: "base64",
      }),
  );
}

function requireStorage(
  config?: OrihaFontRegistryConfig,
  port: FontRegistryStoragePort = expoFontRegistryStoragePort,
): FontRegistryStorage {
  const documentDirectory = port.documentDirectory;
  if (!documentDirectory) {
    throw new Error("Expo FileSystem documentDirectory is not available");
  }
  const normalizedConfig = normalizeConfig(config);
  const rootDir = `${ensureTrailingSlash(documentDirectory)}${VENDOR_STORAGE_DIR}/${normalizedConfig.storageNamespace}/`;
  return {
    config: normalizedConfig,
    rootDir,
    installedIndexUri: `${rootDir}${INSTALLED_INDEX_FILE}`,
    registryCacheUri: `${rootDir}${REGISTRY_CACHE_FILE}`,
    filesDir: `${rootDir}${FONT_FILES_DIR}/`,
  };
}

async function readInstalledIndex(
  storage: FontRegistryStorage,
  port: FontRegistryStoragePort,
): Promise<OrihaInstalledFont[]> {
  return readPrunedIndexEntries({
    port,
    indexUri: storage.installedIndexUri,
    getRawEntries(index) {
      const parsed = index as InstalledFontIndex | null;
      return Array.isArray(parsed?.fonts) ? parsed.fonts : [];
    },
    normalizeEntry: (font) => normalizeInstalledFont(font, storage),
    entryExists: (font) => installedFontFilesExist(font, port),
    entryNeedsRewrite: installedFontIndexEntryNeedsRewrite,
    serializeIndex(fonts) {
      return {
        fonts: fonts.map(serializeInstalledFontForIndex),
      };
    },
  });
}

async function writeInstalledIndex(
  storage: FontRegistryStorage,
  port: FontRegistryStoragePort,
  fonts: OrihaInstalledFont[],
): Promise<void> {
  await ensureDirectoryEntry(port, storage.rootDir);
  await writeJsonEntry(port, storage.installedIndexUri, {
    fonts: fonts.map(serializeInstalledFontForIndex),
  });
}

async function readRegistryCache(
  storage: FontRegistryStorage,
  port: FontRegistryStoragePort,
): Promise<OrihaInstallableFont[]> {
  const parsed = await readJsonEntry<FontRegistryCache>(
    port,
    storage.registryCacheUri,
  );
  const fonts = Array.isArray(parsed?.fonts) ? parsed.fonts : [];
  return fonts
    .map((font) => normalizeInstallableFont(font, storage.config.baseUrl))
    .filter((font): font is OrihaInstallableFont => font !== null);
}

async function installedFontFilesExist(
  font: OrihaInstalledFont,
  port: FontRegistryStoragePort,
): Promise<boolean> {
  const fileChecks = await Promise.all(
    font.files.map((file) => fileEntryExists(port, file.uri)),
  );
  return fileChecks.every(Boolean);
}
