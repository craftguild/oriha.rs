import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { injectedChunk } from "../scripts/deployment-injection.mjs";
import { digest } from "../scripts/registry-assets.mjs";

for (const quote of ["'", '"']) {
  test(`only the known JSON manifest insertion preserves module evidence (${quote})`, () => {
    const root = mkdtempSync(join(tmpdir(), "oriha-manifest-proof-"));
    try {
      mkdirSync(join(root, "snapshots"));
      const marker = `${quote}@@ASTRO_MANIFEST_REPLACE@@${quote}`;
      const original = `const manifest = deserializeManifest((${marker}));\nexport { manifest };\n`;
      writeFileSync(join(root, "snapshots/entry"), original);
      const parts = [{ outputs: { "dist/server/entry.mjs": { snapshot: "snapshots/entry", inputs: {} } }, inputs: {}, virtual: {} }];
      const value = JSON.stringify({ adapterName: "@astrojs/vercel", routes: [] });
      const actual = original.replace(marker, value);
      const result = injectedChunk(root, "_functions/entry.mjs", Buffer.from(actual), parts, digest);
      assert.equal(result.length, 1);
      assert.equal(Object.keys(result[0].virtual).length, 1);
      assert.equal(injectedChunk(root, "_functions/entry.mjs", Buffer.from(actual + "unrelated();"), parts, digest), null);
      assert.throws(() => injectedChunk(root, "_functions/entry.mjs", Buffer.from(actual.replace(value, "runCode()")), parts, digest), SyntaxError);
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });
}
