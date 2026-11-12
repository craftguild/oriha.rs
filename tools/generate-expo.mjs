#!/usr/bin/env node
// Reproduce the complete checked-in Expo bridge and icon output in an owned root.
import { execFileSync } from "node:child_process";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const args = process.argv.slice(2);
if (args.length !== 2 || args[0] !== "--output-dir") {
  throw new Error("Usage: generate-expo.mjs --output-dir DIRECTORY");
}
for (const script of ["generate-heroicons.mjs", "generate-bridge-types.mjs"]) {
  execFileSync(process.execPath, [
    `packages/expo-oriha/scripts/${script}`, "--output-dir", resolve(args[1]),
  ], { cwd: root, stdio: "inherit" });
}
