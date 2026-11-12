# Verification methods

Keep the public behavioral fixtures, snapshots, runnable examples and generators
with the source. Original experiment logs, consumer overlays, source archives and
implementation prompts are private historical material; ordinary builds and tests
do not require them. Their removal does not change an accepted ADR's status.

## Required correctness checks

`make check` runs the locked release-profile Rust workspace tests, Clippy,
formatting, npm tests/typechecks/lint, explicit source/size assessment, canonical
generation and dependency/publication policies. Product gates also cover block
handling exhaustiveness, reproducible third-party notices, JavaScript bundle
provenance, native binding ABI checksums and actual browser image decoding.

The CommonMark profile test compares all 652 official examples with the expected
HTML after removing renderer-specific wrappers/attributes. Product-profile
extensions have separate tests. JSON snapshots, prepared layout versions, block
budgets, escaping, annotations and typed native outputs remain compatibility
contracts. Do not accept new expected output solely to make a refactor pass.

The browser image test renders real API inputs, checks `src`/`srcset`, waits for
actual image decoding and retains a deliberately broken data-URL control. Its
browser process/profile and output directory are owned temporary resources.

`tools/check-bindings.py` generates isolated Swift/Kotlin bindings and compares
exported checksums with the compiled host library. Moving UniFFI public types or
exported functions into another Rust namespace changes the ABI; preserve their
original paths. Native device execution is separate from this host check.

## Packaging and standalone use

`make build-expo` verifies JavaScript module evidence, five actual native payloads,
target SBOM seals and the exact npm file inventory. It creates an owned temporary
tarball and deletes that tarball after inspection. It never invokes publication.
`make acceptance` exercises a copied CLI without checkout access, useful PATH
entries, configuration or network; it also checks safe staged installation.
Its assertions are an opt-in Rust integration test of the engine-built artifact.
The script invokes Cargo with the receipt's toolchain/target/profile and a separate
test build directory. Run `make build` first; the default Cargo suite's ignored
artifact test is not acceptance evidence.

A standalone source export must retain the complete reviewed source inventory,
lockfiles, generators, fixtures, third-party licenses and pinned standards assets.
Exclude ignored caches/private research. Rebuild in that owned export without
sibling repositories before claiming standalone delivery.

## Optional measurements

`crates/core/examples/` contains current adoption/source/consumer examples.
`tools/benchmarks/` preserves historical baseline programs needed by
`scripts/measure-adoption.py`. Their measurements are optional investigations,
not correctness gates or asserted current performance. Use isolated output and
explicit source identities; never require a private consumer checkout for the
ordinary renderer test suite.
