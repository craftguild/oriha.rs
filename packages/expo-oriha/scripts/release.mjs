#!/usr/bin/env node
// Compatibility entry: common version/note preparation, with no commit or publish.
import { spawnSync } from "node:child_process";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
const args = process.argv.slice(2);
const result = spawnSync("sh", ["tools/standards.sh", "release", "--group", "oriha", ...(args.length ? args : ["--apply"])], {
  cwd: root, stdio: "inherit",
});
if (result.error) throw result.error;
process.exitCode = result.status ?? 1;
