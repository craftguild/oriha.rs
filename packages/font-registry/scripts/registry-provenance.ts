// Retain the exact upstream license and downloaded-byte provenance per family.
import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

type LicenseRecord = {
  id: string; family: string; license: string; file: string; source: string; sha256: string;
};
type AssetRecord = { source: string; url: string; sha256: string; size: number };
type FamilyRecord = {
  id: string; cssSource: string; cssFile: string; cssSha256: string; files: AssetRecord[];
  license: LicenseRecord & { url: string };
};

export class RegistryProvenance {
  private families: FamilyRecord[] = [];

  constructor(private packageRoot: string, private destination: string, private configSha256: string) {}

  async retain(id: string, family: string, cssSource: string, css: string, files: AssetRecord[]): Promise<void> {
    const manifest: { revision: string; fonts: LicenseRecord[] } = JSON.parse(
      await readFile(path.join(this.packageRoot, "licenses/manifest.json"), "utf8"),
    );
    const record = manifest.fonts.find((record) => record.id === id && record.family === family);
    if (!record || !/^[0-9a-f]{40}$/.test(manifest.revision) ||
        !record.source.startsWith(`https://raw.githubusercontent.com/google/fonts/${manifest.revision}/`)) {
      throw new Error(`Missing pinned upstream license: ${id}`);
    }
    const absolute = path.resolve(this.packageRoot, record.file);
    if (!absolute.startsWith(path.join(this.packageRoot, "licenses") + path.sep)) {
      throw new Error(`License escapes its owned directory: ${id}`);
    }
    const bytes = await readFile(absolute);
    if (digest(bytes) !== record.sha256) throw new Error(`Changed font license: ${id}`);
    const url = `fonts/${id}/${path.basename(record.file)}`;
    await mkdir(path.dirname(path.join(this.destination, url)), { recursive: true });
    await writeFile(path.join(this.destination, url), bytes);
    const cssFile = `sources/${id}.css`;
    await mkdir(path.join(this.destination, "sources"), { recursive: true });
    await writeFile(path.join(this.destination, cssFile), css);
    this.families.push({
      id, cssSource, cssFile, cssSha256: digest(Buffer.from(css)), files,
      license: { ...record, url },
    });
  }

  async finish(): Promise<void> {
    await writeFile(path.join(this.destination, "provenance.json"), JSON.stringify({
      schemaVersion: 1,
      configSha256: this.configSha256,
      scope: "Actual Google Fonts CSS2 downloads and retained pinned upstream license text. CDN byte identity is recorded independently of the source license revision.",
      fonts: this.families,
    }, null, 2) + "\n");
  }
}

function digest(bytes: Buffer): string {
  return createHash("sha256").update(bytes).digest("hex");
}
