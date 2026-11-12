# Agent Notes

This repository is edited by multiple agents and humans at the same time.
Never revert or rewrite changes you did not make. Before editing, check the
current diff and keep your write scope narrow.

## Commit Messages

Commits on `main` drive
`packages/expo-oriha/CHANGELOG.md` through
`commit-and-tag-version`, so commit messages must describe the actual change.
Follow Conventional Commits:

- `feat: ...` user-visible feature or rendering capability (minor bump)
- `fix: ...` bug fix (patch bump)
- `chore: ...` / `ci: ...` / `doc: ...` / `refactor: ...` no release impact
- Append `!` or add a `BREAKING CHANGE:` footer for contract-breaking changes

State what changed and, when not obvious, why. Subjects like `fix: 不具合修正`
or `fix: 見た目の調整` are not acceptable; name the component and behavior
instead, for example `fix: ネストしたリストのインデント幅を修正`.
Japanese subjects are fine; keep the type prefix in English.

## Expo Bridge Development Flow

The intended ownership model is:

1. Rust owns the real business logic and the source-of-truth option catalogs.
2. UniFFI exposes the minimal native boundary.
3. The Expo native modules are thin Swift/Kotlin transport layers.
4. TypeScript types and option catalogs are generated from a Rust-emitted
   intermediate JSON manifest.

The preferred transport between TypeScript and the Expo native module is JSON
strings. The native module passes those strings through to the Rust UniFFI JSON
functions. Avoid duplicating plan policy, feature gating, catalog filtering, or
render-setting normalization in TypeScript.

## Where Things Live

- Rust bridge and plan policy:
  `crates/native-bridge/src/lib.rs`
- Rust manifest emitter for TypeScript generation:
  `crates/native-bridge/src/bin/oriha-native-bridge-manifest.rs`
- TypeScript code generator:
  `packages/expo-oriha/scripts/generate-bridge-types.mjs`
- Generated manifest/types:
  `packages/expo-oriha/generated/generatedBridgeManifest.json`
  `packages/expo-oriha/generated/generatedBridgeTypes.ts`
- Android thin native bridge:
  `packages/expo-oriha/android/src/main/java/co/progressiveworks/oriha/`
- iOS thin native bridge:
  `packages/expo-oriha/ios/`
- Public TypeScript wrapper:
  `packages/expo-oriha/src/ExpoOrihaModule.ts`
  `packages/expo-oriha/src/render.ts`
  `packages/expo-oriha/src/types.ts`
  `packages/expo-oriha/src/index.ts`

## When Changing Rust Core Behavior

If the change only affects HTML/CSS/rendering behavior inside `crates/core`,
update Rust tests and run the relevant Rust checks. If the change affects data
that crosses into Expo, continue through the bridge generation steps below.

Useful checks:

```sh
cargo fmt -p oriha-core -p oriha-native-bridge
cargo test -p oriha-core -p oriha-native-bridge
```

For PlantUML or renderer-adjacent changes, also keep the Android native build in
mind because the Expo package embeds the Rust bridge as native libraries:

```sh
npm --prefix packages/expo-oriha run build:rust:android
```

## When Changing UniFFI-Exposed Rust API

Use this flow when adding or changing exported functions, records, enums, JSON
payloads, option catalogs, or plan policy:

1. Update `crates/native-bridge/src/lib.rs`.
   - Keep plan and feature policy in Rust.
   - Prefer JSON-exported UniFFI functions for deeply nested bridge payloads.
   - Keep typed Rust structs internally where they help correctness.
2. Update `crates/native-bridge/src/bin/oriha-native-bridge-manifest.rs`.
   - Add or update enum specs, string unions, record specs, function specs, and
     catalogs.
   - The manifest should describe the TypeScript-facing API shape.
   - Do not hand-write generated TypeScript as the source of truth.
3. Regenerate TypeScript bridge artifacts:

```sh
npm --prefix packages/expo-oriha run generate:bridge
```

This emits:

- `packages/expo-oriha/generated/generatedBridgeManifest.json`
- `packages/expo-oriha/generated/generatedBridgeTypes.ts`

4. Regenerate UniFFI Swift/Kotlin bindings:

```sh
npm --prefix packages/expo-oriha run generate:uniffi
```

5. If a new JSON function is exposed, wire the thin native transport manually.
   Keep these files as pass-through glue:
   - Android `ExpoOrihaModule.kt`
   - Android `OrihaRustBridge.kt`
   - iOS `ExpoOrihaModule.swift`
   - iOS `OrihaRustBridge.swift`
6. Update the TypeScript wrapper only at the API boundary.
   - Add methods to `ExpoOrihaModule.ts`.
   - Parse/stringify JSON in `render.ts` or a similarly narrow wrapper.
   - Re-export generated types/options from `types.ts` and `index.ts`.

## Native Artifact Builds

After Rust bridge changes, regenerate Android shared libraries:

```sh
npm --prefix packages/expo-oriha run build:rust:android
```

For iOS native artifacts:

```sh
npm --prefix packages/expo-oriha run build:rust:ios
```

The generated native artifacts live under ignored package directories such as:

- `packages/expo-oriha/android/src/main/jniLibs/**/liboriha_native_bridge.so`
- `packages/expo-oriha/ios/native/OrihaNativeBridge.xcframework`

## Validation Checklist

Run the smallest relevant checks first, then broaden before handing off.

Common bridge/core checks:

```sh
cargo test -p oriha-core -p oriha-native-bridge
npm --prefix packages/expo-oriha exec -- tsc -p tsconfig.json --noEmit
```

Example app typecheck:

```sh
./node_modules/.bin/tsc -p tsconfig.json --noEmit
```

Run that from `examples/expo-preview`.

Android bundle smoke check:

```sh
npx expo export --platform android --output-dir /tmp/oriha-expo-export --clear
```

Run that from `examples/expo-preview`. Use a unique temporary output directory
when running repeatedly.

Final whitespace check:

```sh
git diff --check
```

If UniFFI-generated Kotlin introduces trailing whitespace, fix only formatting
noise. Do not edit generated binding semantics by hand.

## Generated Files Rule

Do not manually maintain generated files as source-of-truth:

- `packages/expo-oriha/generated/generatedBridge*.ts`
- `packages/expo-oriha/generated/generatedBridgeManifest.json`
- `packages/expo-oriha/android/src/main/java/uniffi/**`
- `packages/expo-oriha/ios/generated/**`

It is acceptable to inspect generated files to verify that Rust manifest changes
landed correctly. If generated output is wrong, fix the Rust manifest emitter or
the generator script, then regenerate.

## Multi-Agent Safety

- Check `git status --short` before and after your work.
- Do not run broad cleanup commands.
- Do not restore, checkout, or delete files outside your task scope.
- If a file has unrelated edits, preserve them and make the smallest compatible
  change.
- Mention any generated files you updated in the handoff.
