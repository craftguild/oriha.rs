export type FontRegistryStorageEncoding = "base64" | "utf8";

export type FontRegistryStorageInfo = {
  exists: boolean;
  isDirectory?: boolean;
};

export type FontRegistryStoragePort = {
  readonly documentDirectory?: string | null;
  getInfo(uri: string): Promise<FontRegistryStorageInfo>;
  readString(
    uri: string,
    options?: { encoding?: FontRegistryStorageEncoding },
  ): Promise<string>;
  writeString(
    uri: string,
    value: string,
    options?: { encoding?: FontRegistryStorageEncoding },
  ): Promise<void>;
  makeDirectory(uri: string, options?: { intermediates?: boolean }): Promise<void>;
  delete(uri: string, options?: { idempotent?: boolean }): Promise<void>;
};

export type ReadPrunedIndexEntriesOptions<Entry> = {
  port: FontRegistryStoragePort;
  indexUri: string;
  getRawEntries(index: unknown): unknown[];
  normalizeEntry(entry: unknown): Entry | null;
  entryExists(entry: Entry): Promise<boolean>;
  entryNeedsRewrite(entry: unknown): boolean;
  serializeIndex(entries: Entry[]): unknown;
};

export async function readJsonEntry<T>(
  port: FontRegistryStoragePort,
  uri: string,
): Promise<T | null> {
  const info = await port.getInfo(uri);
  if (!info.exists || info.isDirectory) {
    return null;
  }

  try {
    return JSON.parse(await port.readString(uri)) as T;
  } catch {
    return null;
  }
}

export async function writeJsonEntry(
  port: FontRegistryStoragePort,
  uri: string,
  value: unknown,
): Promise<void> {
  await port.writeString(uri, `${JSON.stringify(value, null, 2)}\n`);
}

export async function ensureDirectoryEntry(
  port: FontRegistryStoragePort,
  uri: string,
): Promise<void> {
  await port.makeDirectory(uri, { intermediates: true });
}

export async function deleteEntryIfExists(
  port: FontRegistryStoragePort,
  uri: string,
): Promise<void> {
  const info = await port.getInfo(uri);
  if (info.exists) {
    await port.delete(uri, { idempotent: true });
  }
}

export async function fileEntryExists(
  port: FontRegistryStoragePort,
  uri: string,
): Promise<boolean> {
  try {
    const info = await port.getInfo(uri);
    return info.exists && !info.isDirectory;
  } catch {
    return false;
  }
}

export async function readPrunedIndexEntries<Entry>(
  options: ReadPrunedIndexEntriesOptions<Entry>,
): Promise<Entry[]> {
  const parsed = await readJsonEntry<unknown>(options.port, options.indexUri);
  const rawEntries = options.getRawEntries(parsed);
  const normalizedEntries = rawEntries
    .map(options.normalizeEntry)
    .filter((entry): entry is Entry => entry !== null);

  const existingEntries = await filterExistingEntries(
    normalizedEntries,
    options.entryExists,
  );
  const needsRewrite =
    normalizedEntries.length !== rawEntries.length ||
    rawEntries.some(options.entryNeedsRewrite) ||
    existingEntries.length !== normalizedEntries.length;

  if (needsRewrite) {
    try {
      await writeJsonEntry(
        options.port,
        options.indexUri,
        options.serializeIndex(existingEntries),
      );
    } catch {
      // The caller can still treat pruned entries as missing for this read.
    }
  }

  return existingEntries;
}

async function filterExistingEntries<Entry>(
  entries: Entry[],
  entryExists: (entry: Entry) => Promise<boolean>,
): Promise<Entry[]> {
  const checkedEntries = await Promise.all(
    entries.map(async (entry) => ({
      entry,
      exists: await entryExists(entry),
    })),
  );
  return checkedEntries
    .filter((checkedEntry) => checkedEntry.exists)
    .map((checkedEntry) => checkedEntry.entry);
}
