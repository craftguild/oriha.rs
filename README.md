<h1 align="center">oriha.rs</h1>

<p align="center">
  <a href="https://github.com/Essk/ai-contribution-level">
    <img src="https://raw.githubusercontent.com/Essk/ai-contribution-level/main/badges/level-3.svg" alt="AI Contribution">
  </a>
</p>

oriha.rs turns Markdown into HTML, structured blocks and React Native views. Its
Rust engine preserves source locations for editors and annotations; the
`@craftguild/expo-oriha` package brings that engine to Expo applications.
A local `oriha` command provides the same parsing and rendering workflows.

> [!WARNING]
> This project has not reached version 1.0 and may be unstable.
> Its behavior and specifications may change, and upgrades may require manual migration.
> Our team uses it in internal production systems to refine its design and implementation, but it may not meet your requirements.
> Please keep these limitations in mind when using it.

## Quick start

Release binaries are available for Linux x86_64 GNU and macOS arm64 from
[GitHub Releases](https://github.com/craftguild/oriha.rs/releases/tag/v0.1.0).
The installer needs POSIX sh, curl, and `sha256sum` or `shasum`. It verifies the
downloaded checksum and installs to `~/.local/bin` by default.

Install the CLI, then render a short document:

```sh
curl -fsSL https://raw.githubusercontent.com/craftguild/oriha.rs/v0.1.0/scripts/install.sh | sh -s -- --version 0.1.0
BIN="$HOME/.local/bin/oriha"
printf '# Hello\n\nA local Markdown document.\n' > document.md
"$BIN" render document.md > document.html
"$BIN" inspect document.md
```

Open `document.html` to see the rendered document. `inspect` reports the parsed
structure. To embed the native renderer in an app, follow the
[Expo package guide](packages/expo-oriha/README.md).

Pass `--bindir /absolute/bin` after `--version 0.1.0` to choose another directory.
Use `--uninstall` instead of `--version 0.1.0` to remove the command; pass the same
`--bindir` for a custom destination. Add that directory to PATH for bare commands.
Each release includes target-specific `SHA256SUMS` files and a CLI SBOM.

## System requirements

The compiled CLI needs no Rust, Node.js or registry service at runtime. Use a
binary built for the intended OS and architecture. Expo applications need a
custom native build with the package's declared React/React Native/Expo peers;
Expo Go does not include this native module. Fonts are local assets; the font
registry is optional.

## Build requirements

Source builds use Rust/Cargo **1.95.0**, a native linker, Git, Make, Python 3
and curl. First-party Rust crates declare minimum Rust **1.95**. The repository
fixes standards **v0.6.1** in `standards/engine.json`; explicit `make setup`
restores its verified cache on macOS arm64 or Linux x86_64 GNU. A global
standards command is unnecessary after checkout.

Full checks also need rustfmt, Clippy, tokei **15.0.0**, cargo-deny, Node.js
**24+**, Chrome/Chromium and the three locked development dependency trees:

```sh
npm ci --prefix packages/expo-oriha --ignore-scripts
npm ci --prefix packages/font-registry --ignore-scripts
npm ci --prefix examples/expo-preview --ignore-scripts
make check
make acceptance PROFILE=release
```

`make build-expo` assembles and inspects a local package. It additionally needs
full Xcode on macOS, the configured iOS/Android targets, Android NDK and cargo-ndk.
Builds do not install those tools or select a different compiler implicitly.
See [development](docs/guides/development.md) for preparation and focused checks.

Build the CLI from source after preparing the build tools:

```sh
make setup
cargo +1.95.0 fetch --locked
make
```

## Usage

```sh
oriha render input.md --math --diagrams
oriha render-json input.md
oriha --version
oriha sbom > oriha.cdx.json
```

Rust callers use `render`, `parse` and `render_parsed` from `oriha-core`.
The [parsing](docs/guides/0001-source-aware-parsing.md),
[block output](docs/guides/0002-block-output.md) and
[responsive image](docs/guides/0003-image-presentation.md) guides describe the APIs.
Existing consumers should read the [rename guide](docs/guides/0004-oriha-rename.md).

Builds write `target/<target-triple>/<debug|release>/oriha`.
`PROFILE`, `TARGET` and `CARGO_TARGET_DIR` select the artifact.
`make install BINDIR=/absolute/bin` copies a verified existing binary;
`PREFIX` and `DESTDIR` support staged installation. `make release` prepares
versions and notes locally; publication is a separate operation.
The [documentation index](docs/README.md) links guides, design and accepted ADRs.

## Known limitations

- Math, Mermaid and PlantUML support the subsets defined by the product contracts;
  unsupported input has documented diagnostics or fallbacks.
- Native packages require actual platform SDKs. Host checks do not qualify
  device behavior or every cross target.
- Optional Wasm, Windows and macOS bridge artifacts have separate scopes and
  are not all included in the Expo package.
- A generated package cannot know which peer versions an application will select.
- Rust crates are consumed from source rather than published on crates.io.

## Community

Use [GitHub issues](https://github.com/craftguild/oriha.rs/issues) for ordinary
bugs and proposals. Read [AGENTS.md](AGENTS.md) and the documentation index before
changing parsing, ABI or generated contracts. [Release guidance](docs/guides/releases.md)
separates local preparation from signing, tagging and publishing.

## Security

A private vulnerability-reporting route has not yet been documented for oriha.rs.
Do not include credentials or sensitive exploit details in public issues.

## License

First-party code is licensed under either the [MIT License](LICENSE-MIT) or the
[Apache License, Version 2.0](LICENSE-APACHE), at your option.
[Third-party notices](THIRD_PARTY_NOTICES.md) describe separate font, data, icon
and vendored dependency terms.
