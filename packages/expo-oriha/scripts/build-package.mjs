#!/usr/bin/env node
// Build distribution bytes without changing versions, Git state or npm registry.
import { run } from "./native-build-utils.mjs";

run(process.execPath, ["packages/expo-oriha/scripts/release-security-check.mjs", "--environment-only"]);
run("sh", ["tools/standards.sh", "build", "--artifact", "native-bridge"], {
  env: { ...process.env, TARGET: "", PROFILE: "release" },
});
for (const script of ["generate-heroicons.mjs", "generate-bridge-types.mjs", "generate-uniffi-bindings.mjs"]) {
  run(process.execPath, [`packages/expo-oriha/scripts/${script}`]);
}
run("sh", ["tools/standards.sh", "build", "--artifact", "expo-js"], {
  // Rust selectors belong to the native payload, not its independent JS graph.
  env: { ...process.env, TARGET: "", TOOLCHAIN: "", FEATURES: "", NO_DEFAULT_FEATURES: "" },
});
for (const script of ["build-rust-ios.mjs", "build-rust-android.mjs", "compose-package-sbom.mjs", "pack-dry-run.mjs"]) {
  run(process.execPath, [`packages/expo-oriha/scripts/${script}`]);
}
