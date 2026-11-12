import assert from "node:assert/strict";
import test from "node:test";

import {
  VENDOR_STORAGE_DIR,
  buildFontRelativePath,
  installedFontIndexEntryNeedsRewrite,
  normalizeInstalledFontFileRecord,
  normalizeStoredFontRelativePath,
  relativePathFromLegacyFontUri,
  resolveFontFileUri,
  serializeInstalledFontForIndex,
} from "../src/fonts/fontRegistryStorage.ts";

const storageNamespace = "my-app";
const oldStorageRoot =
  `file:///old-simulator-container/Documents/${VENDOR_STORAGE_DIR}/${storageNamespace}/`;
const currentStorageRoot =
  `file:///current-simulator-container/Documents/${VENDOR_STORAGE_DIR}/${storageNamespace}/`;

test("buildFontRelativePath encodes font ids under the font files directory", () => {
  assert.equal(
    buildFontRelativePath("Noto Sans/JP", "regular.woff2"),
    "files/Noto%20Sans%2FJP/regular.woff2",
  );
});

test("normalizeInstalledFontFileRecord rebuilds uri from stored relativePath", () => {
  const file = normalizeInstalledFontFileRecord(
    {
      relativePath: "files/inter/regular.woff2",
      fileName: "regular.woff2",
      sha256: "abc123",
      weight: "400",
      format: "woff2",
    },
    { storageRootUri: currentStorageRoot, storageNamespace },
  );

  assert.deepEqual(file, {
    uri: `${currentStorageRoot}files/inter/regular.woff2`,
    relativePath: "files/inter/regular.woff2",
    fileName: "regular.woff2",
    sha256: "abc123",
    weight: "400",
    style: undefined,
    format: "woff2",
    unicodeRange: undefined,
    size: undefined,
  });
});

test("normalizeInstalledFontFileRecord migrates legacy absolute uri to current storage root", () => {
  const file = normalizeInstalledFontFileRecord(
    {
      uri: `${oldStorageRoot}files/inter/regular.woff2`,
      fileName: "regular.woff2",
      sha256: "abc123",
    },
    { storageRootUri: currentStorageRoot, storageNamespace },
  );

  assert.equal(file?.relativePath, "files/inter/regular.woff2");
  assert.equal(file?.uri, `${currentStorageRoot}files/inter/regular.woff2`);
});

test("relativePathFromLegacyFontUri ignores other storage namespaces", () => {
  assert.equal(
    relativePathFromLegacyFontUri(
      `${oldStorageRoot}files/inter/regular.woff2`,
      "another-app",
    ),
    null,
  );
});

test("pre-Oriha storage is not migrated into the new font directory", () => {
  const uri =
    "file:///old-simulator-container/Documents/" +
    "co.progressiveworks.expo-markdown-document-engine/my-app/files/inter/regular.woff2";

  assert.equal(relativePathFromLegacyFontUri(uri, storageNamespace), null);
  assert.equal(
    normalizeInstalledFontFileRecord(
      { uri, fileName: "regular.woff2", sha256: "abc123" },
      { storageRootUri: currentStorageRoot, storageNamespace },
    ),
    null,
  );
});

test("normalizeStoredFontRelativePath rejects absolute and traversal paths", () => {
  for (const value of [
    "../files/inter/regular.woff2",
    "files/../inter/regular.woff2",
    "files/inter/../../regular.woff2",
    "files//inter/regular.woff2",
    "font-files/inter/regular.woff2",
    "/files/inter/regular.woff2",
    "file:///tmp/regular.woff2",
    "https://example.com/regular.woff2",
  ]) {
    assert.equal(normalizeStoredFontRelativePath(value), null, value);
  }
});

test("serializeInstalledFontForIndex strips absolute uri", () => {
  const serialized = serializeInstalledFontForIndex({
    id: "inter",
    files: [
      {
        uri: `${currentStorageRoot}files/inter/regular.woff2`,
        relativePath: "files/inter/regular.woff2",
        fileName: "regular.woff2",
        sha256: "abc123",
      },
    ],
  });

  assert.equal("uri" in serialized.files[0], false);
  assert.deepEqual(serialized.files[0], {
    relativePath: "files/inter/regular.woff2",
    fileName: "regular.woff2",
    sha256: "abc123",
  });
});

test("installedFontIndexEntryNeedsRewrite detects legacy uri entries", () => {
  assert.equal(
    installedFontIndexEntryNeedsRewrite({
      files: [
        {
          uri: `${currentStorageRoot}files/inter/regular.woff2`,
          relativePath: "files/inter/regular.woff2",
          fileName: "regular.woff2",
          sha256: "abc123",
        },
      ],
    }),
    true,
  );

  assert.equal(
    installedFontIndexEntryNeedsRewrite({
      files: [
        {
          relativePath: "files/inter/regular.woff2",
          fileName: "regular.woff2",
          sha256: "abc123",
        },
      ],
    }),
    false,
  );
});

test("installedFontIndexEntryNeedsRewrite detects absolute path-like relativePath entries", () => {
  assert.equal(
    installedFontIndexEntryNeedsRewrite({
      files: [
        {
          relativePath: "/files/inter/regular.woff2",
          fileName: "regular.woff2",
          sha256: "abc123",
        },
      ],
    }),
    true,
  );
});

test("resolveFontFileUri tolerates storage roots without trailing slash", () => {
  assert.equal(
    resolveFontFileUri(currentStorageRoot.replace(/\/$/, ""), "files/inter/regular.woff2"),
    `${currentStorageRoot}files/inter/regular.woff2`,
  );
});
