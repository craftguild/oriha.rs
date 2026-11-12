#!/usr/bin/env node
// Exercise the actual generated Vercel handler on an owned ephemeral loopback port.
import assert from "node:assert/strict";
import { fork, execFileSync } from "node:child_process";
import { once } from "node:events";
import { createServer } from "node:http";
import { cpSync, mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { digest, readJson, verifyAssets } from "../packages/font-registry/scripts/registry-assets.mjs";

const script = fileURLToPath(import.meta.url);
const args = process.argv.slice(2);
if (args[0] === "--serve") {
  await serve(resolve(args[1]));
} else {
  let root = resolve(dirname(script), "../packages/font-registry");
  let screenshot;
  for (let index = 0; index < args.length; index += 2) {
    if (!args[index + 1]) throw new Error("Missing option value");
    if (args[index] === "--package") root = resolve(args[index + 1]);
    else if (args[index] === "--screenshot") screenshot = resolve(args[index + 1]);
    else throw new Error("Use [--package DIRECTORY] [--screenshot FILE]");
  }
  await verify(root, screenshot);
}

async function serve(root) {
  const directory = join(root, ".vercel/output/functions/_render.func");
  const config = readJson(join(directory, ".vc-config.json"));
  const entry = resolve(directory, config.handler);
  assert.ok(entry.startsWith(directory + "/"), "Handler must belong to the deployment");
  process.chdir(directory);
  const { default: handler } = await import(pathToFileURL(entry).href);
  const server = createServer((request, response) => {
    const path = new URL(request.url, "http://localhost").pathname;
    if (path.startsWith("/_astro/") && !path.includes("..")) {
      try {
        const bytes = readFileSync(join(root, ".vercel/output/static", path));
        response.writeHead(200, { "content-type": path.endsWith(".css") ? "text/css" : "application/javascript" });
        response.end(bytes);
      } catch {
        response.writeHead(404); response.end();
      }
      return;
    }
    const exchange = async () => {
      const input = new Request(`http://${request.headers.host}${request.url}`, {
        method: request.method, headers: request.headers,
      });
      const output = await handler.fetch(input);
      response.writeHead(output.status, Object.fromEntries(output.headers));
      response.end(Buffer.from(await output.arrayBuffer()));
    };
    exchange().catch((error) => {
      console.error(error);
      if (!response.headersSent) response.writeHead(500);
      response.end();
    });
  });
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  process.send({ port: server.address().port });
  process.on("message", () => {
    server.closeAllConnections();
    server.close(() => process.exit(0));
  });
}

async function verify(root, screenshot) {
  const provenance = verifyAssets(root);
  const expected = readFileSync(join(root, "generated/index.json"));
  const manifest = JSON.parse(expected);
  const deployment = realpathSync(mkdtempSync(join(tmpdir(), "oriha-registry-deployment-")));
  mkdirSync(join(deployment, ".vercel"));
  cpSync(join(root, ".vercel/output"), join(deployment, ".vercel/output"), { recursive: true });
  const permitted = [deployment, script, resolve(dirname(script), "../packages/font-registry/scripts/registry-assets.mjs")];
  const permissions = ["--permission", ...permitted.map((path) => `--allow-fs-read=${path}`)];
  if (process.allowedNodeEnvironmentFlags.has("--allow-net")) permissions.push("--allow-net");
  const child = fork(script, ["--serve", deployment], {
    cwd: deployment, execArgv: permissions,
    env: { NODE_ENV: "production", TZ: "UTC", TMPDIR: tmpdir() },
    stdio: ["ignore", "pipe", "pipe", "ipc"],
  });
  let diagnostics = "";
  for (const stream of [child.stdout, child.stderr]) {
    stream.on("data", (bytes) => { diagnostics = (diagnostics + bytes).slice(-64000); });
  }
  try {
    const ready = await Promise.race([
      once(child, "message").then(([value]) => value),
      once(child, "exit").then(() => { throw new Error(`Handler exited: ${diagnostics}`); }),
      new Promise((_, reject) => {
        const timer = setTimeout(() => reject(new Error(`Handler startup timed out: ${diagnostics}`)), 10000);
        timer.unref();
      }),
    ]);
    const base = `http://127.0.0.1:${ready.port}`;
    const get = (path) => fetch(new URL(path, base), { signal: AbortSignal.timeout(10000) });
    const index = await get("/index.json");
    assert.equal(index.status, 200);
    assert.equal(index.headers.get("access-control-allow-origin"), "*");
    assert.equal(index.headers.get("cache-control"), "public, max-age=300, stale-while-revalidate=86400");
    assert.deepEqual(Buffer.from(await index.arrayBuffer()), expected);
    const page = await get("/");
    assert.equal(page.status, 200);
    const html = await page.text();
    for (const font of manifest.fonts) assert.ok(html.includes(font.label), font.label);
    const file = manifest.fonts[0].files[0];
    const font = await get(`/${file.url}`);
    assert.equal(font.status, 200);
    assert.equal(font.headers.get("content-type"), "font/woff2");
    assert.equal(font.headers.get("cache-control"), "public, max-age=31536000, immutable");
    assert.equal(digest(Buffer.from(await font.arrayBuffer())), file.sha256);
    const license = provenance.fonts[0].license;
    const text = await get(`/${license.url}`);
    assert.equal(text.status, 200);
    assert.equal(digest(Buffer.from(await text.arrayBuffer())), license.sha256);
    if (screenshot) {
      const profile = mkdtempSync(join(tmpdir(), "oriha-registry-browser-"));
      try {
        execFileSync(process.env.CHROME ?? "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome", [
          "--headless", "--disable-gpu", "--no-first-run", "--no-default-browser-check",
          "--disable-background-networking", "--disable-component-update", "--disable-sync", "--disable-extensions",
          `--user-data-dir=${profile}`, "--window-size=1280,960", `--screenshot=${screenshot}`, base,
        ], { timeout: 20000, stdio: "ignore" });
      } finally {
        rmSync(profile, { recursive: true, force: true });
      }
    }
    console.log(`PASS: actual deployment serves ${manifest.fonts.length} families, exact registry JSON, font/license bytes and original HTTP headers`);
  } catch (error) {
    throw new Error(`Deployment acceptance failed: ${error.message}\n${diagnostics}`, { cause: error });
  } finally {
    if (child.exitCode === null && child.signalCode === null) {
      if (child.connected) child.send("close", () => {});
      const timer = setTimeout(() => child.kill("SIGKILL"), 2000);
      timer.unref();
      await once(child, "exit");
      clearTimeout(timer);
    }
    rmSync(deployment, { recursive: true, force: true });
  }
}
