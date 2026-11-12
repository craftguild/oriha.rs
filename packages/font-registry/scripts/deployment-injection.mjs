// Astro inserts its JSON manifest after Rollup writes the entrypoint chunk.
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { basename, dirname, join } from "node:path";

export function injectedChunk(root, name, bytes, parts, digest) {
  const markers = ["'@@ASTRO_MANIFEST_REPLACE@@'", '"@@ASTRO_MANIFEST_REPLACE@@"'];
  const actual = bytes.toString("utf8");
  const matches = [];
  for (const part of parts) {
    for (const [source, output] of Object.entries(part.outputs)) {
      if (basename(source) !== basename(name)) continue;
      const original = readFileSync(join(root, output.snapshot), "utf8");
      const marker = markers.find((value) => original.includes(value));
      if (!marker) continue;
      const sections = original.split(marker);
      if (sections.length !== 2 || !actual.startsWith(sections[0]) || !actual.endsWith(sections[1])) continue;
      const replacement = actual.slice(sections[0].length, actual.length - sections[1].length);
      const manifest = JSON.parse(replacement);
      if (manifest.adapterName !== "@astrojs/vercel" || !Array.isArray(manifest.routes)) {
        throw new Error("Unexpected injected Astro manifest");
      }
      const payload = Buffer.from(replacement);
      const snapshot = `.craftguild/bundle/inputs/${digest(payload)}.json`;
      mkdirSync(dirname(join(root, snapshot)), { recursive: true });
      writeFileSync(join(root, snapshot), payload);
      matches.push({
        output: { ...output, inputs: { ...output.inputs, [snapshot]: { bytesInOutput: payload.length } } },
        inputs: { ...part.inputs, [snapshot]: { bytes: payload.length, sha256: digest(payload) } },
        virtual: { ...part.virtual, [snapshot]: { id: "\0virtual:astro:injected-manifest", sha256: digest(payload) } },
      });
    }
  }
  if (matches.length > 1) throw new Error(`Ambiguous injected bundle: ${name}`);
  return matches.length ? matches : null;
}
