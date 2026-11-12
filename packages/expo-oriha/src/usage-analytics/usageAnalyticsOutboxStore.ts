import * as FileSystem from "expo-file-system/legacy";

import { VENDOR_STORAGE_DIR } from "../fonts/fontRegistryStorage";
import type { UsageAnalyticsStoredEnvelope } from "../types";

export type UsageAnalyticsOutboxStore = {
  loadEnvelopes(): Promise<UsageAnalyticsStoredEnvelope[]>;
  writeEnvelope(envelope: UsageAnalyticsStoredEnvelope): Promise<void>;
  deleteEnvelope(id: string): Promise<void>;
  purgeEnvelopes(): Promise<void>;
  pendingCount?(): Promise<number>;
};

export type UsageAnalyticsOutboxFileStoreConfig = {
  storageNamespace?: string;
  rootDirectory?: string | null;
};

const DEFAULT_STORAGE_NAMESPACE = "default";
const OUTBOX_DIR = "usage-analytics-outbox";
const ENVELOPE_FILE_EXTENSION = ".json";

export function createExpoUsageAnalyticsOutboxStore(
  config?: UsageAnalyticsOutboxFileStoreConfig,
): UsageAnalyticsOutboxStore {
  const rootDirectory = config?.rootDirectory ?? FileSystem.documentDirectory;
  if (!rootDirectory) {
    throw new Error("Expo FileSystem documentDirectory is not available");
  }

  return new ExpoUsageAnalyticsOutboxStore(
    `${ensureTrailingSlash(rootDirectory)}${VENDOR_STORAGE_DIR}/${sanitizeStorageNamespace(
      config?.storageNamespace ?? DEFAULT_STORAGE_NAMESPACE,
    )}/${OUTBOX_DIR}/`,
  );
}

class ExpoUsageAnalyticsOutboxStore implements UsageAnalyticsOutboxStore {
  private readonly directoryUri: string;

  constructor(directoryUri: string) {
    this.directoryUri = directoryUri;
  }

  async loadEnvelopes(): Promise<UsageAnalyticsStoredEnvelope[]> {
    await this.ensureDirectory();
    const fileNames = await FileSystem.readDirectoryAsync(this.directoryUri);
    const envelopes = await Promise.all(
      fileNames
        .map((fileName) => envelopeIdFromFileName(fileName))
        .filter((id): id is string => id !== null)
        .map(async (id) => {
          try {
            return {
              id,
              envelopeJson: await FileSystem.readAsStringAsync(this.envelopeUri(id)),
            };
          } catch {
            return null;
          }
        }),
    );
    return envelopes.filter(
      (envelope): envelope is UsageAnalyticsStoredEnvelope => envelope !== null,
    );
  }

  async writeEnvelope(envelope: UsageAnalyticsStoredEnvelope): Promise<void> {
    await this.ensureDirectory();
    const finalUri = this.envelopeUri(envelope.id);
    const temporaryUri = this.temporaryEnvelopeUri(envelope.id);

    try {
      await FileSystem.writeAsStringAsync(temporaryUri, envelope.envelopeJson);
      await FileSystem.deleteAsync(finalUri, { idempotent: true });
      await FileSystem.moveAsync({ from: temporaryUri, to: finalUri });
    } catch (error) {
      await deleteIfExists(temporaryUri);
      throw error;
    }
  }

  async deleteEnvelope(id: string): Promise<void> {
    await FileSystem.deleteAsync(this.envelopeUri(id), { idempotent: true });
  }

  async purgeEnvelopes(): Promise<void> {
    await FileSystem.deleteAsync(this.directoryUri, { idempotent: true });
  }

  async pendingCount(): Promise<number> {
    await this.ensureDirectory();
    const fileNames = await FileSystem.readDirectoryAsync(this.directoryUri);
    return fileNames.filter((fileName) => envelopeIdFromFileName(fileName) !== null)
      .length;
  }

  private envelopeUri(id: string): string {
    return `${this.directoryUri}${encodeURIComponent(id)}${ENVELOPE_FILE_EXTENSION}`;
  }

  private temporaryEnvelopeUri(id: string): string {
    return `${this.directoryUri}${encodeURIComponent(id)}.${Date.now()}.${Math.random()
      .toString(36)
      .slice(2)}.tmp`;
  }

  private async ensureDirectory(): Promise<void> {
    await FileSystem.makeDirectoryAsync(this.directoryUri, { intermediates: true });
  }
}

function envelopeIdFromFileName(fileName: string): string | null {
  if (!fileName.endsWith(ENVELOPE_FILE_EXTENSION)) {
    return null;
  }

  try {
    const id = decodeURIComponent(
      fileName.slice(0, -ENVELOPE_FILE_EXTENSION.length),
    );
    return id.length > 0 ? id : null;
  } catch {
    return null;
  }
}

function sanitizeStorageNamespace(value: string): string {
  const namespace = value.trim() || DEFAULT_STORAGE_NAMESPACE;
  return encodeURIComponent(namespace);
}

function ensureTrailingSlash(value: string): string {
  return value.endsWith("/") ? value : `${value}/`;
}

async function deleteIfExists(uri: string): Promise<void> {
  try {
    await FileSystem.deleteAsync(uri, { idempotent: true });
  } catch {
    // Cleanup is best-effort; the original write error is more important.
  }
}
