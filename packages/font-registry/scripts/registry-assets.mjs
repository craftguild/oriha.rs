// Verify retained licenses and actual production downloads without substituting fixtures.
import { createHash } from "node:crypto";
import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";

export const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
export const readJson = (path) => JSON.parse(readFileSync(path, "utf8"));
export function files(directory) {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    if (entry.isSymbolicLink()) throw new Error(`Unowned deployment symlink: ${entry.name}`);
    const path = join(directory, entry.name);
    return entry.isDirectory() ? files(path) : [path];
  });
}

export function verifyLicenses(root) {
  const config = readJson(join(root, "registry.config.json"));
  const manifest = readJson(join(root, "licenses/manifest.json"));
  if (!/^[0-9a-f]{40}$/.test(manifest.revision) || manifest.fonts.length !== config.fonts.length) {
    throw new Error("Font license revision/inventory differs from configuration");
  }
  const seen = new Set();
  for (const font of config.fonts) {
    const record = manifest.fonts.find((record) => record.id === font.id && record.family === font.family);
    if (!record || seen.has(record.id) || !/^licenses\/[a-z0-9-]+-(?:OFL|LICENSE)\.txt$/.test(record.file) ||
        !record.source.startsWith(`https://raw.githubusercontent.com/google/fonts/${manifest.revision}/`)) {
      throw new Error(`Missing pinned font license identity: ${font.id}`);
    }
    if (digest(readFileSync(join(root, record.file))) !== record.sha256) throw new Error(`Changed license: ${font.id}`);
    seen.add(record.id);
  }
  return manifest;
}

export function verifyAssets(root) {
  const licenses = verifyLicenses(root);
  const provenance = readJson(join(root, "generated/provenance.json"));
  const index = readJson(join(root, "generated/index.json"));
  if (provenance.configSha256 !== digest(readFileSync(join(root, "registry.config.json"))) ||
      provenance.fonts.length !== licenses.fonts.length || index.fonts.length !== licenses.fonts.length) {
    throw new Error("Production font assets need regeneration for this configuration");
  }
  const expected = new Set(["index.json", "provenance.json"]);
  for (const font of provenance.fonts) {
    const indexed = index.fonts.find((entry) => entry.id === font.id);
    const license = licenses.fonts.find((entry) => entry.id === font.id);
    if (!indexed || !license || indexed.files.length !== font.files.length || font.license.sha256 !== license.sha256) {
      throw new Error(`Production font association differs: ${font.id}`);
    }
    if (font.cssFile !== `sources/${font.id}.css`) throw new Error("Unowned CSS provenance path");
    const records = [
      ...font.files.map((file) => ({ ...file, path: file.url })),
      { path: font.cssFile, sha256: font.cssSha256 },
      { path: font.license.url, sha256: license.sha256 },
    ];
    for (const record of records) {
      if (!/^(?:fonts|sources)\/[a-z0-9-]+\/[a-zA-Z0-9_.-]+$/.test(record.path) && record.path !== font.cssFile) {
        throw new Error(`Unowned font output path: ${record.path}`);
      }
      const bytes = readFileSync(join(root, "generated", record.path));
      if (digest(bytes) !== record.sha256 || (record.size !== undefined && bytes.length !== record.size)) {
        throw new Error(`Production font bytes changed: ${record.path}`);
      }
      expected.add(record.path);
    }
    for (const file of indexed.files) {
      if (!font.files.some((record) => record.url === file.url && record.sha256 === file.sha256 && record.size === file.size)) {
        throw new Error(`Font API inventory differs from downloaded bytes: ${font.id}`);
      }
    }
  }
  const actual = files(join(root, "generated")).map((path) => path.slice(join(root, "generated").length + 1));
  if (actual.length !== expected.size || actual.some((path) => !expected.has(path))) {
    throw new Error("Missing or extra production font assets");
  }
  return provenance;
}
