#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import fs from "node:fs";
import path from "node:path";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const packageDir = path.resolve(scriptDir, "..");
const repoRoot = path.resolve(packageDir, "../..");
const args = process.argv.slice(2);
if (args.length && (args.length !== 2 || args[0] !== "--output-dir")) {
  throw new Error("Usage: generate-bridge-types.mjs [--output-dir DIRECTORY]");
}
const generatedDir = args.length
  ? path.resolve(args[1])
  : path.join(packageDir, "generated");

execFileSync("sh", ["tools/standards.sh", "verify-receipt", "--artifact", "native-bridge"],
  { cwd: repoRoot, stdio: "inherit" });
const native = JSON.parse(fs.readFileSync(path.join(repoRoot, ".craftguild/oriha-native-bridge/receipt.json"), "utf8"));
const host = native.context.host;
if (!host || native.context.target !== host || native.context.profile !== "release") {
  throw new Error("Generate bridge types from a shared release build for the recorded host");
}

const manifestText = execFileSync(
  "cargo",
  [
    "run",
    "--locked",
    "--release",
    "--target",
    host,
    "--quiet",
    "--manifest-path",
    path.join(repoRoot, "Cargo.toml"),
    "-p",
    "oriha-native-bridge",
    "--bin",
    "oriha-native-bridge-manifest",
  ],
  {
    cwd: repoRoot,
    encoding: "utf8",
    env: {
      ...process.env,
      ...(native.context.toolchain ? { RUSTUP_TOOLCHAIN: native.context.toolchain } : {}),
      CARGO_TARGET_DIR: path.join(repoRoot, ".craftguild/generators/bridge-manifest"),
    },
  },
);

const manifest = JSON.parse(manifestText);
validateManifest(manifest);

fs.mkdirSync(generatedDir, { recursive: true });
writeGenerated("generatedBridgeManifest.json", `${JSON.stringify(manifest, null, 2)}\n`);
writeGenerated("generatedBridgeEnums.ts", generatedBridgeEnums(manifest));
writeGenerated("generatedBridgeTypes.ts", generatedBridgeTypes(manifest));
removeGenerated("generatedBridgeOptions.ts");

function writeGenerated(fileName, contents) {
  fs.writeFileSync(path.join(generatedDir, fileName), `${contents.trimEnd()}\n`);
}

function removeGenerated(fileName) {
  const filePath = path.join(generatedDir, fileName);
  if (fs.existsSync(filePath)) {
    fs.unlinkSync(filePath);
  }
}

function generatedBridgeEnums(manifest) {
  const out = [header()];

  for (const enumSpec of manifest.enums) {
    out.push(emitEnum(enumSpec));
  }

  for (const unionSpec of manifest.stringUnions) {
    out.push(emitStringUnion(unionSpec));
  }
  return out.join("");
}

function generatedBridgeTypes(manifest) {
  const names = [...manifest.enums, ...manifest.stringUnions].map((spec) => spec.name);
  const out = [
    header(),
    `import type { ${names.join(", ")} } from "./generatedBridgeEnums";\n`,
    'export * from "./generatedBridgeEnums";\n\n',
  ];

  for (const recordSpec of manifest.records) {
    out.push(emitRecord(recordSpec));
  }

  if (manifest.functions.length > 0) {
    out.push(emitFunctionMap("BridgeFunctionMap", manifest.functions));
  }
  return out.join("");
}

function emitEnum(spec) {
  return [
    `export enum ${spec.name} {\n`,
    ...spec.variants.map(
      (variant) => `  ${variant.name} = ${quote(variant.value)},\n`,
    ),
    "}\n\n",
  ].join("");
}

function emitStringUnion(spec) {
  return [
    `export type ${spec.name} =\n`,
    spec.values.map((value) => `  | ${quote(value)}`).join("\n"),
    ";\n\n",
  ].join("");
}

function emitRecord(spec) {
  const genericParams = spec.genericParams?.length
    ? `<${spec.genericParams.join(", ")}>`
    : "";
  const out = [`export type ${spec.name}${genericParams} = {\n`];
  for (const field of spec.fields) {
    out.push(
      `  ${field.name}${field.optional ? "?" : ""}: ${typeRefToTs(field.typeRef)};\n`,
    );
  }
  if (spec.indexSignature) {
    out.push(
      `  [${spec.indexSignature.keyName}: ${spec.indexSignature.keyType}]: ${typeRefToTs(spec.indexSignature.valueType)};\n`,
    );
  }
  out.push("};\n\n");
  return out.join("");
}

function emitFunctionMap(name, functions) {
  const out = [`export type ${name} = {\n`];
  for (const fn of functions) {
    const params = fn.params
      .map((param) => `${param.name}: ${typeRefToTs(param.typeRef)}`)
      .join(", ");
    const returns = typeRefToTs(fn.returns);
    out.push(
      `  ${fn.name}(${params}): ${fn.asyncResult ? `Promise<${returns}>` : returns};\n`,
    );
  }
  out.push("};\n");
  return out.join("");
}

function typeRefToTs(typeRef) {
  switch (typeRef.kind) {
    case "primitive":
    case "named":
    case "generic":
      return typeRef.name;
    case "genericInstance":
      return `${typeRef.name}<${typeRef.items.map(typeRefToTs).join(", ")}>`;
    case "array":
      return `${wrapArrayItem(typeRefToTs(typeRef.item))}[]`;
    case "nullable":
      return `${typeRefToTs(typeRef.item)} | null`;
    case "map":
      return `Record<${typeRef.name}, ${typeRefToTs(typeRef.item)}>`;
    case "union":
      return typeRef.items.map(typeRefToTs).join(" | ");
    case "unknown":
      return "unknown";
    case "undefined":
      return "undefined";
    default:
      throw new Error(`Unsupported type ref kind: ${typeRef.kind}`);
  }
}

function wrapArrayItem(typeName) {
  return typeName.includes(" | ") ? `(${typeName})` : typeName;
}

function header() {
  return "// @generated by scripts/generate-bridge-types.mjs. Do not edit.\n\n";
}

function quote(value) {
  return JSON.stringify(value);
}

function validateManifest(manifest) {
  if (manifest.schemaVersion !== 1) {
    throw new Error(`Unsupported bridge manifest schemaVersion: ${manifest.schemaVersion}`);
  }
  if (!Array.isArray(manifest.enums)) {
    throw new Error("Bridge manifest must contain an enums array");
  }
  if (!Array.isArray(manifest.stringUnions)) {
    throw new Error("Bridge manifest must contain a stringUnions array");
  }
  if (manifest.catalogs !== undefined && !Array.isArray(manifest.catalogs)) {
    throw new Error("Bridge manifest catalogs must be an array when present");
  }
  if (!Array.isArray(manifest.records)) {
    throw new Error("Bridge manifest must contain a records array");
  }
  if (!Array.isArray(manifest.functions)) {
    throw new Error("Bridge manifest must contain a functions array");
  }
}
