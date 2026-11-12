# Local Askama configuration-parser backport

This is `askama_derive` 0.14.0 from crates.io, MIT OR Apache-2.0. `UPSTREAM.json`
records hashes of the original packaged files. The original licenses, sources,
test templates and README are retained. The upstream package lockfile and
packaging metadata are omitted; the local lockfile pins standalone test tools.

The root `[patch.crates-io]` selects this copy for UniFFI 0.31.1's build-time
template compiler. The only functional changes are:

- `Cargo.toml`: replace optional `basic-toml` with maintained `toml` >=1.1.8,
  enabling `parse`, `serde`, and `std`, and adjust the `config` feature.
- `src/config.rs`: call `toml::from_str` at the same configuration boundary.

The newer parser supports the borrowed strings required by Askama's configuration
types; `toml` 0.9 does not satisfy that requirement. Template parsing and code
generation are unchanged. This removes RUSTSEC-2026-0374's dependency rather than
adding an advisory ignore. This crate is a host build tool, not the Markdown
runtime parser. No code from `basic-toml` is retained.

UniFFI 0.32.2 was examined but changed existing API checksums and still depended
on `basic-toml` through Askama 0.16.1. Keeping UniFFI 0.31.1 preserves the existing
foreign bindings. Remove this patch when a compatible upstream dependency graph
removes the unmaintained parser; check generated bindings and API checksums then.

Reproduce the standalone upstream tests with:

```sh
cargo test --manifest-path vendor/askama_derive/Cargo.toml --features full --locked --target-dir target/askama-backport
```

The workspace tests, actual UniFFI generation comparison, checksum probe, and
dependency audit are documented in the ADR-0001 adoption-remediation report.
