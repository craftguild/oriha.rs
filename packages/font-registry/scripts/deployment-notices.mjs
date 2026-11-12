// Retain notices only for packages evidenced in the actual deployment modules.
import { copyFileSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { basename, join, relative } from "node:path";

export function retainNotices(root, metadata, digest, readJson) {
  const keys = new Set(["node_modules/astro", "node_modules/@astrojs/vercel"]);
  for (const input of Object.keys(metadata.inputs)) {
    const parts = input.split("/");
    const index = parts.lastIndexOf("node_modules");
    if (index < 0) continue;
    keys.add(parts.slice(0, index + (parts[index + 1].startsWith("@") ? 3 : 2)).join("/"));
  }
  const lock = readJson(join(root, "package-lock.json"));
  const retained = readJson(join(root, "licenses/runtime-notices.json"));
  const records = [];
  for (const key of [...keys].sort()) {
    const manifest = readJson(join(root, key, "package.json"));
    if (lock.packages[key]?.version !== manifest.version) throw new Error(`Notice package differs from lock: ${key}`);
    const directory = join(root, ".vercel/output/notices", `${encodeURIComponent(manifest.name)}-${manifest.version}`);
    mkdirSync(directory, { recursive: true });
    let licenses = readdirSync(join(root, key))
      .filter((name) => /^(?:licen[cs]e|copying|notice)(?:[.-].*)?$/i.test(name))
      .map((name) => `${key}/${name}`);
    let provenance;
    if (!licenses.length) {
      provenance = retained.packages.find((record) => record.name === manifest.name && record.version === manifest.version);
      if (!provenance || provenance.license !== manifest.license ||
          !/^licenses\/[a-zA-Z0-9.-]+\.txt$/.test(provenance.file) ||
          digest(readFileSync(join(root, provenance.file))) !== provenance.sha256) {
        throw new Error(`Runtime notice text needs explicit provenance: ${manifest.name}`);
      }
      licenses = [provenance.file, `${key}/README.md`, `${key}/package.json`];
    }
    const files = [];
    for (const input of licenses) {
      const output = relative(root, join(directory, basename(input)));
      copyFileSync(join(root, input), join(root, output));
      const bytes = readFileSync(join(root, output));
      metadata.inputs[input] = { bytes: bytes.length, sha256: digest(bytes) };
      metadata.outputs[output] = {
        bytes: bytes.length, sha256: digest(bytes), inputs: { [input]: { bytesInOutput: bytes.length } },
      };
      files.push(output.slice(".vercel/output/".length));
    }
    records.push({ name: manifest.name, version: manifest.version, license: manifest.license, files, provenance });
  }
  const name = ".vercel/output/notices/index.json";
  writeFileSync(join(root, name), JSON.stringify({ schemaVersion: 1, packages: records }, null, 2) + "\n");
  const bytes = readFileSync(join(root, name));
  metadata.outputs[name] = { bytes: bytes.length, sha256: digest(bytes), inputs: {} };
  return records;
}
