# Development

`make build` uses the shared engine to prepare the selected CLI SBOM, verify its
inputs, and write `target/<selected-target>/release/oriha`. `PROFILE=debug` prepares
and builds debug identity directly under the corresponding `debug` directory.
`make prepare` prepares the CLI
and host library records for direct release-profile Cargo tests. Changed sources,
features, toolchains or targets require preparation again. Build scripts refuse
missing/stale records; they never recursively run Cargo.

## Tools and commands

The entry selects standards engine v0.6.1 at complete public HTTPS revision
`6852b210a9a614920bc5be2187f2705f46db978d`, recorded in
`standards/engine.json`. The normative rules baseline remains 0.1.0.
Explicit `make setup` verifies and caches the matching macOS ARM64 or Linux x86_64
GNU engine under `.craftguild/engine/`; `make prime` renders its offline rules.
The committed thin Make/Python entry needs no sibling checkout, global engine or
copied rules/binary archive. Ordinary operations never acquire/update that engine.
Use `cargo fetch --locked` and the locked npm installations explicitly before
offline builds. Engine setup needs Python 3 and curl. Install Rust 1.95.0 through rustup, tokei 15.0.0,
cargo-deny and Node 24+. `make check` additionally needs the three npm projects'
locked installations and Chrome/Chromium for image decoding.

| Entry | Scope |
| --- | --- |
| `make setup` / `make prime` | Explicit pinned-engine acquisition / offline matching rules |
| `make build` / `make build-cli` | Locked CLI release build and embedded SBOM |
| `make build-expo` | Actual JS/declarations, iOS/Android native payload, composed SBOM and tarball inspection |
| `make install` | Copy an existing CLI binary; fail when it is missing |
| `make test` | Rust workspace, Expo unit tests and font fixtures |
| `make check` | Common quality, source roles/size, generated reproduction, dependency/publication policies, all language gates and product checks |
| `make acceptance` | Copied-binary offline behavior and owned staged installation |
| `make acceptance-font-registry` | Actual previously built Vercel handler, registry/font/license bytes and HTTP headers on an owned loopback port |
| `make release` | Common local version/note preparation; see the release guide |

Rust outputs use `<target-dir>/<target-triple>/<profile>/<file>` for native and
cross builds. Common artifact selection owns `PROFILE`, `TARGET` and output paths;
the shell compatibility entry only supplies this product's Node/counter environment.
`make installer INSTALLER_ARGS=--check` checks the generated standalone CLI download
installer. Actual release preparation and clean-HEAD packaging are separate owner
operations; the installer does not imply CLI assets have already been published.

CLI build/install honor `TARGET`, `PROFILE` and `CARGO_TARGET_DIR`. Install also honors
`PREFIX`, `BINDIR` and `DESTDIR`; defaults resolve to `/usr/local/bin`. Installation
never runs Cargo/npm. Use an owned `DESTDIR` when testing it.
Repeat `TARGET` during install when it was supplied at build time. The pinned
engine also checks whether that selector was explicit.

## Native and optional libraries

The complete Expo build requires full Xcode, Android NDK, cargo-ndk, and targets
`aarch64-apple-ios`, `aarch64-apple-ios-sim`, `aarch64-linux-android`,
`armv7-linux-androideabi` and `x86_64-linux-android`. Each native output keeps a
verified architecture-specific SBOM and binary hash before npm composition.
`TOOLCHAIN` selects the installed Rust toolchain through the common engine. Its
required targets must already be installed for that exact selection. For example,
`make build-expo TOOLCHAIN=1.95.0` uses targets prepared for that exact toolchain;
it never installs them. SDKs, product builds and checks use the fixed 1.95.0
toolchain. Earlier qualification retains its recorded source/compiler scope. Rust-only selectors do not enter the independent JS build.

Use `sh tools/artifacts.sh build --library wasm --target wasm32-unknown-unknown`
for the optional Wasm library. `windows-bridge` and `native-bridge` select the
other library declarations in `craftguild.toml`. These commands
prepare a real library dependency graph, rather than the manifest helper's graph.
The Windows bridge still needs its applicable C++/Windows toolchain for delivery.
The engine performs Cargo compilation, including cargo-ndk through a bounded argv
adapter that preserves the selected target/manifest. Product scripts retain SDK
checks, architecture lists, XCFramework assembly and package copying, and compare
copied bytes with the engine's verified receipts.

Build the host library with `sh tools/artifacts.sh build --library native-bridge`
before invoking bridge/UniFFI generators directly. Test/check operations declare
that prerequisite in the common configuration. Generators consume the recorded
host, toolchain and library outputs; their own repository-tool Cargo caches are
separate from published distribution outputs. The common Expo ownership wrapper
is refreshed from `assets/expo-generation.json` into ignored
`.craftguild/ownership/expo.json`; exact isolated generation remains mandatory.

Swift/macOS packaging is separate from Expo. Generate UniFFI bindings, then run
`node packages/expo-oriha/scripts/build-rust-macos.mjs`. Optional target selection
uses `MACOS_RUST_TARGETS`. No local device, daemon or production deployment is
started by the ordinary checks.

## Benchmark CI

The benchmark workflow retains its actual render benchmark output as an artifact.
It no longer creates unsigned commits or pushes a benchmark pages branch.
Historical comparisons remain a separate reviewed operation. Coverage and bounded
fuzz campaigns keep their own workflows; neither replaces correctness tests.

## Build-test ownership

Standards tests the shared build, install, receipt, release and measurement
contracts. This repository runs actual builds, verifies packaged artifacts and
keeps renderer behavior, native ABI, language and generated-ownership checks.
The CLI acceptance check uses the built binary offline and stages one install.
It does not repeat the common engine regression suite.
