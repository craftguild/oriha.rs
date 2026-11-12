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

const outDir = join(packageRoot, "ios/native");
const headerDir = join(packageRoot, "ios/generated");

ensureCommand("xcodebuild", ["-version"], "xcodebuild is required to create the iOS XCFramework.");

const developerDir = capture("xcode-select", ["-p"], { allowFailure: true }).stdout.trim();
if (developerDir.includes("/CommandLineTools")) {
  console.error("The active developer directory is Command Line Tools, but iOS builds require full Xcode.");
  console.error(`Current developer directory: ${developerDir}`);
  console.error("Install/open Xcode, then run:");
  console.error("  sudo xcode-select -s /Applications/Xcode.app/Contents/Developer");
  console.error("  sudo xcodebuild -license accept");
  process.exit(1);
}

ensureCommand("xcrun", ["--sdk", "iphoneos", "--show-sdk-path"], [
  "Unable to locate the iPhoneOS SDK. A full Xcode installation is required.",
  "If Xcode is installed, run:",
  "  sudo xcode-select -s /Applications/Xcode.app/Contents/Developer",
].join("\n"));

ensureCommand("xcrun", ["--sdk", "iphonesimulator", "--show-sdk-path"], [
  "Unable to locate the iPhoneSimulator SDK. A full Xcode installation is required.",
  "If Xcode is installed, run:",
  "  sudo xcode-select -s /Applications/Xcode.app/Contents/Developer",
].join("\n"));

ensureFile(
  join(headerDir, "oriha_native_bridgeFFI.h"),
  "Missing UniFFI headers. Run node scripts/generate-uniffi-bindings.mjs first.",
);

const targets = ["aarch64-apple-ios", "aarch64-apple-ios-sim"];
const installedTargets = installedRustTargets();
for (const target of targets) {
  ensureRustTarget(target, installedTargets);
  buildNativeArtifact(target);
  retainNativeArtifact(target, [join(targetRoot, target, "release/liboriha_native_bridge.a")]);
}

mkdirSync(outDir, { recursive: true });
for (const entry of readdirSync(outDir, { withFileTypes: true })) {
  if (entry.isDirectory() && /^Oriha.*Bridge\.xcframework$/.test(entry.name)) {
    rmSync(join(outDir, entry.name), { recursive: true, force: true });
  }
}

run("xcodebuild", [
  "-create-xcframework",
  "-library",
  join(targetRoot, "aarch64-apple-ios/release/liboriha_native_bridge.a"),
  "-headers",
  headerDir,
  "-library",
  join(targetRoot, "aarch64-apple-ios-sim/release/liboriha_native_bridge.a"),
  "-headers",
  headerDir,
  "-output",
  join(outDir, "OrihaNativeBridge.xcframework"),
]);
