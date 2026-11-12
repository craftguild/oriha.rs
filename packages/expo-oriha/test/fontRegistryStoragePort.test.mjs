import assert from "node:assert/strict";
import test from "node:test";

import {
  VENDOR_STORAGE_DIR,
  installedFontIndexEntryNeedsRewrite,
  normalizeInstalledFontFileRecord,
  serializeInstalledFontForIndex,
} from "../src/fonts/fontRegistryStorage.ts";
import {
  deleteEntryIfExists,
  ensureDirectoryEntry,
  fileEntryExists,
  readJsonEntry,
  readPrunedIndexEntries,
  writeJsonEntry,
} from "../src/fonts/fontRegistryStoragePort.ts";

const storageNamespace = "my-app";
const oldStorageRoot =
  `file:///old-simulator-container/Documents/${VENDOR_STORAGE_DIR}/${storageNamespace}/`;
const currentStorageRoot =
  `file:///current-simulator-container/Documents/${VENDOR_STORAGE_DIR}/${storageNamespace}/`;

test("storage port JSON helpers read, write, and prune filesystem entries", async () => {
  const port = createMemoryStoragePort({
    "file:///valid.json": JSON.stringify({ ok: true }),
    "file:///invalid.json": "{",
  });

  assert.deepEqual(await readJsonEntry(port, "file:///valid.json"), { ok: true });
  assert.equal(await readJsonEntry(port, "file:///missing.json"), null);
  assert.equal(await readJsonEntry(port, "file:///invalid.json"), null);

  await ensureDirectoryEntry(port, "file:///fonts/");
  assert.deepEqual(await port.getInfo("file:///fonts/"), {
    exists: true,
    isDirectory: true,
  });

  await writeJsonEntry(port, "file:///index.json", { fonts: [] });
  assert.equal(
    port.files.get("file:///index.json"),
    "{\n  \"fonts\": []\n}\n",
  );
  assert.equal(await fileEntryExists(port, "file:///index.json"), true);
  assert.equal(await fileEntryExists(port, "file:///fonts/"), false);

  await deleteEntryIfExists(port, "file:///index.json");
  assert.equal(await fileEntryExists(port, "file:///index.json"), false);
});

test("readPrunedIndexEntries migrates legacy uri entries and drops missing font files", async () => {
  const indexUri = `${currentStorageRoot}installed-index.json`;
  const port = createMemoryStoragePort({
    [indexUri]: JSON.stringify({
      fonts: [
        makeStoredFont("current", {
          relativePath: "files/current/regular.woff2",
          fileName: "regular.woff2",
          sha256: "abc123",
        }),
        makeStoredFont("legacy", {
          uri: `${oldStorageRoot}files/legacy/regular.woff2`,
          fileName: "regular.woff2",
          sha256: "def456",
        }),
        makeStoredFont("missing", {
          relativePath: "files/missing/regular.woff2",
          fileName: "regular.woff2",
          sha256: "987abc",
        }),
        makeStoredFont("invalid", {
          relativePath: "../outside.woff2",
          fileName: "outside.woff2",
          sha256: "456def",
        }),
      ],
    }),
    [`${currentStorageRoot}files/current/regular.woff2`]: "current font bytes",
    [`${currentStorageRoot}files/legacy/regular.woff2`]: "legacy font bytes",
  });

  const entries = await readPrunedIndexEntries({
    port,
    indexUri,
    getRawEntries(index) {
      return Array.isArray(index?.fonts) ? index.fonts : [];
    },
    normalizeEntry: normalizeStoredFontForTest,
    entryExists: async (font) => {
      const fileChecks = await Promise.all(
        font.files.map((file) => fileEntryExists(port, file.uri)),
      );
      return fileChecks.every(Boolean);
    },
    entryNeedsRewrite: installedFontIndexEntryNeedsRewrite,
    serializeIndex(fonts) {
      return {
        fonts: fonts.map(serializeInstalledFontForIndex),
      };
    },
  });

  assert.deepEqual(
    entries.map((font) => font.id),
    ["current", "legacy"],
  );
  assert.equal(
    entries[1].files[0].uri,
    `${currentStorageRoot}files/legacy/regular.woff2`,
  );

  const rewrittenIndex = JSON.parse(port.files.get(indexUri));
  assert.deepEqual(
    rewrittenIndex.fonts.map((font) => font.id),
    ["current", "legacy"],
  );
  assert.equal("uri" in rewrittenIndex.fonts[1].files[0], false);
  assert.equal(
    rewrittenIndex.fonts[1].files[0].relativePath,
    "files/legacy/regular.woff2",
  );
});

function createMemoryStoragePort(seed = {}) {
  const files = new Map(Object.entries(seed));
  const directories = new Set();

  return {
    documentDirectory: "file:///documents/",
    files,
    directories,
    async getInfo(uri) {
      if (directories.has(uri)) {
        return { exists: true, isDirectory: true };
      }
      if (files.has(uri)) {
        return { exists: true, isDirectory: false };
      }
      return { exists: false };
    },
    async readString(uri) {
      if (!files.has(uri)) {
        throw new Error(`Missing memory file: ${uri}`);
      }
      return files.get(uri);
    },
    async writeString(uri, value) {
      files.set(uri, value);
    },
    async makeDirectory(uri) {
      directories.add(uri);
    },
    async delete(uri) {
      files.delete(uri);
      directories.delete(uri);
    },
  };
}

function makeStoredFont(id, file) {
  return {
    id,
    label: id,
    family: id,
    version: "1.0.0",
    installedAt: "2026-05-15T00:00:00.000Z",
    fontFamilySpec: { family: id },
    kinds: ["body"],
    files: [file],
  };
}

function normalizeStoredFontForTest(value) {
  if (!value || typeof value !== "object" || !Array.isArray(value.files)) {
    return null;
  }
  const files = value.files
    .map((file) =>
      normalizeInstalledFontFileRecord(file, {
        storageRootUri: currentStorageRoot,
        storageNamespace,
      }),
    )
    .filter((file) => file !== null);
  if (files.length === 0) {
    return null;
  }
  return {
    ...value,
    files,
  };
}
