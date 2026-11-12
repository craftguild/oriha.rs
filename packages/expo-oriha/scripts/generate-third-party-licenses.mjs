#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import {
  existsSync,
  copyFileSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { dirname, isAbsolute, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const packageRoot = resolve(scriptDir, "..");
const repositoryRoot = resolve(packageRoot, "../..");
const args = process.argv.slice(2);
if (args.length && (args.length !== 2 || args[0] !== "--output-dir")) {
  throw new Error("Usage: generate-third-party-licenses.mjs [--output-dir DIRECTORY]");
}
const outputDir = args.length ? resolve(args[1]) : join(packageRoot, "licenses");
const outputPath = join(outputDir, "third-party-licenses.json");
const targetCrateName = "oriha-native-bridge";
const githubGemojiCommit = "0eca75db9301421efc8710baf7a7576793ae452a";
const cldrJsonVersion = "48.2.0";
const unicodeEmojiVersion = "16.0";
const cargoTargets = [
  "aarch64-apple-ios",
  "aarch64-apple-ios-sim",
  "x86_64-apple-ios",
  "aarch64-linux-android",
  "armv7-linux-androideabi",
  "x86_64-linux-android",
];

function readJsonFromCommand(command, args, cwd) {
  const output = execFileSync(command, args, {
    cwd,
    encoding: "utf8",
    maxBuffer: 1024 * 1024 * 100,
  });

  return JSON.parse(output);
}

function cargoMetadata() {
  return readJsonFromCommand(
    "cargo",
    ["metadata", "--format-version", "1", "--locked"],
    repositoryRoot,
  );
}

function cargoTreeOutput(target) {
  return execFileSync(
    "cargo",
    [
      "tree",
      "-p",
      targetCrateName,
      "--target",
      target,
      "--edges",
      "normal,no-proc-macro",
      "--prefix",
      "none",
      "--format",
      "{p}",
      "--locked",
    ],
    {
      cwd: repositoryRoot,
      encoding: "utf8",
      maxBuffer: 1024 * 1024 * 100,
    },
  );
}

function cargoTreePackageKeys() {
  const packageKeys = new Set();

  for (const target of cargoTargets) {
    for (const line of cargoTreeOutput(target).split("\n")) {
      const match = line.match(/^([^ ]+) v([^ ]+)/);

      if (!match) {
        continue;
      }

      packageKeys.add(`${match[1]}@${match[2]}`);
    }
  }

  return packageKeys;
}

function readTextFiles(directory, matcher) {
  if (!existsSync(directory)) {
    return [];
  }

  return readdirSync(directory)
    .filter((fileName) => matcher.test(fileName))
    .sort((a, b) => a.localeCompare(b))
    .map((fileName) => {
      const filePath = join(directory, fileName);

      if (!statSync(filePath).isFile()) {
        return "";
      }

      const content = readFileSync(filePath, "utf8").trim();

      if (!content) {
        return "";
      }

      return `${fileName}\n\n${content}`;
    })
    .filter(Boolean);
}

function readExplicitLicenseFile(packageData, packageDirectory) {
  if (!packageData.license_file) {
    return "";
  }

  const licenseFilePath = isAbsolute(packageData.license_file)
    ? packageData.license_file
    : join(packageDirectory, packageData.license_file);

  if (!existsSync(licenseFilePath) || !statSync(licenseFilePath).isFile()) {
    return "";
  }

  return readFileSync(licenseFilePath, "utf8").trim();
}

function licenseText(packageData) {
  const packageDirectory = dirname(packageData.manifest_path);
  const texts = [
    readExplicitLicenseFile(packageData, packageDirectory),
    ...readTextFiles(
      packageDirectory,
      /^(licen[cs]e|copying|copyright|unlicense)([-_.].*)?$/i,
    ),
  ].filter(Boolean);

  return Array.from(new Set(texts)).join("\n\n");
}

function noticeText(packageData) {
  return readTextFiles(
    dirname(packageData.manifest_path),
    /^(notice|third_party_notices)([-_.].*)?$/i,
  ).join("\n\n");
}

function cargoEntry(packageData) {
  return {
    source: "cargo",
    name: packageData.name,
    version: packageData.version,
    license: packageData.license ?? "",
    repository: packageData.repository ?? "",
    homepage: packageData.homepage ?? "",
    licenseText: licenseText(packageData),
    noticeText: noticeText(packageData),
  };
}

function vendoredLicenseText(relativePath) {
  return readFileSync(join(repositoryRoot, relativePath), "utf8").trim();
}

function vendoredEntry({
  name,
  version,
  license,
  repository = "",
  homepage = "",
  licensePath,
}) {
  return {
    source: "vendored",
    name,
    version,
    license,
    repository,
    homepage,
    licenseText: vendoredLicenseText(licensePath),
    noticeText: "",
  };
}

function vendoredEntries() {
  const heroicons = JSON.parse(readFileSync(join(packageRoot, "node_modules/heroicons/package.json"), "utf8"));
  const locked = JSON.parse(readFileSync(join(packageRoot, "package-lock.json"), "utf8"));
  if (heroicons.version !== locked.packages["node_modules/heroicons"].version) {
    throw new Error("Installed Heroicons differs from the package lockfile");
  }
  return [
    vendoredEntry({
      name: "rolldown-runtime",
      version: JSON.parse(readFileSync(join(packageRoot, "node_modules/rolldown/package.json"), "utf8")).version,
      license: "MIT",
      repository: "https://github.com/rolldown/rolldown",
      licensePath: "packages/expo-oriha/node_modules/rolldown/LICENSE",
    }),
    vendoredEntry({
      name: "STIX Two Math",
      version: "v2.13b171",
      license: "OFL-1.1",
      repository: "https://github.com/stipub/stixfonts",
      licensePath: "crates/latex/assets/fonts/OFL.txt",
    }),
    vendoredEntry({
      name: "WHATWG character references",
      version: "sha256-d741d877ac77c4194c4ad526b5b4a19aef8dfe411ab840a466891cdbb9f362e6",
      license: "BSD-3-Clause",
      homepage: "https://html.spec.whatwg.org/entities.json",
      licensePath: "crates/core/entity-data/LICENSE-WHATWG",
    }),
    vendoredEntry({
      name: "heroicons",
      version: heroicons.version,
      license: heroicons.license,
      repository: "https://github.com/tailwindlabs/heroicons",
      licensePath: "packages/expo-oriha/node_modules/heroicons/LICENSE",
    }),
    vendoredEntry({
      name: "github-gemoji-emoji-data",
      version: githubGemojiCommit,
      license: "MIT",
      repository: "https://github.com/github/gemoji",
      licensePath: `crates/core/emoji-data/github-gemoji-${githubGemojiCommit}/LICENSE`,
    }),
    vendoredEntry({
      name: "unicode-cldr-json-annotations",
      version: cldrJsonVersion,
      license: "Unicode-3.0",
      repository: "https://github.com/unicode-org/cldr-json",
      licensePath: `crates/core/emoji-data/unicode-cldr-json-${cldrJsonVersion}/LICENSE`,
    }),
    vendoredEntry({
      name: "unicode-emoji-test-data",
      version: unicodeEmojiVersion,
      license: "Unicode-3.0",
      homepage: `https://unicode.org/Public/emoji/${unicodeEmojiVersion}/emoji-test.txt`,
      licensePath: `crates/core/emoji-data/unicode-emoji-${unicodeEmojiVersion}/LICENSE`,
    }),
  ];
}

function main() {
  const metadata = cargoMetadata();
  const rootPackage = metadata.packages.find(
    (packageData) => packageData.name === targetCrateName && packageData.source === null,
  );

  if (!rootPackage) {
    throw new Error(`Could not find workspace crate: ${targetCrateName}`);
  }

  const runtimePackageKeys = cargoTreePackageKeys();
  const cargoEntries = metadata.packages
    .filter(
      (packageData) =>
        packageData.source !== null &&
        runtimePackageKeys.has(`${packageData.name}@${packageData.version}`),
    )
    .map(cargoEntry)
    .filter((entry) => entry.license || entry.licenseText || entry.noticeText);
  const entries = [...cargoEntries, ...vendoredEntries()]
    .sort((a, b) => {
      const nameComparison = a.name.localeCompare(b.name);
      return nameComparison === 0
        ? a.version.localeCompare(b.version)
        : nameComparison;
    });

  mkdirSync(dirname(outputPath), { recursive: true });
  copyFileSync(
    join(packageRoot, "node_modules/heroicons/LICENSE"),
    join(outputDir, "heroicons-LICENSE"),
  );
  copyFileSync(
    join(packageRoot, "node_modules/rolldown/LICENSE"),
    join(outputDir, "rolldown-LICENSE"),
  );
  writeFileSync(
    outputPath,
    `${JSON.stringify({ schemaVersion: 1, entries }, null, 2)}\n`,
  );

  console.log(
    `Generated ${entries.length} third-party license entries (${cargoEntries.length} Cargo, ${vendoredEntries().length} vendored).`,
  );
}

main();
