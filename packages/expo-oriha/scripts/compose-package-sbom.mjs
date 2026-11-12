#!/usr/bin/env node
// Require current preparation seals and associate them with the exact npm bytes.
import { createHash } from "node:crypto";
import { copyFileSync, lstatSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { execFileSync } from "node:child_process";
import { appendGraph, packageBom } from "./sbom-composition.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const repository = resolve(root, "../..");
const destination = join(root, "sbom");
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const read = (path) => JSON.parse(readFileSync(path, "utf8"));
const write = (name, value) => writeFileSync(join(destination, name), JSON.stringify(value, null, 2) + "\n");
const adapter = (...args) => {
  const env = args[0] === "verify-frontend"
    ? { ...process.env, TARGET: "", TOOLCHAIN: "", FEATURES: "", NO_DEFAULT_FEATURES: "" }
    : process.env;
  execFileSync("sh", ["tools/artifacts.sh", ...args], { cwd: repository, stdio: "inherit", env });
};
const manifest = read(join(root, "package.json"));
const native = [
  ["aarch64-apple-ios", "ios/native/OrihaNativeBridge.xcframework/ios-arm64/liboriha_native_bridge.a"],
  ["aarch64-apple-ios-sim", "ios/native/OrihaNativeBridge.xcframework/ios-arm64-simulator/liboriha_native_bridge.a"],
  ["aarch64-linux-android", "android/src/main/jniLibs/arm64-v8a/liboriha_native_bridge.so"],
  ["armv7-linux-androideabi", "android/src/main/jniLibs/armeabi-v7a/liboriha_native_bridge.so"],
  ["x86_64-linux-android", "android/src/main/jniLibs/x86_64/liboriha_native_bridge.so"],
];
const required = [
  "ios/generated/oriha_native_bridge.swift", "ios/generated/oriha_native_bridgeFFI.h",
  "android/src/main/java/uniffi/oriha_native_bridge/oriha_native_bridge.kt",
  "licenses/third-party-licenses.json", "licenses/heroicons-LICENSE", "licenses/rolldown-LICENSE",
  "LICENSE", "LICENSE-MIT", "LICENSE-APACHE",
];
// Never retain an earlier composed result when replacement verification fails.
if (lstatSync(root).isSymbolicLink()) throw new Error("Package root must be owned");
try {
  if (lstatSync(destination).isSymbolicLink()) throw new Error("SBOM directory cannot be a symlink");
} catch (error) {
  if (error.code !== "ENOENT") throw error;
}
rmSync(destination, { recursive: true, force: true });
mkdirSync(destination);
for (const name of required) {
  if (!lstatSync(join(root, name)).isFile()) throw new Error(`Missing package input: ${name}`);
}
const bom = packageBom(manifest);
const rootEdges = bom.dependencies[0].dependsOn;
const associations = [];
adapter("verify-frontend", "--artifact", "expo-js");
const jsRecord = read(join(repository, ".craftguild/expo-oriha-js/prepared.json"));
copyFileSync(join(repository, jsRecord.sbom), join(destination, "javascript.cdx.json"));
rootEdges.push(appendGraph(bom, read(join(destination, "javascript.cdx.json")), "javascript"));
associations.push({ artifact: "javascript", sbom: "javascript.cdx.json", sha256: jsRecord.sbom_sha256 });
for (const [target, path] of native) {
  const directory = `.craftguild/native-outputs/oriha-native-bridge/${target}`;
  adapter("verify", "--library", "native-bridge", "--target", target, "--record", `${directory}/prepared.json`);
  const receipt = read(join(repository, directory, "artifacts.json"));
  const prepared = read(join(repository, directory, "prepared.json"));
  const bytes = readFileSync(join(root, path));
  if (receipt.target !== target || receipt.sbomSha256 !== prepared.sbom_sha256 ||
      receipt.outputs.length !== 1 || receipt.outputs[0].sha256 !== hash(bytes)) {
    throw new Error(`Native payload differs from its retained build: ${target}`);
  }
  const name = `${target}.cdx.json`;
  copyFileSync(join(repository, prepared.sbom), join(destination, name));
  rootEdges.push(appendGraph(bom, read(join(destination, name)), target));
  associations.push({ artifact: target, sbom: name, sha256: receipt.sbomSha256, output: path, outputSha256: hash(bytes) });
}

const temporary = mkdtempSync(join(tmpdir(), "oriha-package-evidence-"));
try {
  const pack = (args) => JSON.parse(execFileSync("npm", ["pack", "--ignore-scripts", "--json", ...args], {
    cwd: root, encoding: "utf8", env: { ...process.env, npm_config_cache: join(temporary, "cache") },
  }))[0];
  const inventory = pack(["--dry-run"]).files.filter(({ path }) => !path.startsWith("sbom/"));
  const files = inventory.map(({ path, size }) => {
    const absolute = resolve(root, path);
    if (relative(root, absolute).startsWith(`..${sep}`) || !lstatSync(absolute).isFile()) {
      throw new Error(`Unowned package file: ${path}`);
    }
    const bytes = readFileSync(absolute);
    if (bytes.length !== size) throw new Error(`Package input changed: ${path}`);
    const sha256 = hash(bytes);
    const reference = `urn:oriha:payload:${encodeURIComponent(path)}`;
    bom.components.push({ type: "file", "bom-ref": reference, name: path, hashes: [{ alg: "SHA-256", content: sha256 }] });
    bom.dependencies.push({ ref: reference, dependsOn: [] });
    rootEdges.push(reference);
    return { path, bytes: size, sha256 };
  }).sort((a, b) => a.path.localeCompare(b.path, "en"));
  const included = new Set(files.map(({ path }) => path));
  for (const path of [...required, ...native.map(([, path]) => path)]) {
    if (!included.has(path)) throw new Error(`Required file is excluded by npm: ${path}`);
  }
  write("bom.cdx.json", bom);
  adapter("validate-sbom", "--path", "packages/expo-oriha/sbom/bom.cdx.json");
  write("artifacts.json", {
    schemaVersion: 1, name: manifest.name, version: manifest.version,
    bom: "bom.cdx.json", bomSha256: hash(readFileSync(join(destination, "bom.cdx.json"))),
    artifacts: associations, files,
  });
  const packed = pack(["--pack-destination", temporary]);
  for (const file of files) {
    if (!packed.files.some(({ path, size }) => path === file.path && size === file.bytes)) {
      throw new Error(`Actual tarball differs from the recorded inventory: ${file.path}`);
    }
  }
  for (const file of packed.files) {
    if (!file.path.startsWith("sbom/") && !included.has(file.path)) throw new Error(`Unexpected tarball file: ${file.path}`);
  }
  const archive = join(temporary, packed.filename);
  const members = execFileSync("tar", ["-tzf", archive], { encoding: "utf8" }).trim().split("\n");
  const expected = new Set(packed.files.map(({ path }) => `package/${path}`));
  if (members.length !== expected.size || members.some((name) => !expected.has(name))) {
    throw new Error("Actual tarball paths differ from npm's payload inventory");
  }
  const extraction = join(temporary, "extracted");
  mkdirSync(extraction);
  execFileSync("tar", ["-xzf", archive, "-C", extraction]);
  for (const file of packed.files) {
    const bytes = readFileSync(join(extraction, "package", file.path));
    if (hash(bytes) !== hash(readFileSync(join(root, file.path)))) {
      throw new Error(`Tarball content differs from the verified payload: ${file.path}`);
    }
  }
  console.log(`Verified ${packed.filename}: ${files.length} payload files, JavaScript and ${native.length} native SBOMs`);
} catch (error) {
  rmSync(destination, { recursive: true, force: true });
  throw error;
} finally {
  rmSync(temporary, { recursive: true, force: true });
}
