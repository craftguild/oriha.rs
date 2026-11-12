#!/usr/bin/env node

import { existsSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";
import { checkGitState } from "./release-git-state.mjs";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const packageRoot = resolve(scriptDir, "..");
const repoRoot = resolve(packageRoot, "../..");
const npmCacheDir = mkdtempSync(join(tmpdir(), "oriha-release-npm-cache-"));
const args = new Set(process.argv.slice(2));

const allowDirty =
  args.has("--allow-dirty") || process.env.ORIHA_RELEASE_CHECK_ALLOW_DIRTY === "1";
const skipRemote =
  args.has("--skip-remote") || process.env.ORIHA_RELEASE_CHECK_SKIP_REMOTE === "1";
const skipAuditSignatures =
  args.has("--skip-audit-signatures") ||
  process.env.ORIHA_RELEASE_CHECK_SKIP_AUDIT_SIGNATURES === "1";
const verbose =
  args.has("--verbose") || process.env.ORIHA_RELEASE_CHECK_VERBOSE === "1";
const environmentOnly = args.has("--environment-only");

const failures = [];
const warnings = [];
const ansiPattern = /\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])/g;
const ansiReset = "\x1b[0m";

const lifecycleScripts = ["preinstall", "install", "postinstall", "prepare"];
const forbiddenPackageLifecycleScripts = new Set([
  "preinstall",
  "install",
  "postinstall",
  "prepare",
  "prepack",
  "postpack",
  "prepublish",
  "prepublishOnly",
  "publish",
  "postpublish",
]);

const dangerousScriptPattern =
  /\b(curl|wget|Invoke-WebRequest|iwr|irm|powershell|pwsh|nc|ncat|socat|sshpass)\b|\/dev\/tcp|\bbase64\s+(-d|--decode)\b|\bnode\s+-e\b|\bpython[23]?\s+-c\b|\bruby\s+-e\b|\bperl\s+-e\b|\beval\b/i;

const sensitiveEnvPatterns = [
  /^NPM_TOKEN$/i,
  /^NODE_AUTH_TOKEN$/i,
  /^GITHUB_TOKEN$/i,
  /^GH_TOKEN$/i,
  /^GITLAB_TOKEN$/i,
  /^AWS_(ACCESS_KEY_ID|SECRET_ACCESS_KEY|SESSION_TOKEN)$/i,
  /^GOOGLE_APPLICATION_CREDENTIALS$/i,
  /^GCP_.*(TOKEN|KEY|SECRET|PASSWORD|CREDENTIAL)$/i,
  /^AZURE_.*(TOKEN|KEY|SECRET|PASSWORD|CREDENTIAL)$/i,
  /^(CLOUDFLARE|DIGITALOCEAN|HEROKU|VERCEL|NETLIFY|SENTRY|SLACK|OPENAI|ANTHROPIC|MISTRAL|HUGGINGFACE|HF)_.*(TOKEN|KEY|SECRET|PASSWORD|CREDENTIAL)$/i,
  /^DOCKER_.*(TOKEN|PASSWORD|CONFIG)$/i,
  /^KUBE(CONFIG|_.*(TOKEN|KEY|SECRET|PASSWORD|CREDENTIAL))$/i,
  /(^|_)AUTH_TOKEN$/i,
  /(^|_)API_KEY$/i,
  /(^|_)ACCESS_TOKEN$/i,
  /(^|_)SECRET_ACCESS_KEY$/i,
  /(^|_)CLIENT_SECRET$/i,
  /(^|_)PRIVATE_KEY$/i,
  /^npm_config_.*(_auth|authToken|token)$/i,
];

const advisoryEnvKeys = ["SSH_AUTH_SOCK"];

function fail(message) {
  failures.push(message);
}

function warn(message) {
  warnings.push(message);
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

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

function run(command, commandArgs, options = {}) {
  const result = spawnSync(command, commandArgs, {
    cwd: options.cwd ?? repoRoot,
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

function checkNoSensitiveEnvironment() {
  const sensitiveKeys = Object.keys(process.env)
    .filter((key) => {
      const value = process.env[key];
      return value && sensitiveEnvPatterns.some((pattern) => pattern.test(key));
    })
    .sort();

  if (sensitiveKeys.length > 0) {
    fail(
      `Sensitive environment variables are set: ${sensitiveKeys.join(
        ", ",
      )}. Unset them before install/build/pack; log in to npm only after this check passes.`,
    );
  }

  const advisoryKeys = advisoryEnvKeys.filter((key) => process.env[key]);
  if (advisoryKeys.length > 0) {
    warn(
      `Advisory: ${advisoryKeys.join(
        ", ",
      )} is set. Prefer a shell without agent sockets while building release artifacts.`,
    );
  }
}

function checkLocalNpmrc() {
  for (const npmrcPath of [join(repoRoot, ".npmrc"), join(packageRoot, ".npmrc")]) {
    if (!existsSync(npmrcPath)) {
      continue;
    }

    const lines = readFileSync(npmrcPath, "utf8")
      .split(/\r?\n/)
      .map((line) => line.trim())
      .filter((line) => line && !line.startsWith("#") && !line.startsWith(";"));

    const authLines = lines.filter((line) =>
      /(_authToken|_auth|username\s*=|_password\s*=)/i.test(line),
    );

    if (authLines.length > 0) {
      fail(
        `${relative(repoRoot, npmrcPath)} contains npm auth settings. Keep publish credentials out of the repository tree.`,
      );
    }
  }
}

function checkNpmSession() {
  const result = run("npm", ["whoami", "--registry=https://registry.npmjs.org"], {
    cwd: packageRoot,
    timeout: 10_000,
  });

  if (result.ok) {
    fail(
      "An npmjs.org login session is already active. Run this release check before npm login, then publish, then npm logout.",
    );
    return;
  }

  if (result.error && result.error.code !== "ENOENT") {
    warn(`Could not confirm npm login state: ${result.error.message}`);
  }
}

function checkOwnPackageScripts(packageJson) {
  const scripts = packageJson.scripts ?? {};

  for (const name of Object.keys(scripts)) {
    if (forbiddenPackageLifecycleScripts.has(name)) {
      fail(
        `package.json script "${name}" is not allowed in the publish package. Use explicit release commands instead of npm lifecycle hooks.`,
      );
    }

    if (
      name.startsWith("pre") &&
      name.length > 3 &&
      Object.prototype.hasOwnProperty.call(scripts, name.slice(3))
    ) {
      fail(`package.json script "${name}" is an implicit npm pre-hook. Remove it.`);
    }

    if (
      name.startsWith("post") &&
      name.length > 4 &&
      Object.prototype.hasOwnProperty.call(scripts, name.slice(4))
    ) {
      fail(`package.json script "${name}" is an implicit npm post-hook. Remove it.`);
    }

    if (dangerousScriptPattern.test(String(scripts[name]))) {
      fail(
        `package.json script "${name}" contains a suspicious command pattern: ${scripts[name]}`,
      );
    }
  }
}

function checkDependencyLifecycleScripts() {
  const lockPath = join(packageRoot, "package-lock.json");
  if (!existsSync(lockPath)) {
    fail("package-lock.json is missing.");
    return;
  }

  const lock = readJson(lockPath);
  const packageEntries = Object.keys(lock.packages ?? {}).filter((path) =>
    path.startsWith("node_modules/"),
  );
  const prepareScripts = [];

  for (const packagePath of packageEntries) {
    const dependencyPackageJsonPath = join(packageRoot, packagePath, "package.json");
    if (!existsSync(dependencyPackageJsonPath)) {
      if (!lock.packages[packagePath]?.optional) {
        warn(`Installed dependency is missing package.json: ${packagePath}`);
      }
      continue;
    }

    let dependencyPackageJson;
    try {
      dependencyPackageJson = readJson(dependencyPackageJsonPath);
    } catch (error) {
      fail(`Could not parse ${packagePath}/package.json: ${error.message}`);
      continue;
    }

    const dependencyName = dependencyPackageJson.name ?? packagePath;
    const scripts = dependencyPackageJson.scripts ?? {};

    for (const lifecycleName of lifecycleScripts) {
      const command = scripts[lifecycleName];
      if (!command) {
        continue;
      }

      const label = `${dependencyName} (${packagePath}) ${lifecycleName}: ${command}`;

      if (dangerousScriptPattern.test(String(command))) {
        fail(`Dependency lifecycle script has a suspicious command pattern: ${label}`);
      } else if (lifecycleName === "prepare") {
        prepareScripts.push(label);
      } else {
        fail(`Dependency has an install-time lifecycle script: ${label}`);
      }
    }
  }

  if (prepareScripts.length > 0) {
    const visible = verbose ? prepareScripts : prepareScripts.slice(0, 20);
    warn(
      [
        `${prepareScripts.length} dependency prepare scripts are present in installed package metadata.`,
        "They are not run by npm ci --ignore-scripts, but review newly added entries before release.",
        ...visible.map((entry) => `  - ${entry}`),
        verbose || prepareScripts.length <= visible.length
          ? ""
          : `  ... ${prepareScripts.length - visible.length} more; rerun with --verbose to list all.`,
      ]
        .filter(Boolean)
        .join("\n"),
    );
  }
}

function checkVersions(packageJson) {
  const version = packageJson.version;
  if (!/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/.test(version)) {
    fail(`Invalid package.json version: ${String(version)}`);
    return null;
  }

  const lock = readJson(join(packageRoot, "package-lock.json"));
  if (lock.version !== version) {
    fail(`package-lock.json version ${lock.version} does not match package.json ${version}.`);
  }

  const lockRootVersion = lock.packages?.[""]?.version;
  if (lockRootVersion !== version) {
    fail(
      `package-lock.json root package version ${lockRootVersion} does not match package.json ${version}.`,
    );
  }

  const cargoToml = readFileSync(join(repoRoot, "Cargo.toml"), "utf8");
  const workspacePackageSection = cargoToml.match(
    /(\[workspace\.package\][\s\S]*?)(?=\n\[|$)/,
  );
  const cargoVersion = workspacePackageSection?.[1]?.match(/^version\s*=\s*"([^"]+)"/m)?.[1];

  if (cargoVersion !== version) {
    fail(`Cargo workspace version ${cargoVersion} does not match package.json ${version}.`);
  }

  return version;
}

function runAuditSignatures() {
  if (skipAuditSignatures) {
    warn("Skipped npm audit signatures.");
    return;
  }

  const result = run("npm", ["audit", "signatures", "--json"], {
    cwd: packageRoot,
    timeout: 120_000,
  });

  if (!result.ok) {
    fail(`npm audit signatures failed: ${summarizeCommandFailure(result)}`);
    return;
  }

  try {
    const audit = JSON.parse(result.stdout);
    const invalid = audit.invalid?.length ?? 0;
    const missing = audit.missing?.length ?? 0;
    if (invalid > 0 || missing > 0) {
      fail(`npm audit signatures reported invalid=${invalid}, missing=${missing}.`);
    }
  } catch (error) {
    fail(`Could not parse npm audit signatures output: ${error.message}`);
  }
}

function finish(successMessage, failureTitle) {
  if (warnings.length > 0) {
    console.log("\nWarnings:");
    for (const message of warnings) {
      console.log(`- ${stripAnsi(message)}`);
    }
  }

  if (failures.length > 0) {
    console.error(`\n${failureTitle}`);
    for (const message of failures) {
      console.error(`- ${stripAnsi(message)}`);
    }
    resetTerminalStyle();
    process.exit(1);
  }

  console.log(successMessage);
  resetTerminalStyle();
}

function main() {
  resetTerminalStyle();

  console.log(
    environmentOnly
      ? "Running release environment checks..."
      : "Running release security checks...",
  );

  const packageJson = readJson(join(packageRoot, "package.json"));

  checkNoSensitiveEnvironment();
  checkLocalNpmrc();
  checkNpmSession();
  checkOwnPackageScripts(packageJson);

  if (environmentOnly) {
    finish("Release environment checks passed.", "Release environment check failed:");
    return;
  }

  checkDependencyLifecycleScripts();

  const version = checkVersions(packageJson);
  if (version) {
    checkGitState(version, {
      repoRoot, run, fail, warn, allowDirty, skipRemote, summarizeCommandFailure,
    });
  }

  runAuditSignatures();

  finish("Release security checks passed.", "Release security check failed:");
}

process.on("exit", () => {
  rmSync(npmCacheDir, { recursive: true, force: true });
  resetTerminalStyle();
});

main();
