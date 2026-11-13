# Artifact preparation and provenance

[ADR 0004](../adr/0004-standards-v0.5-integration.md) defines v0.6.2 integration and its local completion criteria. Earlier v0.2.0 results remain historical; current qualification is recorded against the final source in Beads.

Oriha's application package is `@craftguild/expo-oriha`. Its npm name
and version remain separate from the pinned Craftguild standards version.
`oriha` is also available as a standalone local renderer CLI.

## Distribution boundaries

| Artifact | Actual payload | Evidence boundary |
| --- | --- | --- |
| `oriha` CLI | One selected-target executable, including its renderer, math font and generated data | Embedded CycloneDX JSON, selected Cargo runtime graph and supplemental assets |
| Expo npm package | ESM/CJS JavaScript, declarations, native iOS/Android libraries, generated bindings, platform wrappers and notices | Actual bundle modules and output bytes, native artifact records and package inventory |
| Native bridge library | One architecture and feature selection of the static/dynamic Rust bridge | Target-filtered normal Cargo dependencies, embedded font/data inputs and associated binary hashes |
| Font registry deployment | Independently built server/browser code and the generated font registry assets | Its own bundle graph, installed asset hashes and retained font licenses |
| Optional Wasm/Windows bridge | The explicitly built target's library and binding outputs | Its own target graph and output inventory; these are not part of the Expo npm payload |

The manifest emitter, UniFFI generator, PlantUML fixture CLI and stats utilities
are repository tools. They are not included in the Expo package's `files`
inventory. Preserve their machine-readable output and operational contracts;
do not count their dependencies as if they were bundled application code.

Cargo crate publication remains disabled. Git dependencies use the complete
public HTTPS revision, and builds do not require another product checkout.

## Preparation order

1. Resolve the artifact, target, profile, features and output paths in the shared
   engine. Prepare invocation-local evidence and retain the last verified output
   and receipt if replacement fails; never report an old receipt as a new success.
2. Build JavaScript with the real bundler. Record contributing modules and exact
   output lengths. Type declarations are sealed output assets, rather than
   evidence that a development-only type compiler is shipped at runtime.
3. Select each Rust library's real library target, effective features and normal
   dependency closure. Exclude build, development and proc-macro edges. Prepare
   and schema-validate its SBOM before compiler execution.
4. Verify the selected compiler, target, profile, features and sealed inputs
   during compilation. Build scripts observe and verify; they never run Cargo.
5. Associate the produced native binary hashes with their SBOMs. Compose the npm
   inventory from actual JavaScript and native outputs, bindings and notices.
   Fail when a required target or asset is missing.

An SBOM does not hash its own bytes or the final binary that embeds it. Native
artifact association records and the containing package inventory provide the
subsequent output hashes. Keep those boundaries explicit to avoid self-hashing
cycles or describing a helper executable as a library.

Standards engine v0.6.2 at revision `be63eefb44a850a01f4cc065c66fff2fe558ee18` owns executable and library preparation, compilation, output verification and receipts. Configuration schema 2 declares actual Cargo library names and formats; the normative rules baseline remains 0.1.0. Oriha does not maintain a parallel Rust execution adapter.

The Makefile includes `standards/make/common.mk` and forwards product aliases.
The thin launcher verifies the unique `standards/engine.json` and cached full
engine. Init or explicit `make setup` acquires only that fixed macOS ARM64 or
Linux x86_64 GNU engine; ordinary operations never acquire/update it. A standalone
checkout requires neither copied archives/rules, a sibling checkout nor a global
engine. `make prime` renders matching offline guidance. The compatibility shell
supplies only the product Node/counter environment, with no host/archive loader.
Install refuses a missing verified cache rather than acquiring or compiling tools.
Native and cross Rust outputs use `<target-dir>/<target-triple>/<profile>/<file>`.

Product operations retain native SDK checks, typed binding generation, package
composition, exact generated reproduction and ordinary versus acceptance scopes.
Shared preparation around test/check is declared in `craftguild.toml`; specialized
commands must not repeat the shared build sequence. Debug identity is prepared as
debug from the start. The same output resolver serves build and copy-only install.

New consumer SBOMs contain artifact identity, effective features and actual shipped
components. Source file inventories and hashes remain in local seals/receipts.
The Expo common ownership manifest is derived under `.craftguild/ownership` from
its public generation recipe, current inputs and actual outputs. Exact isolated
regeneration remains required. Refreshing that cache after version preparation
must not create an unrelated tracked edit or another version increment.

External Expo/React Native peers are supplied by the consuming application.
Distinguish their declared requirements from code actually included by the
bundler. A library cannot assert the consumer's eventual peer versions. Font
registry fixtures are test assets and must never stand in for production fonts.

The registry deployment uses actual final Vercel files, including traced runtime
modules, copied assets and Astro's late JSON-manifest insertion. Verify that
insertion changes only the known JSON placeholder; arbitrary chunk changes fail.
Retain per-family CSS, source URLs, byte hashes and original license text, plus
notices for actual runtime packages. Reuse cached production fonts only after
configuration and complete inventory/hash verification. The registry has no image
transformation feature, so it explicitly selects Astro's no-op image service to
avoid shipping an unused host-specific libvips binary.

## Stable native interface

UniFFI incorporates Rust module paths in type/function metadata. Native record
definitions remain in the existing `types` namespace. Cohesive definition files
are included into that namespace and measured individually as implementation.
Exported FFI functions also keep their original namespace. Ordinary internal
conversions and rendering operations use child modules.

Reproduce the manifest and Swift/Kotlin bindings, verify their exported library
checksums and compare generated outputs when changing these boundaries. Do not
update binding expectations merely because a refactor altered metadata.

## Lifecycle scope

Preparation, compilation, installation and release preparation are transient
developer operations. Git revisions and sealed artifact/release records provide
their provenance. They do not introduce business Header/Detail records or a new
persisted state machine. Existing renderer snapshots, telemetry sessions,
generation guards, storage formats and transition behavior remain unchanged.

Installation copies an already verified executable to the requested staged
destination. Release preparation delegates version selection and notes to the
pinned standards tool. Committing, signing, tagging and publishing remain
explicit operations by the repository owner.
