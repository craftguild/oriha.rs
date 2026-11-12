#!/usr/bin/env node
import { copyFileSync, mkdirSync, rmSync } from "node:fs";
import { join } from "node:path";
import {
  ensureCommand,
  ensureRustTarget,
  installedRustTargets,
  packageRoot,
  buildNativeArtifact,
  retainNativeArtifact,
  targetRoot,
} from "./native-build-utils.mjs";

const outDir = join(packageRoot, "android/src/main/jniLibs");
const targets = [
  "aarch64-linux-android",
  "armv7-linux-androideabi",
  "x86_64-linux-android",
];

ensureCommand("cargo", ["ndk", "--version"], [
  "cargo-ndk is required to build Android native libraries.",
  "Install it with: cargo install cargo-ndk",
].join("\n"));

const installedTargets = installedRustTargets();
for (const target of targets) {
  ensureRustTarget(
    target,
    installedTargets,
    `Install all Android targets with:\n  rustup target add ${targets.join(" ")}`,
  );
}

rmSync(outDir, { recursive: true, force: true });
mkdirSync(outDir, { recursive: true });

const abis = ["arm64-v8a", "armeabi-v7a", "x86_64"];
for (const [index, target] of targets.entries()) {
  buildNativeArtifact(target);
  const directory = join(outDir, abis[index]);
  mkdirSync(directory, { recursive: true });
  copyFileSync(join(targetRoot, target, "release/liboriha_native_bridge.so"),
    join(directory, "liboriha_native_bridge.so"));
  retainNativeArtifact(target, [join(outDir, abis[index], "liboriha_native_bridge.so")]);
}
