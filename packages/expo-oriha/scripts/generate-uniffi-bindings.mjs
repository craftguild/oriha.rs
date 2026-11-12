#!/usr/bin/env node
import {
  copyFileSync,
  cpSync,
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { join, resolve } from "node:path";
import { packageRoot, repoRoot, run } from "./native-build-utils.mjs";

const args = process.argv.slice(2);
if (args.length && (args.length !== 2 || args[0] !== "--output-dir")) {
  throw new Error("Usage: generate-uniffi-bindings.mjs [--output-dir DIRECTORY]");
}
const outputRoot = args.length ? resolve(args[1]) : null;
run("sh", ["tools/standards.sh", "verify-receipt", "--artifact", "native-bridge"]);
const native = JSON.parse(readFileSync(join(repoRoot, ".craftguild/oriha-native-bridge/receipt.json"), "utf8"));
const host = native.context.host;
if (!host || native.context.target !== host || native.context.profile !== "release") {
  throw new Error("Generate UniFFI bindings from a shared release build for the recorded host");
}
const hostLib = native.context.outputs.find((file) => /liboriha_native_bridge\.(dylib|so)$/.test(file));
if (!hostLib) throw new Error("The shared host receipt has no supported dynamic library");
const generatorOptions = {
  env: {
    ...process.env,
    ...(native.context.toolchain ? { RUSTUP_TOOLCHAIN: native.context.toolchain } : {}),
    CARGO_TARGET_DIR: join(repoRoot, ".craftguild/generators/uniffi-bindgen"),
  },
};
const iosGeneratedDir = outputRoot ? join(outputRoot, "swift") : join(packageRoot, "ios/generated");
const macosGeneratedDir = outputRoot ? join(outputRoot, "macos") : join(packageRoot, "macos/generated");
const javaDir = outputRoot ? join(outputRoot, "kotlin") : join(packageRoot, "android/src/main/java");
const uniffiJavaDir = join(javaDir, "uniffi");

rmSync(iosGeneratedDir, { recursive: true, force: true });
rmSync(macosGeneratedDir, { recursive: true, force: true });
rmSync(join(uniffiJavaDir, "oriha_native_bridge"), { recursive: true, force: true });
rmSync(join(uniffiJavaDir, "oriha_uniffi"), { recursive: true, force: true });

if (existsSync(uniffiJavaDir)) {
  for (const entry of readdirSync(uniffiJavaDir, { withFileTypes: true })) {
    if (
      entry.isDirectory() &&
      /^oriha_.*_bridge$/.test(entry.name) &&
      entry.name !== "oriha_native_bridge"
    ) {
      rmSync(join(uniffiJavaDir, entry.name), { recursive: true, force: true });
    }
  }
}

mkdirSync(iosGeneratedDir, { recursive: true });
mkdirSync(macosGeneratedDir, { recursive: true });
mkdirSync(javaDir, { recursive: true });

run("cargo", [
  "run",
  "--locked",
  "--release",
  "--target",
  host,
  "-p",
  "oriha-uniffi-bindgen",
  "--",
  "generate",
  "--library",
  hostLib,
  "--language",
  "swift",
  "--out-dir",
  iosGeneratedDir,
  "--no-format",
], generatorOptions);

for (const entry of readdirSync(iosGeneratedDir)) {
  cpSync(join(iosGeneratedDir, entry), join(macosGeneratedDir, entry), {
    recursive: true,
  });
}
copyFileSync(
  join(macosGeneratedDir, "oriha_native_bridgeFFI.modulemap"),
  join(macosGeneratedDir, "module.modulemap"),
);

run("cargo", [
  "run",
  "--locked",
  "--release",
  "--target",
  host,
  "-p",
  "oriha-uniffi-bindgen",
  "--",
  "generate",
  "--library",
  hostLib,
  "--language",
  "kotlin",
  "--out-dir",
  javaDir,
  "--no-format",
], generatorOptions);

function trimKotlinTrailingWhitespace(dir) {
  if (!existsSync(dir)) {
    return;
  }

  for (const entry of readdirSync(dir)) {
    const path = join(dir, entry);
    const stats = statSync(path);
    if (stats.isDirectory()) {
      trimKotlinTrailingWhitespace(path);
      continue;
    }

    if (!/\.(kt|kts)$/.test(entry)) {
      continue;
    }

    const source = readFileSync(path, "utf8");
    const trimmed = source.replace(/[ \t]+$/gm, "");
    if (trimmed !== source) {
      writeFileSync(path, trimmed);
    }
  }
}

trimKotlinTrailingWhitespace(javaDir);
