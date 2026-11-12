#!/usr/bin/env node
// Build the real package, then verify module evidence against the actual outputs.
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const cache = join(root, ".craftguild/bundle");
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
rmSync(cache, { recursive: true, force: true });
mkdirSync(cache, { recursive: true });
execFileSync(process.execPath, ["scripts/generate-third-party-licenses.mjs"], { cwd: root, stdio: "inherit" });
execFileSync(process.execPath, ["node_modules/tsdown/dist/run.mjs"], { cwd: root, stdio: "inherit" });

const metadata = { inputs: {}, outputs: {}, virtual: {} };
const parts = join(cache, "parts");
if (!existsSync(parts) || readdirSync(parts).length === 0) {
  throw new Error("The real bundler did not provide any module evidence");
}
for (const name of readdirSync(parts).sort()) {
  const part = JSON.parse(readFileSync(join(parts, name), "utf8"));
  for (const field of ["inputs", "outputs", "virtual"]) {
    for (const [key, value] of Object.entries(part[field])) {
      if (metadata[field][key] && JSON.stringify(metadata[field][key]) !== JSON.stringify(value)) {
        throw new Error(`Conflicting bundle evidence: ${key}`);
      }
      metadata[field][key] = value;
    }
  }
}
for (const [name, record] of Object.entries(metadata.inputs)) {
  const bytes = readFileSync(join(root, name));
  if (bytes.length !== record.bytes || hash(bytes) !== record.sha256) {
    throw new Error(`Bundled source changed during compilation: ${name}`);
  }
}
function files(directory) {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name);
    return entry.isDirectory() ? files(path) : [path];
  });
}
for (const path of files(join(root, "dist"))) {
  const name = relative(root, path).replaceAll("\\", "/");
  const bytes = readFileSync(path);
  if (!metadata.outputs[name]) {
    if (!/\.d\.(?:ts|mts|cts)(?:\.map)?$/.test(name)) {
      throw new Error(`Output lacks real bundle evidence: ${name}`);
    }
    metadata.outputs[name] = { bytes: bytes.length, inputs: {} };
  }
  if (metadata.outputs[name].bytes !== bytes.length) throw new Error(`Output changed after bundle generation: ${name}`);
  metadata.outputs[name].sha256 = hash(bytes);
}
for (const name of Object.keys(metadata.outputs)) {
  if (!existsSync(join(root, name))) throw new Error(`Missing bundled output: ${name}`);
}
writeFileSync(join(cache, "metafile.json"), JSON.stringify(metadata, null, 2) + "\n");
const repositoryPath = (name) => `packages/expo-oriha/${name}`;
const supplements = [];
const virtual = Object.keys(metadata.virtual);
if (virtual.some((name) => metadata.virtual[name].id !== "\0rolldown/runtime.js")) {
  throw new Error("Unknown virtual module needs explicit runtime provenance");
}
if (virtual.length) {
  const rolldown = JSON.parse(readFileSync(join(root, "node_modules/rolldown/package.json"), "utf8"));
  supplements.push({
    name: "Rolldown generated runtime helpers", version: rolldown.version, type: "library",
    reference: `urn:oriha:rolldown-runtime:${rolldown.version}`, license: rolldown.license,
    files: [...virtual.map(repositoryPath), repositoryPath("licenses/rolldown-LICENSE")],
    evidence: "Actual positive-contribution Rolldown runtime modules, captured from rendered module code. Only the generated helpers are embedded; the compiler is a build tool.",
  });
}
if (metadata.inputs["generated/generatedHeroicons.ts"]) {
  const icons = JSON.parse(readFileSync(join(root, "node_modules/heroicons/package.json"), "utf8"));
  supplements.push({
    name: "Heroicons solid icon paths", version: icons.version, type: "data",
    reference: `urn:oriha:heroicons:${icons.version}`, license: icons.license,
    files: [repositoryPath("generated/generatedHeroicons.ts"), repositoryPath("licenses/heroicons-LICENSE")],
    evidence: "The real bundle includes the generated six-icon table, reproduced from pinned Heroicons 24/solid SVGs with complete ownership hashes.",
  });
}
writeFileSync(join(cache, "supplemental.json"), JSON.stringify(supplements, null, 2) + "\n");
console.log(`Recorded ${Object.keys(metadata.inputs).length} real inputs and ${Object.keys(metadata.outputs).length} outputs`);
