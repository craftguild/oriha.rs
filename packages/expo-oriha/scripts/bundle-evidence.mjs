// Convert actual Rolldown module contribution into the pinned SBOM input format.
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, isAbsolute, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const cache = join(root, ".craftguild/bundle");
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");

function local(file) {
  const name = relative(root, file).replaceAll("\\", "/");
  if (name.startsWith("../") || isAbsolute(name)) {
    throw new Error(`Bundled source escapes the Expo package: ${file}`);
  }
  return name;
}

export function bundleEvidence() {
  return {
    name: "oriha-bundle-evidence",
    generateBundle(options, bundle) {
      const inputs = {};
      const outputs = {};
      const virtual = {};
      for (const output of Object.values(bundle)) {
        const name = local(resolve(options.dir ?? join(root, "dist"), output.fileName));
        if (output.type !== "chunk") {
          throw new Error(`Bundle asset needs explicit ownership: ${name}`);
        }
        if (/\.d\.(?:ts|mts|cts)$/.test(name)) {
          outputs[name] = { bytes: Buffer.byteLength(output.code), inputs: {} };
          continue;
        }
        const contributions = {};
        for (const [id, rendered] of Object.entries(output.modules)) {
          if (rendered.renderedLength === 0) continue;
          if (typeof rendered.code !== "string" || rendered.code.length === 0) {
            throw new Error(`Missing rendered module code: ${id}`);
          }
          let file;
          let bytes;
          if (!id.includes("\0") && isAbsolute(id) && existsSync(id)) {
            file = local(id);
            bytes = readFileSync(id);
          } else {
            bytes = Buffer.from(rendered.code);
            file = `.craftguild/bundle/inputs/${hash(bytes)}.mjs`;
            mkdirSync(dirname(join(root, file)), { recursive: true });
            writeFileSync(join(root, file), bytes);
            virtual[file] = { id: id.replaceAll(root, "<package>"), sha256: hash(bytes) };
          }
          inputs[file] = { bytes: bytes.length, sha256: hash(bytes) };
          contributions[file] = { bytesInOutput: Buffer.byteLength(rendered.code) };
        }
        outputs[name] = { bytes: Buffer.byteLength(output.code), inputs: contributions };
      }
      mkdirSync(join(cache, "parts"), { recursive: true });
      const key = hash(Buffer.from(Object.keys(outputs).sort().join("\n")));
      writeFileSync(join(cache, "parts", key + ".json"), JSON.stringify({
        inputs, outputs, virtual, contribution: "UTF-8 byte length of Rolldown rendered module code",
      }, null, 2) + "\n");
    },
  };
}
