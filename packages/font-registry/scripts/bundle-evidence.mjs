// Record modules from the actual Vite/Rollup output, including virtual modules.
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, isAbsolute, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const cache = join(root, ".craftguild/bundle");
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");

export function bundleEvidence() {
  return {
    name: "oriha-font-registry-evidence",
    // Final write hook observes changes made by later generateBundle hooks.
    writeBundle(options, bundle) {
      const inputs = {}, outputs = {}, virtual = {};
      for (const output of Object.values(bundle)) {
        const outputName = relative(root, resolve(options.dir ?? "dist", output.fileName));
        const outputBytes = output.type === "chunk" ? Buffer.from(output.code) : Buffer.from(output.source);
        const snapshot = `.craftguild/bundle/emitted/${hash(outputBytes)}`;
        mkdirSync(dirname(join(root, snapshot)), { recursive: true });
        writeFileSync(join(root, snapshot), outputBytes);
        const contribution = {};
        if (output.type === "chunk") {
          for (const [id, rendered] of Object.entries(output.modules)) {
            if (rendered.renderedLength === 0) continue;
            if (typeof rendered.code !== "string" || rendered.code.length === 0) {
              throw new Error(`Missing rendered module byte evidence: ${id}`);
            }
            const path = id.split("?")[0];
            let name, bytes;
            if (!id.includes("\0") && isAbsolute(path) && existsSync(path)) {
              name = relative(root, path);
              if (name.startsWith("../")) throw new Error(`Bundle input escapes registry: ${id}`);
              bytes = readFileSync(path);
            } else {
              bytes = Buffer.from(rendered.code);
              name = `.craftguild/bundle/inputs/${hash(bytes)}.mjs`;
              mkdirSync(dirname(join(root, name)), { recursive: true });
              writeFileSync(join(root, name), bytes);
              virtual[name] = { id: id.replaceAll(root, "<package>"), sha256: hash(bytes) };
            }
            inputs[name] = { bytes: bytes.length, sha256: hash(bytes) };
            contribution[name] = { bytesInOutput: Buffer.byteLength(rendered.code) };
          }
        }
        outputs[outputName] = { bytes: outputBytes.length, sha256: hash(outputBytes), snapshot, inputs: contribution };
      }
      mkdirSync(join(cache, "parts"), { recursive: true });
      writeFileSync(join(cache, "parts", hash(Buffer.from(Object.keys(outputs).join("\n"))) + ".json"),
        JSON.stringify({ inputs, outputs, virtual }, null, 2) + "\n");
    },
  };
}
