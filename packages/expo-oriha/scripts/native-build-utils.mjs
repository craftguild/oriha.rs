import { spawnSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { basename, dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));

export const packageRoot = resolve(scriptDir, "..");
export const repoRoot = resolve(packageRoot, "../..");
export const targetRoot = resolve(repoRoot, process.env.CARGO_TARGET_DIR ?? "target");

export function buildNativeArtifact(target) {
  run("sh", ["tools/artifacts.sh", "build", "--library", "native-bridge", "--target", target, "--profile", "release"]);
}

export function retainNativeArtifact(target, files) {
  run("sh", ["tools/standards.sh", "verify-receipt", "--artifact", "native-bridge"]);
  const receipt = JSON.parse(readFileSync(join(repoRoot, ".craftguild/oriha-native-bridge/receipt.json"), "utf8"));
  const prepared = receipt.prepared;
  if (prepared.build.target !== target) throw new Error("Native target differs from its preparation");
  const directory = `.craftguild/native-outputs/oriha-native-bridge/${target}`;
  const destination = join(repoRoot, directory);
  mkdirSync(destination, { recursive: true });
  copyFileSync(join(repoRoot, prepared.sbom), join(destination, "bom.json"));
  prepared.sbom = `${directory}/bom.json`;
  writeFileSync(join(destination, "prepared.json"), JSON.stringify(prepared, null, 2) + "\n");
  const outputs = files.map((file) => {
    const name = basename(file);
    const sha256 = createHash("sha256").update(readFileSync(file)).digest("hex");
    if (receipt.outputs[name] !== sha256) throw new Error("Packaged native bytes differ from the shared receipt");
    return { name, sha256 };
  });
  writeFileSync(join(destination, "artifacts.json"), JSON.stringify({
    schemaVersion: 1, target, sbomSha256: prepared.sbom_sha256, outputs,
  }, null, 2) + "\n");
}

export function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: options.cwd ?? repoRoot,
    env: options.env ?? process.env,
    stdio: options.stdio ?? "inherit",
    encoding: options.encoding,
  });

  if (result.error) {
    console.error(`Failed to start ${command}: ${result.error.message}`);
    process.exit(1);
  }

  if (result.status !== 0 && !options.allowFailure) {
    process.exit(result.status ?? 1);
  }

  return result;
}

export function capture(command, args, options = {}) {
  return run(command, args, {
    cwd: options.cwd,
    stdio: ["ignore", "pipe", "pipe"],
    encoding: "utf8",
    allowFailure: options.allowFailure,
  });
}

export function ensureCommand(command, args, message) {
  const result = run(command, args, {
    stdio: "ignore",
    allowFailure: true,
  });

  if (result.status !== 0) {
    console.error(message);
    process.exit(1);
  }
}

export function installedRustTargets() {
  return new Set(
    capture("rustup", ["target", "list", "--installed"])
      .stdout.split(/\r?\n/)
      .map((line) => line.trim())
      .filter(Boolean),
  );
}

export function ensureRustTarget(target, installedTargets, installHint) {
  if (!installedTargets.has(target)) {
    console.error(`Missing Rust target: ${target}`);
    console.error(installHint ?? `Install it with: rustup target add ${target}`);
    process.exit(1);
  }
}

export function ensureFile(path, message) {
  if (!existsSync(path)) {
    console.error(message);
    process.exit(1);
  }
}
