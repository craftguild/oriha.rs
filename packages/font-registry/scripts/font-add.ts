import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import type {
  FontRegistryConfig,
  FontRegistryFontKind,
} from "../src/registry-types.js";

const VALID_KINDS = new Set<FontRegistryFontKind>(["heading", "body", "code"]);
const PACKAGE_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const CONFIG_PATH = path.join(PACKAGE_ROOT, "registry.config.json");

type AddArgs = {
  url: string | null;
  kinds: string[];
};

type SpecimenFont = {
  id: string;
  label: string;
  family: string;
  specimenUrl: string;
};

async function main(): Promise<void> {
  const args = parseArgs(process.argv.slice(2));

  if (!args.url) {
    fail("font:add requires --url with a Google Fonts specimen URL.");
  }

  if (args.kinds.length === 0) {
    fail("font:add requires at least one --kind (heading, body, code).");
  }

  const kinds = args.kinds.map((kind) => {
    if (!isFontKind(kind)) {
      fail(`Invalid --kind "${kind}". Valid values are heading, body, code.`);
    }
    return kind;
  });

  const font = parseSpecimenUrl(args.url);
  const config = JSON.parse(await readFile(CONFIG_PATH, "utf8")) as FontRegistryConfig;
  const existing = config.fonts.find((entry) => entry.id === font.id);

  if (existing) {
    existing.kinds = mergeKinds(existing.kinds, kinds);
  } else {
    config.fonts.push({
      id: font.id,
      label: font.label,
      family: font.family,
      kinds: sortKinds(kinds),
      source: {
        specimenUrl: font.specimenUrl,
        css2Family: `${encodeFamily(font.family)}:wght@400;700`,
      },
    });
  }

  config.fonts.sort((a, b) => a.id.localeCompare(b.id));
  await writeFile(CONFIG_PATH, `${JSON.stringify(config, null, 2)}\n`);

  const action = existing ? "updated" : "added";
  console.log(`${action} ${font.id} (${sortKinds(kinds).join(", ")})`);
}

function parseArgs(argv: string[]): AddArgs {
  const result: AddArgs = { url: null, kinds: [] };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--url") {
      result.url = requireValue(argv, ++index, "--url");
    } else if (arg.startsWith("--url=")) {
      result.url = arg.slice("--url=".length);
    } else if (arg === "--kind") {
      result.kinds.push(...splitKinds(requireValue(argv, ++index, "--kind")));
    } else if (arg.startsWith("--kind=")) {
      result.kinds.push(...splitKinds(arg.slice("--kind=".length)));
    } else {
      fail(`Unknown argument "${arg}".`);
    }
  }

  result.kinds = [...new Set(result.kinds)];
  return result;
}

function requireValue(argv: string[], index: number, flag: string): string {
  const value = argv[index];
  if (!value || value.startsWith("--")) {
    fail(`${flag} requires a value.`);
  }
  return value;
}

function splitKinds(value: string): string[] {
  return value
    .split(",")
    .map((kind) => kind.trim())
    .filter(Boolean);
}

function parseSpecimenUrl(rawUrl: string): SpecimenFont {
  let url: URL;
  try {
    url = new URL(rawUrl);
  } catch {
    fail(`Invalid --url "${rawUrl}".`);
  }

  if (url.hostname !== "fonts.google.com") {
    fail("--url must point to fonts.google.com.");
  }

  const parts = url.pathname.split("/").filter(Boolean);
  if (parts[0] !== "specimen" || !parts[1]) {
    fail("--url must look like https://fonts.google.com/specimen/Source+Sans+3.");
  }

  const family = decodeURIComponent(parts[1]).replace(/\+/g, " ").trim();
  if (!family) {
    fail("Unable to derive a font family from --url.");
  }

  return {
    id: slugify(family),
    label: family,
    family,
    specimenUrl: `https://fonts.google.com/specimen/${encodeFamily(family)}`,
  };
}

function mergeKinds(
  current: readonly FontRegistryFontKind[],
  added: readonly FontRegistryFontKind[],
): FontRegistryFontKind[] {
  return sortKinds([...new Set([...current, ...added])]);
}

function sortKinds(kinds: readonly FontRegistryFontKind[]): FontRegistryFontKind[] {
  const order: Record<FontRegistryFontKind, number> = {
    heading: 0,
    body: 1,
    code: 2,
  };
  return [...kinds].sort((a, b) => order[a] - order[b]);
}

function encodeFamily(family: string): string {
  return family.trim().replace(/\s+/g, "+");
}

function slugify(value: string): string {
  return value
    .normalize("NFKD")
    .replace(/[\u0300-\u036f]/g, "")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

function fail(message: string): never {
  console.error(message);
  process.exit(1);
}

function isFontKind(value: string): value is FontRegistryFontKind {
  return VALID_KINDS.has(value as FontRegistryFontKind);
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
});
