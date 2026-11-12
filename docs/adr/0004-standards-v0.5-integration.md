# ADR 0004: Integrate the pinned standards v0.5.0 engine

Status: Accepted; v0.5.0 integrated in the worktree, qualification tracked in Beads

Date: 2026-11-07

## Context and decision

The former v0.2.0 bundle integration and its results remain historical evidence.
Replace its delivery mechanism with published standards v0.5.0 at
`69ef0ba960e4d61bff3f363c07f8d49c4d53d77e`, retaining normative rules 0.1.0.
This decision supersedes the bundle-delivery section of the
[distribution design](../design/distribution.md). Current implementation and
qualification evidence is recorded in Beads, separately from this decision.

After preserving and explicitly retiring the old pin, copied common rules,
archives and loaders, run the matching `craftguild-standards init` from this root.
The initializer creates `standards/engine.json`, the thin launcher/Make entry,
local Skill and common AGENTS block, and caches `craftguild-standards-engine`
under `.craftguild/engine/`. It preserves product configuration and dependencies.
Adapt `craftguild.toml`, Make/CI and common-crate full Git revisions explicitly.
Do not replace product recipes or duplicate common execution in hooks.

Repository operations use `include standards/make/common.mk` and the fixed
launcher. Explicit `make setup` restores the pinned cache after cloning, without
a global initializer or engine. `make prime` reads its offline embedded rules.
Ordinary operations never acquire or update the engine. No copied rules tree,
archive loader, sibling checkout or global-engine fallback remains.

Enable `rust_output_layout = "standard"` in configuration schema 2. Rust outputs
use `<target-dir>/<target-triple>/<profile>/<file>` for native and cross builds;
remove output overrides and update SDK, binding and packaging callers together.
Keep frontend output boundaries and specialized platform admission separate.

## Uniform Rust build toolchain

The user's latest 2026-11-07 decision selects Rust **1.95.0** across the five
adoption repositories, including SDK builds, ordinary tests/checks/acceptance,
CI and canonical generation. It supersedes the briefly attempted 1.86 decision
and removes the requirement to preserve Rust1.85 compatibility for other users.
Declare first-party minimum Rust1.95 and select the same fixed toolchain, without
falling back to latest/stable or keeping an older compatibility lane.

The 1.86-only dependency downgrades and source workarounds were retired; their
attempt evidence remains private, not a successful qualification. Preserve the
published standards pin and captured historical SDK producer/compiler facts.
Regenerate outputs only through their real generator when inputs/contracts change.
Explicitly prepare toolchains/targets; ordinary builds never install them.
Requalify I1–I8 on current source rather than reuse a prior compiler's result.
Auxiliary nightly sanitizer instrumentation remains a distinct campaign and does
not substitute for the fixed product build. No business lifecycle/schema changes
are introduced by this technical toolchain policy.

## Product contracts and completion

The 2026-11-08 test-language amendment moves the product assertions added by this
adoption into the CLI crate's Rust integration test. The acceptance script only
verifies the existing common receipt and invokes Cargo with the selected fixed
toolchain, target, profile and immutable preparation. Cargo test output uses a
separate directory so compiling tests cannot replace the verified shipped CLI.
The test is opt-in because it needs that engine-built artifact; `make acceptance`
explicitly runs it, and an ignored default-suite result never qualifies I4.
Product scenarios, offline metadata/rendering and installed-byte assertions stay
in Rust. Other components/frameworks and unchanged legacy tests retain their
existing ownership; no new runtime command or test framework is introduced.

Apply published standards
[ADR 0006](https://github.com/craftguild/standards.rs/blob/69ef0ba960e4d61bff3f363c07f8d49c4d53d77e/docs/adr/0006-initializer-cli-and-controlled-engine-upgrades.md)
with the local completion criteria below:

| Criterion | Completion evidence |
| --- | --- |
| I1 — Fixed identity | v0.5.0 reference, managed entry/Skill/common block and common-crate full revision agree. |
| I2 — Connected operations | Make/CI and product configuration use the fixed engine; old bundle authorities and duplicate common loaders are retired. |
| I3 — Standalone workflow | Isolated intended public source supports explicit setup, offline prime and actual build without siblings, private archives or a global engine. |
| I4 — Actual artifacts | Required profiles/targets use standard Rust paths, exact SBOMs and relevant offline interfaces; copy-only install uses a temporary destination. |
| I5 — Product quality | Final `make check`, actual builds, applicable acceptance and canonical generated ownership/reproduction succeed. |
| I6 — Maintained delivery | Documentation, licenses, dependencies, implementation Code measurement and declared installer/release settings match. |
| I7 — Current evidence | Beads records the final source/index basis, pin, commands, profiles, hosts and deferred qualification; required missing/failing local gates block completion. |
| I8 — Execution boundary | Local completion leaves real release, clean-HEAD packaging, staging, commits and publication for separate authorization. |

Init or prime alone does not satisfy these criteria.

Preserve Markdown/source mapping, the 652-case CommonMark profile, typed native
contracts, JSON formats, resource bounds and escaping. Retain actual CLI, native
library, Expo payload, font registry and canonical generation checks. Existing
Unix-to-Windows cross-library output remains a product contract, without adding
a Windows build host. Shared engine/font/measurement failure matrices stay in
standards; do not restore the removed FIGlet regression or its dependencies.

On final source, run required `make check`, actual builds and applicable acceptance;
verify standard output paths, exact embedded SBOMs, temporary copy-only install
and source-only standalone operation. Keep unavailable SDK/host qualification
explicit. Record evidence in Beads, preserving prior v0.2.0 results as history.

## Operational and provenance boundary

Beads tracks current implementation and qualification. Maintaining this ADR
does not itself certify implementation. Local integration does not authorize
stage/commit/tag/push, actual `make release`, version/release-note changes, real
installation, service operations or publication. Installer declarations and
generation checks can be prepared; clean-HEAD packaging and release stay separate.
Every later authorized commit is OpenPGP-signed and verified by full revision;
a real signing or verification failure pauses the whole adoption goal.

Build/init/release receipts and Git provide technical provenance. No persistent
business lifecycle, transition-history table or Header-Detail model is introduced.

## Published v0.6.0 adoption amendment (2026-11-11)

The user authorized the standards authentication release and explicit adoption
for this initial CLI delivery. Upgrade through the published v0.6.0 initializer
to full revision `72cf076962dc8401b2e4d1d84933c8a818ff390c`, then align the
common Rust Git declarations and lockfile with that same revision. This replaces
the v0.5.0 pin in the current identity/completion criteria above; earlier decisions,
results and actual generator-producer records keep their original provenance.
Normative baseline 0.1.0, reference schema 3, configuration schema 2 and both
compatibility protocols remain unchanged. Rust 1.95.0 and all product gates stay
required. Init preserves product recipes, configuration and local task state.

The shared release removes its implicit aggregate command deadline while retaining
explicit watchdogs, cancellation and owned-child cleanup. This does not establish
a diagnosis for an earlier hosted-runner shutdown. Requalify the current product
source, actual artifacts and native CI rather than reuse older-source passes.
Ship the CLI for Linux x86_64 GNU and macOS arm64 through CI-built Release assets,
with target checksums and SBOMs, a generated installer and README instructions.
Verify real download/install/uninstall in owned temporary destinations.

The public CLI installer retains the default curl transport. CLI delivery does
not publish npm/crates registry packages or qualify additional SDK/OS payloads.

The active release request separately authorizes signed commits/tags, pushes and
GitHub publication. Preserve the original local-integration boundary as history;
this amendment does not claim any unexecuted qualification or introduce a business
lifecycle, transition-history table or Header-Detail model.

## 2026-11-11 renamed standards release

The owner authorized adoption of published `craftguild/standards.rs` v0.6.1 at
`6852b210a9a614920bc5be2187f2705f46db978d`. The actual published initializer re-created the
managed entry after its previous owned bytes were verified and archived.
The engine and common Cargo crates use that same full revision. Product
configuration and instructions remain intact. No old repository-name fallback
is provided. The normative baseline stays 0.1.0; product Rust stays 1.95.

## 2026-11-11 Linux CI browser namespace preparation

The ephemeral Linux validation runner prepares an owned copy of preinstalled
Google Chrome under `RUNNER_TEMP` and verifies its launcher and executable bytes.
A temporary AppArmor profile permits user namespaces for that copy's exact
executable path, following the [Chromium namespace guidance](https://chromium.googlesource.com/chromium/src/+/main/docs/security/apparmor-userns-restrictions.md).
The global namespace restriction retains its original value. The browser's
sandbox flags, 48 real image assertions, negative control and deadlines remain
unchanged. Job cleanup removes the owned profile and browser copy.
