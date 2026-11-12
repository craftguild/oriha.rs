#!/usr/bin/env node
// Build the deployment and map its actual emitted/copied bytes to their sources.
import { execFileSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { digest, files, readJson, verifyAssets } from "./registry-assets.mjs";
import { retainNotices } from "./deployment-notices.mjs";
import { injectedChunk } from "./deployment-injection.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const cache = join(root, ".craftguild/bundle");
if (process.argv.length !== 2) throw new Error("build-evidence.mjs takes no arguments");
if (!existsSync(join(root, "generated/provenance.json"))) {
  execFileSync(process.execPath, ["node_modules/tsx/dist/cli.mjs", "scripts/build-registry.ts", "--dist", "generated"], { cwd: root, stdio: "inherit" });
}
const fonts = verifyAssets(root);
rmSync(cache, { recursive: true, force: true });
rmSync(join(root, ".vercel/output"), { recursive: true, force: true });
mkdirSync(cache, { recursive: true });
execFileSync(process.execPath, ["node_modules/astro/bin/astro.mjs", "build"], { cwd: root, stdio: "inherit" });

const parts = files(join(cache, "parts")).map(readJson);
const bundled = new Map();
for (const part of parts) {
  for (const [name, output] of Object.entries(part.outputs)) {
    // The adapter relocates/removes intermediate chunks. Select only hashes
    // actually present in the final deployment below, including copied chunks.
    if (existsSync(join(root, name)) && digest(readFileSync(join(root, name))) !== output.sha256) {
      throw new Error(`Emitted bundle changed: ${name}`);
    }
    const evidence = { output, inputs: part.inputs, virtual: part.virtual };
    const previous = bundled.get(output.sha256);
    bundled.set(output.sha256, previous ? [...previous, evidence] : [evidence]);
  }
}
const metadata = { inputs: {}, outputs: {}, virtual: {} };
const readInput = (name) => {
  const bytes = readFileSync(join(root, name));
  metadata.inputs[name] = { bytes: bytes.length, sha256: digest(bytes) };
  return bytes;
};
for (const path of files(join(root, ".vercel/output"))) {
  const name = relative(root, path);
  const bytes = readFileSync(path);
  const sha256 = digest(bytes);
  const contributions = {};
  const evidence = bundled.get(sha256) ?? (name.endsWith(".mjs")
    ? injectedChunk(root, name, bytes, parts, digest) : null);
  if (evidence) {
    for (const part of evidence) {
      for (const [input, contribution] of Object.entries(part.output.inputs)) {
        const actual = readInput(input);
        if (digest(actual) !== part.inputs[input].sha256) throw new Error(`Bundle source changed: ${input}`);
        contributions[input] = contribution;
        if (part.virtual[input]) metadata.virtual[input] = part.virtual[input];
      }
    }
  } else if (name.includes("/node_modules/")) {
    const source = name.slice(name.indexOf("/node_modules/") + 1);
    if (digest(readInput(source)) !== sha256) throw new Error(`Copied runtime module differs from installed source: ${name}`);
    contributions[source] = { bytesInOutput: bytes.length };
  } else if (name.includes("/generated/")) {
    const source = name.slice(name.indexOf("/generated/") + 1);
    if (digest(readInput(source)) !== sha256) throw new Error(`Copied font asset changed: ${name}`);
    contributions[source] = { bytesInOutput: bytes.length };
  } else if (name.startsWith(".vercel/output/functions/_render.func/src/")) {
    const source = name.slice(".vercel/output/functions/_render.func/".length);
    if (digest(readInput(source)) !== sha256) throw new Error(`Copied application source changed: ${name}`);
    contributions[source] = { bytesInOutput: bytes.length };
  } else if (!/\.(?:json|gitkeep)$/.test(name) && !name.endsWith("/index.mjs")) {
    throw new Error(`Deployment output needs explicit byte provenance: ${name}`);
  }
  metadata.outputs[name] = { bytes: bytes.length, sha256, inputs: contributions };
}
retainNotices(root, metadata, digest, readJson);
const snapshots = [];
const helpers = [];
for (const [name, record] of Object.entries(metadata.virtual)) {
  if (record.id === "\0rolldown/runtime.js") helpers.push(name);
  else if (record.id.startsWith("\0virtual:astro") || record.id === "\0astro:assets") snapshots.push(name);
  else throw new Error(`Unknown virtual runtime provenance: ${record.id}`);
}
let runtime;
const runtimeNotice = ".vercel/output/notices/rolldown-runtime-LICENSE";
if (helpers.length) {
  runtime = readJson(join(root, "node_modules/rolldown/package.json"));
  const lock = readJson(join(root, "package-lock.json"));
  if (runtime.version !== lock.packages["node_modules/rolldown"]?.version) {
    throw new Error("Runtime helper producer differs from the locked compiler");
  }
  copyFileSync(join(root, "node_modules/rolldown/LICENSE"), join(root, runtimeNotice));
  const bytes = readFileSync(join(root, runtimeNotice));
  metadata.outputs[runtimeNotice] = { bytes: bytes.length, sha256: digest(bytes), inputs: {} };
}
// Adapter-generated bootstrap/config has an explicit retained-byte supplement.
const generated = Object.entries(metadata.outputs)
  .filter(([name, record]) => name !== runtimeNotice && !Object.keys(record.inputs).length).map(([name]) => name);
const packagePath = (name) => `packages/font-registry/${name}`;
const supplements = fonts.fonts.map((font) => ({
  name: font.license.family, version: `css2-${font.cssSha256.slice(0, 16)}`, type: "file",
  reference: `urn:oriha:registry-font:${font.id}:${font.cssSha256}`, license: font.license.license,
  files: [
    ...font.files.map((file) => packagePath(`generated/${file.url}`)),
    packagePath(`generated/${font.cssFile}`), packagePath(`generated/${font.license.url}`),
    packagePath(font.license.file), packagePath("generated/provenance.json"),
  ],
  evidence: `Actual downloaded CSS2/font bytes and retained upstream license: ${font.license.source}. The component version identifies the observed CSS hash, not an asserted upstream font version.`,
}));
if (runtime) {
  supplements.push({
    name: "Rolldown generated runtime helpers", version: runtime.version, type: "library",
    reference: `urn:oriha:registry-rolldown-runtime:${runtime.version}`, license: runtime.license,
    files: [...helpers, runtimeNotice, "node_modules/rolldown/package.json", "node_modules/rolldown/LICENSE"].map(packagePath),
    evidence: "Actual positive-contribution virtual helper code, independently captured from the compiler; the compiler package is not assumed wholly shipped.",
  });
}
if (snapshots.length || generated.length) {
  const framework = readJson(join(root, "node_modules/astro/package.json"));
  const adapter = readJson(join(root, "node_modules/@astrojs/vercel/package.json"));
  supplements.push({
    name: "Astro/Vercel generated deployment modules", version: `${framework.version}-${adapter.version}`,
    type: "library", reference: "urn:oriha:registry-generated-runtime", license: "MIT",
    files: [...snapshots, ...generated].map(packagePath),
    evidence: "Rendered virtual module snapshots, actual copied deployment bootstrap/config and assets without runtime modules; framework/compiler packages are not assumed wholly shipped. Actual positive-contribution dependency modules are recorded separately.",
  });
}
writeFileSync(join(cache, "metafile.json"), JSON.stringify(metadata, null, 2) + "\n");
writeFileSync(join(cache, "supplemental.json"), JSON.stringify(supplements, null, 2) + "\n");
console.log(`Verified ${fonts.fonts.length} production font families and ${Object.keys(metadata.outputs).length} actual deployment files`);
