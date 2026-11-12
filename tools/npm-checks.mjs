#!/usr/bin/env node
// Explicit npm checks, retaining the demo lint and using owned fixture output.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const projects = ["packages/expo-oriha", "packages/font-registry", "examples/expo-preview"];
const npm = process.platform === "win32" ? "npm.cmd" : "npm";

function run(command, argv, directory = root) {
  execFileSync(command, argv, { cwd: directory, stdio: "inherit" });
}

function test() {
  run(process.execPath, ["--experimental-strip-types", "--test", "examples/expo-preview/test/*.test.ts"]);
  run(npm, ["test"], join(root, projects[0]));
  run(process.execPath, ["--test", "packages/font-registry/test/deployment-injection.test.mjs"]);
  const owned = mkdtempSync(join(tmpdir(), "oriha-font-registry-check-"));
  try {
    const registry = join(root, projects[1]);
    run(process.execPath, [
      "node_modules/tsx/dist/cli.mjs", "scripts/build-registry.ts",
      "--config", "fixtures/registry.fixture.json", "--dist", owned,
      "--css-fixture-dir", "fixtures/google-css",
      "--font-fixture-dir", "fixtures/font-files",
    ], registry);
    const manifest = JSON.parse(readFileSync(join(owned, "index.json"), "utf8"));
    assert.ok(manifest.fonts.length > 0, "fixture registry must contain real fixture files");
    for (const font of manifest.fonts) {
      assert.ok(font.files.length > 0);
      for (const file of font.files) {
        const payload = readFileSync(join(owned, file.url));
        assert.equal(payload.length, file.size);
        assert.equal(createHash("sha256").update(payload).digest("hex"), file.sha256);
      }
    }
    console.log("PASS: font-registry fixture assets match their declared size and SHA-256");
  } finally {
    rmSync(owned, { recursive: true, force: true });
  }
}

function lint() {
  run(npm, ["run", "lint"], join(root, projects[2]));
  run(process.execPath, [
    "examples/expo-preview/node_modules/eslint/bin/eslint.js", "--config", "eslint.config.mjs",
    "packages/expo-oriha/src", "packages/expo-oriha/scripts", "packages/expo-oriha/tsdown.config.mts",
    "packages/font-registry/scripts", "packages/font-registry/src", "tools", "scripts",
    "crates/core/scripts", "eslint.config.mjs",
  ]);
}

function typecheck() {
  for (const project of projects) {
    run(process.execPath, ["node_modules/typescript/bin/tsc", "--project", "tsconfig.json", "--noEmit"], join(root, project));
  }
}

const actions = { test, lint, typecheck };
if (process.argv.length !== 3 || !Object.hasOwn(actions, process.argv[2])) {
  throw new Error("Usage: npm-checks.mjs <test|lint|typecheck>");
}
actions[process.argv[2]]();
