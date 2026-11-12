# Font Registry

This private package builds the optional font installer registry for Oriha. It
serves a human-readable index, registry JSON and the exact downloaded font files
from a separate Astro/Vercel deployment. It is not part of the Expo npm payload.

## Quick start

With the build requirements installed, generate local fixture output:

```sh
npm ci --ignore-scripts
npm run test:fixtures
```

The result is `.tmp/fixture-dist/index.json` with local fake font bytes. Fixtures
are test assets, and are never accepted as production deployment evidence.

## System requirements

The generated Vercel deployment selects Node.js 24 in its function configuration.
Runtime serves already generated assets and needs no Google Fonts connection.
Local app rendering uses installed fonts; registry access is only for explicit
user-selected downloads. No document content is sent to a font provider.

## Build requirements

Use Node 24+, the committed npm lockfile and network access for initial production
font downloads. The complete repository also uses the pinned Rust standards tool
for SBOM validation. From the repository root, `make build-font-registry` builds
the actual Vercel output and its independently sealed SBOM.
Use `make acceptance-font-registry` to exercise that generated handler locally.

`registry.config.json` owns the configured families/styles. The initial build
fetches Google Fonts CSS2 and the referenced WOFF2 files without `text=` or
per-document subsetting. Rebuild downloads explicitly with
`npm run build:registry:server` when changing that configuration or refreshing
upstream assets. Matching retained production assets can be reused after their
configuration, inventory and content hashes are verified.

## Usage

- `/` is the human-readable index.
- `/index.json` serves registry metadata.
- `/fonts/...` serves downloaded assets and retained family license files.
- `generated/provenance.json` records source URLs, CSS/font hashes and upstream
  license associations; `licenses/manifest.json` pins the original license texts.

`npm run font:add -- --url https://fonts.google.com/specimen/Source+Sans+3 --kind body`
adds a configured family. Retain and pin its upstream license before a production
build. Valid kinds are `heading`, `body` and `code`.

Application access logs contain request method/path/status/duration and selected
request/cache metadata, without application-level IP logging. The registry must
not inspect document content. Astro image transformation is explicitly disabled:
this site serves fonts and does not ship host-specific libvips binaries.

## Known limitations

Google Fonts CSS/CDN responses can change. The deployment records the bytes
actually fetched; a CSS hash is not a claim about an upstream font release number.
Local builds do not prove deployed Vercel behavior. Publication and deployment
remain separate owner operations. See the root distribution and verification
contracts for SBOM boundaries.

## Community

Report ordinary issues in [Oriha](https://github.com/craftguild/oriha.rs/issues).

## License

First-party code is [MIT](../../LICENSE-MIT) OR [Apache-2.0](../../LICENSE-APACHE).
Each downloaded font retains its own license from [licenses/manifest.json](licenses/manifest.json).
The actual deployment also retains notices for evidenced runtime packages.
