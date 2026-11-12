#!/usr/bin/env node
// Wait for a real browser assertion result; virtual-time DOM dumps can stop
// before asynchronous image.decode() promises have settled.
import { spawn } from "node:child_process";
import { once } from "node:events";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { setTimeout as delay } from "node:timers/promises";
import { pathToFileURL } from "node:url";

class Protocol {
  constructor(socket) {
    this.socket = socket;
    this.nextId = 1;
    this.pending = new Map();
    this.listeners = new Set();
    socket.addEventListener("message", ({ data }) => {
      const message = JSON.parse(data);
      if (message.id) {
        const pending = this.pending.get(message.id);
        if (!pending) return;
        this.pending.delete(message.id);
        clearTimeout(pending.timer);
        if (message.error) pending.reject(new Error(JSON.stringify(message.error)));
        else pending.resolve(message.result);
      } else {
        for (const listener of this.listeners) listener(message);
      }
    });
    socket.addEventListener("close", () => {
      for (const pending of this.pending.values()) {
        clearTimeout(pending.timer);
        pending.reject(new Error("Owned browser closed before replying"));
      }
      this.pending.clear();
    });
  }

  send(method, params = {}) {
    return new Promise((resolve, reject) => {
      const id = this.nextId++;
      const timer = setTimeout(() => {
        this.pending.delete(id);
        reject(new Error(`Browser protocol timed out: ${method}`));
      }, 20_000);
      this.pending.set(id, { resolve, reject, timer });
      this.socket.send(JSON.stringify({ id, method, params }));
    });
  }

  loaded(url) {
    return new Promise((resolve, reject) => {
      let navigated = false;
      const timer = setTimeout(() => {
        this.listeners.delete(listener);
        reject(new Error("Owned browser did not finish the assertion page load"));
      }, 20_000);
      const listener = (message) => {
        if (message.method === "Page.frameNavigated" && message.params.frame.url === url) {
          navigated = true;
        }
        if (navigated && message.method === "Page.loadEventFired") {
          clearTimeout(timer);
          this.listeners.delete(listener);
          resolve();
        }
      };
      this.listeners.add(listener);
    });
  }
}

async function endpoint(directory, browser) {
  const deadline = Date.now() + 15_000;
  while (Date.now() < deadline) {
    if (browser.exitCode !== null) throw new Error("Owned browser exited during startup");
    try {
      const port = (await readFile(join(directory, "DevToolsActivePort"), "utf8")).split("\n")[0];
      const pages = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
      const page = pages.find((entry) => entry.type === "page");
      if (page) return page.webSocketDebuggerUrl;
    } catch (error) {
      if (error.code !== "ENOENT" && !(error instanceof TypeError)) throw error;
    }
    await delay(25);
  }
  throw new Error("Owned browser did not create a debugging endpoint");
}

async function main() {
  if (process.argv.length !== 4 || typeof WebSocket !== "function") {
    throw new Error("Usage: node browser-result.mjs CHROME PAGE.html (Node 24 or newer)");
  }
  const directory = await mkdtemp(join(tmpdir(), "oriha-browser-result-"));
  const browser = spawn(process.argv[2], [
    "--headless=new", "--disable-gpu", "--no-first-run", "--no-default-browser-check",
    "--disable-background-networking", "--force-device-scale-factor=1", "--window-size=800,600",
    `--user-data-dir=${directory}`, "--remote-debugging-address=127.0.0.1",
    "--remote-debugging-port=0", "about:blank",
  ], { stdio: ["ignore", "ignore", "pipe"] });
  let diagnostics = "";
  browser.stderr.on("data", (data) => { diagnostics = (diagnostics + data).slice(-8_000); });
  const closed = new Promise((resolve) => browser.once("close", resolve));
  const started = once(browser, "spawn");
  let socket;
  try {
    await started;
    socket = new WebSocket(await endpoint(directory, browser));
    await new Promise((resolve, reject) => {
      const timer = setTimeout(() => reject(new Error("Browser WebSocket did not open")), 10_000);
      socket.addEventListener("open", () => { clearTimeout(timer); resolve(); }, { once: true });
      socket.addEventListener("error", () => {
        clearTimeout(timer);
        reject(new Error("Browser WebSocket failed to connect"));
      }, { once: true });
    });
    const protocol = new Protocol(socket);
    await protocol.send("Page.enable");
    const url = pathToFileURL(resolve(process.argv[3])).href;
    const loaded = protocol.loaded(url);
    await Promise.all([protocol.send("Page.navigate", { url }), loaded]);
    const response = await protocol.send("Runtime.evaluate", {
      awaitPromise: true,
      returnByValue: true,
      expression: `new Promise((resolve, reject) => {
        const deadline = Date.now() + 15000;
        function inspect() {
          const text = document.getElementById('result')?.textContent;
          if (text) {
            try { resolve(JSON.parse(text)); } catch (error) { reject(error); }
          } else if (Date.now() >= deadline) {
            reject(new Error('Browser assertions did not produce a result'));
          } else { setTimeout(inspect, 25); }
        }
        inspect();
      })`,
    });
    if (response.exceptionDetails) throw new Error(JSON.stringify(response.exceptionDetails));
    if (response.result.value === undefined) throw new Error("Browser returned no assertion result");
    process.stdout.write(JSON.stringify(response.result.value) + "\n");
    // Closing this private browser also closes its renderer processes.
    socket.send(JSON.stringify({ id: protocol.nextId++, method: "Browser.close" }));
  } catch (error) {
    throw new Error(`${error.message}\n${diagnostics}`, { cause: error });
  } finally {
    socket?.close();
    if (browser.exitCode === null) {
      await Promise.race([closed, delay(1_000)]);
      if (browser.exitCode === null) browser.kill("SIGTERM");
      await Promise.race([closed, delay(1_000)]);
      if (browser.exitCode === null) browser.kill("SIGKILL");
    }
    await closed;
    await rm(directory, { recursive: true, force: true });
  }
}

await main();
