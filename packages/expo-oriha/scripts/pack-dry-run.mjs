#!/usr/bin/env node

import { existsSync, mkdtempSync, readFileSync, rmSync, statSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const packageRoot = resolve(scriptDir, "..");
const npmCacheDir = mkdtempSync(join(tmpdir(), "oriha-pack-npm-cache-"));
const failures = [];
const ansiPattern = /\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])/g;
const ansiReset = "\x1b[0m";

const forbiddenPackPatterns = [
  /(^|\/)\.env($|\.)/,
  /(^|\/)\.npmrc$/,
  /(^|\/)\.git($|\/)/,
  /(^|\/)\.github($|\/)/,
  /(^|\/)\.vscode($|\/)/,
  /(^|\/)\.claude($|\/)/,
  /(^|\/)\.cursor($|\/)/,
  /(^|\/)\.windsurf($|\/)/,
  /^android\/build($|\/)/,
  /(^|\/)(id_rsa|id_ed25519|known_hosts)$/,
  /\.(pem|p12|pfx|key)$/,
  /(^|\/)(setup_bun\.js|bun_environment\.js|truffleSecrets\.json)$/,
  /(^|\/)(discussion\.ya?ml|formatter_[^/]+\.ya?ml)$/,
  /(^|\/).+\.tgz$/,
];

const secretContentPatterns = [
  { name: "npm token", pattern: /npm_[A-Za-z0-9]{36,}/ },
  { name: "GitHub token", pattern: /gh[pousr]_[A-Za-z0-9_]{36,}/ },
  { name: "GitHub fine-grained token", pattern: /github_pat_[A-Za-z0-9_]{80,}/ },
  { name: "AWS access key", pattern: /AKIA[0-9A-Z]{16}/ },
  { name: "Google API key", pattern: /AIza[0-9A-Za-z_-]{35}/ },
  { name: "OpenAI token", pattern: /sk-(proj-)?[A-Za-z0-9_-]{32,}/ },
  { name: "Slack token", pattern: /xox[baprs]-[A-Za-z0-9-]{20,}/ },
  {
    name: "private key",
    pattern: /-----BEGIN (RSA |OPENSSH |EC |DSA )?PRIVATE KEY-----/,
  },
  { name: "Shai-Hulud IOC", pattern: /(SHA1HULUD|setup_bun|bun_environment|truffleSecrets)/i },
];

const textFilePattern =
  /\.(cjs|cts|gradle|h|js|json|kt|mjs|mts|md|modulemap|plist|podspec|properties|swift|ts|tsx|txt|xml)$/i;

function fail(message) {
  failures.push(message);
}

function stripAnsi(value) {
  return String(value).replace(ansiPattern, "");
}

function resetTerminalStyle() {
  if (process.stdout.isTTY) {
    process.stdout.write(ansiReset);
  }
  if (process.stderr.isTTY) {
    process.stderr.write(ansiReset);
  }
}

function run(command, commandArgs, options = {}) {
  const result = spawnSync(command, commandArgs, {
    cwd: options.cwd ?? packageRoot,
    encoding: "utf8",
    env: {
      ...process.env,
      FORCE_COLOR: "0",
      NO_COLOR: process.env.NO_COLOR ?? "1",
      NPM_CONFIG_CACHE: npmCacheDir,
      npm_config_color: "false",
      npm_config_cache: npmCacheDir,
    },
    timeout: options.timeout ?? 30_000,
    stdio: ["ignore", "pipe", "pipe"],
  });

  return {
    ok: result.status === 0,
    status: result.status,
    stdout: stripAnsi(result.stdout ?? ""),
    stderr: stripAnsi(result.stderr ?? ""),
    error: result.error,
  };
}

function summarizeCommandFailure(result) {
  if (result.error) {
    return result.error.message;
  }

  const output = `${result.stderr}\n${result.stdout}`.trim();
  return output || `exit status ${String(result.status)}`;
}

function npmPackDryRun() {
  console.log("Running npm pack dry run...");

  const result = run("npm", ["pack", "--dry-run", "--json", "--ignore-scripts"], {
    timeout: 120_000,
  });

  if (!result.ok) {
    fail(`npm pack --dry-run failed: ${summarizeCommandFailure(result)}`);
    return null;
  }

  try {
    const pack = JSON.parse(result.stdout);
    if (!Array.isArray(pack) || pack.length === 0 || !Array.isArray(pack[0].files)) {
      fail("Unexpected npm pack --dry-run JSON shape.");
      return null;
    }

    const files = pack[0].files.map((file) => file.path);
    console.log(
      `npm pack dry run passed: ${pack[0].filename ?? "package tarball"} (${files.length} files).`,
    );
    return files;
  } catch (error) {
    fail(`Could not parse npm pack --dry-run output: ${error.message}`);
    return null;
  }
}

function checkPackFiles(paths) {
  const fileSet = new Set(paths);
  const requiredFiles = [
    "package.json",
    "README.md",
    "THIRD_PARTY_NOTICES.md",
    "dist/index.cjs",
    "dist/index.mjs",
    "dist/index.d.cts",
    "dist/index.d.mts",
    "expo-module.config.json",
    "ExpoOriha.podspec",
    "licenses/third-party-licenses.json",
    "sbom/bom.cdx.json",
    "sbom/artifacts.json",
    "sbom/javascript.cdx.json",
    "sbom/aarch64-apple-ios.cdx.json",
    "sbom/aarch64-apple-ios-sim.cdx.json",
    "sbom/aarch64-linux-android.cdx.json",
    "sbom/armv7-linux-androideabi.cdx.json",
    "sbom/x86_64-linux-android.cdx.json",
    "android/src/main/jniLibs/arm64-v8a/liboriha_native_bridge.so",
    "android/src/main/jniLibs/armeabi-v7a/liboriha_native_bridge.so",
    "android/src/main/jniLibs/x86_64/liboriha_native_bridge.so",
    "ios/native/OrihaNativeBridge.xcframework/Info.plist",
    "ios/native/OrihaNativeBridge.xcframework/ios-arm64/liboriha_native_bridge.a",
    "ios/native/OrihaNativeBridge.xcframework/ios-arm64-simulator/liboriha_native_bridge.a",
  ];

  for (const requiredFile of requiredFiles) {
    if (!fileSet.has(requiredFile)) {
      fail(`Expected release artifact is missing from npm package: ${requiredFile}`);
    }
  }

  const forbiddenFiles = paths.filter((path) =>
    forbiddenPackPatterns.some((pattern) => pattern.test(path)),
  );

  if (forbiddenFiles.length > 0) {
    fail(`Forbidden files would be published:\n${forbiddenFiles.map((path) => `  - ${path}`).join("\n")}`);
  }

  for (const path of paths) {
    const absolutePath = join(packageRoot, path);
    if (!existsSync(absolutePath)) {
      continue;
    }

    const stats = statSync(absolutePath);
    if (!stats.isFile() || stats.size > 1_000_000 || !textFilePattern.test(path)) {
      continue;
    }

    const content = readFileSync(absolutePath, "utf8");
    for (const { name, pattern } of secretContentPatterns) {
      if (pattern.test(content)) {
        fail(`Possible ${name} found in packed file: ${path}`);
      }
    }
  }
}

function finish() {
  if (failures.length > 0) {
    console.error("\nnpm pack dry run failed:");
    for (const message of failures) {
      console.error(`- ${stripAnsi(message)}`);
    }
    resetTerminalStyle();
    process.exit(1);
  }

  console.log("npm pack dry run checks passed.");
  resetTerminalStyle();
}

function main() {
  resetTerminalStyle();
  const packFiles = npmPackDryRun();
  if (packFiles) {
    checkPackFiles(packFiles);
  }
  finish();
}

process.on("exit", () => {
  rmSync(npmCacheDir, { recursive: true, force: true });
  resetTerminalStyle();
});

main();
