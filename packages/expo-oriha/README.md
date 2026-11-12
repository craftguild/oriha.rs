# oriha.rs for Expo

`@craftguild/expo-oriha` embeds the Oriha Rust Markdown engine in Expo
applications. It provides a measured preview WebView, typed renderer settings,
PDF export and optional local font installation.

## Quick start

Build this source package with `make build-expo` from the repository root, then
use it from an Expo development build. The local
[showcase](https://github.com/craftguild/oriha.rs/tree/main/examples/expo-preview)
demonstrates the complete integration without an npm publication.

```tsx
import { OrihaPreview, OrihaProvider } from "@craftguild/expo-oriha";

export function Document() {
  return (
    <OrihaProvider>
      <OrihaPreview markdown={"# Hello\n\nRendered by Oriha."} />
    </OrihaProvider>
  );
}
```

## System requirements

Use an iOS/Android app build containing the native Expo module; Expo Go cannot
load it. Install the peer dependencies declared in `package.json`, including
React, React Native, Expo, WebView, SVG and the required Expo file/print/share
modules. Peer ranges do not constitute a promise that every upstream version is
compatible. The showcase locks the versions used for development.

## Build requirements

Source packaging uses Node 24+, Rust/Cargo, Python 3, full Xcode, Android NDK and
cargo-ndk. Install the committed lockfile with `npm ci --ignore-scripts` first.
The root [development guide](https://github.com/craftguild/oriha.rs/blob/main/docs/guides/development.md)
lists platform targets and the complete repository checks.

`npm run build` generates bindings, builds JavaScript/declarations and all five
native slices, composes SBOMs and inspects a local npm tarball. It does not
increment versions, commit, tag or publish. Optional macOS Swift output uses
`node scripts/build-rust-macos.mjs` separately.

## Public API

`prepareMarkdownLayout` and `renderPreparedMarkdown` expose the two-step typed
pipeline. `resolveRenderSettings` provides localized controls and plan limits.
`nativeBridgeVersionInfo` reports the embedded bridge version and build identity.
`useOrihaFonts` installs/removes optional local fonts, and `useOrihaOverlay`
provides caller overlays and toasts. See the
[API guide](https://github.com/craftguild/oriha.rs/blob/main/docs/guides/expo.md)
for measurement, rendering options, font configuration and PDF examples.

Downloaded fonts are checked against the registry's SHA-256 values. Preview and
rendering use installed local files, and never send document content to a font
provider. The optional registry has its own deployment and asset provenance.

## Known limitations

The Rust bridge enforces plan limits and reports clamped settings as diagnostics.
Free-plan PDF output has a default watermark. Math and diagrams implement tested
subsets. Native device execution must be tested separately from package builds.
The included `sbom/bom.cdx.json` describes bundled bytes and selected native
runtime graphs; external app peers and system SDKs are outside that graph.

## Community

Report ordinary bugs in [Oriha issues](https://github.com/craftguild/oriha.rs/issues).
The repository documentation explains source contributions and release preparation.

## License

First-party code is [MIT](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE).
See [third-party notices](THIRD_PARTY_NOTICES.md) and `licenses/` for retained
font, data, icon, helper and native dependency license text.
