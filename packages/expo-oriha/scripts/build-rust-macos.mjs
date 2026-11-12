#!/usr/bin/env node
import { mkdirSync, readdirSync, rmSync } from "node:fs";
import { join } from "node:path";
import {
  capture,
  ensureCommand,
  ensureFile,
  ensureRustTarget,
  installedRustTargets,
  packageRoot,
  buildNativeArtifact,
  retainNativeArtifact,
  run,
  targetRoot,
} from "./native-build-utils.mjs";

const outDir = join(packageRoot, "macos/native");
const headerDir = join(packageRoot, "macos/generated");
const supportedTargets = new Set(["aarch64-apple-darwin", "x86_64-apple-darwin"]);

ensureCommand("xcodebuild", ["-version"], "xcodebuild is required to create the macOS XCFramework.");

ensureCommand("xcrun", ["--sdk", "macosx", "--show-sdk-path"], [
  "Unable to locate the macOS SDK. A full Xcode installation is required.",
  "If Xcode is installed, run:",
  "  sudo xcode-select -s /Applications/Xcode.app/Contents/Developer",
].join("\n"));

ensureFile(
  join(headerDir, "oriha_native_bridgeFFI.h"),
  "Missing UniFFI headers. Run node scripts/generate-uniffi-bindings.mjs first.",
);

const installedTargets = installedRustTargets();
const targets = process.env.MACOS_RUST_TARGETS
  ? process.env.MACOS_RUST_TARGETS.split(/\s+/).filter(Boolean)
  : defaultTargets(installedTargets);

const libraries = [];
for (const target of targets) {
  if (!supportedTargets.has(target)) {
    console.error(`Unsupported macOS Rust target: ${target}`);
    console.error("Expected aarch64-apple-darwin or x86_64-apple-darwin.");
    process.exit(1);
  }

  ensureRustTarget(target, installedTargets);
  buildNativeArtifact(target);
  const library = join(targetRoot, target, "release/liboriha_native_bridge.a");
  retainNativeArtifact(target, [library]);
  libraries.push(library);
}

mkdirSync(outDir, { recursive: true });
for (const entry of readdirSync(outDir, { withFileTypes: true })) {
  if (entry.isDirectory() && /^Oriha.*Bridge\.xcframework$/.test(entry.name)) {
    rmSync(join(outDir, entry.name), { recursive: true, force: true });
  }
}
rmSync(join(outDir, "universal"), { recursive: true, force: true });

let library = libraries[0];
if (libraries.length > 1) {
  const universalDir = join(outDir, "universal");
  library = join(universalDir, "liboriha_native_bridge.a");
  mkdirSync(universalDir, { recursive: true });
  run("lipo", ["-create", ...libraries, "-output", library]);
}

run("xcodebuild", [
  "-create-xcframework",
  "-library",
  library,
  "-headers",
  headerDir,
  "-output",
  join(outDir, "OrihaNativeBridge.xcframework"),
]);

function defaultTargets(installedTargets) {
  const rustc = capture("rustc", ["-vV"]);
  const hostTarget = rustc.stdout.match(/^host: (.+)$/m)?.[1]?.trim();

  switch (hostTarget) {
    case "aarch64-apple-darwin":
      return appendOptionalTarget(["aarch64-apple-darwin"], "x86_64-apple-darwin", installedTargets);
    case "x86_64-apple-darwin":
      return appendOptionalTarget(["x86_64-apple-darwin"], "aarch64-apple-darwin", installedTargets);
    default:
      console.error(`Unsupported Rust host target for macOS build: ${hostTarget ?? "unknown"}`);
      process.exit(1);
  }
}

function appendOptionalTarget(targets, optionalTarget, installedTargets) {
  if (installedTargets.has(optionalTarget)) {
    targets.push(optionalTarget);
  }
  return targets;
}
