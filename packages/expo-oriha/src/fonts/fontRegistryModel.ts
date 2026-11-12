// Font registry public contracts and normalized storage descriptors.
import type { FontRegistryFont, FontRegistryFontFile, FontRegistryFontKind } from "./fontRegistryTypes";
import type { InstalledFontFileRuntimeRecord } from "./fontRegistryStorage";

export type OrihaFontKind = FontRegistryFontKind;

export type OrihaFontFile = FontRegistryFontFile;

export type OrihaInstalledFontFile = Omit<
  OrihaFontFile,
  "url"
> &
  InstalledFontFileRuntimeRecord;

export type OrihaInstallableFont = FontRegistryFont;

export type OrihaInstalledFont = Omit<
  OrihaInstallableFont,
  "files"
> & {
  installedAt: string;
  files: OrihaInstalledFontFile[];
};

export type OrihaFontRegistryConfig = {
  baseUrl?: string;
  storageNamespace?: string;
};

export type OrihaFontInstallPhase =
  | "download"
  | "verify"
  | "write"
  | "complete";

export type OrihaFontInstallProgress = {
  fontId: string;
  fontLabel: string;
  fileIndex: number;
  fileCount: number;
  fileName: string;
  fileUrl: string;
  phase: OrihaFontInstallPhase;
  fileReceivedBytes?: number;
  fileTotalBytes?: number;
  installedBytes?: number;
  totalBytes?: number;
  overallProgress?: number;
};

export type OrihaInstallFontOptions = {
  onProgress?: (progress: OrihaFontInstallProgress) => void;
};

export type InstalledFontIndex = {
  fonts?: unknown;
};

export type FontRegistryCache = {
  refreshedAt: string;
  fonts: OrihaInstallableFont[];
};

export const DEFAULT_STORAGE_NAMESPACE = "default";

export const INSTALLED_INDEX_FILE = "installed-index.json";

export const REGISTRY_CACHE_FILE = "registry-cache.json";

export const REGISTRY_MANIFEST_FILE = "index.json";

export type NormalizedFontRegistryConfig = {
  baseUrl?: string;
  storageNamespace: string;
};

export type FontRegistryStorage = {
  config: NormalizedFontRegistryConfig;
  rootDir: string;
  installedIndexUri: string;
  registryCacheUri: string;
  filesDir: string;
};
