# Third-party components

First-party code uses MIT OR Apache-2.0. The following separately licensed
components keep their original terms. Dependency names in an artifact SBOM
describe that artifact's selected runtime graph; the complete lockfile also
contains development, build and other workspace dependencies.

| Component | Use | License and retained source |
| --- | --- | --- |
| STIX Two Math v2.13b171 | Unmodified font embedded by the math renderer | [OFL-1.1](crates/latex/assets/fonts/OFL.txt) |
| WHATWG entities snapshot | Generated character-reference table | [BSD-3-Clause](crates/core/entity-data/LICENSE-WHATWG) |
| GitHub gemoji, revision `0eca75db9301421efc8710baf7a7576793ae452a` | Emoji aliases in the generated shortcode table | [MIT](crates/core/emoji-data/github-gemoji-0eca75db9301421efc8710baf7a7576793ae452a/LICENSE) |
| CLDR JSON 48.2.0 | English names in the generated shortcode table | [Unicode-3.0](crates/core/emoji-data/unicode-cldr-json-48.2.0/LICENSE) |
| Unicode Emoji 16.0 | Emoji allowlist used to generate shortcodes | [Unicode-3.0](crates/core/emoji-data/unicode-emoji-16.0/LICENSE) |
| Heroicons 2.2.0 | Six generated solid icons used by the Expo provider | MIT; included in the [Expo notices](packages/expo-oriha/THIRD_PARTY_NOTICES.md) |
| Rolldown 1.0.1 runtime helpers | Generated helpers that contribute bytes to the Expo JavaScript bundle | MIT; retained in the [Expo license inventory](packages/expo-oriha/licenses/rolldown-LICENSE) |
| Askama derive 0.14.0 | Patched build-time dependency for the existing UniFFI ABI | MIT OR Apache-2.0; original license files remain in [the vendored source](vendor/askama_derive) |
| CraftGuild standards engine/packages 0.6.2 (rules baseline 0.1.0) | Shared CLI presentation, preparation and quality tools | MIT OR Apache-2.0; pinned to `be63eefb44a850a01f4cc065c66fff2fe558ee18` in Cargo and the [fixed engine reference](standards/engine.json) |
| Configured font registry families | Separately downloaded CSS2/WOFF2 assets, outside the Expo npm payload | Original OFL-1.1 or Apache-2.0 terms and full pinned texts in the [font license inventory](packages/font-registry/licenses/manifest.json) |
| piccolore 0.1.3 | Registry runtime terminal formatting dependency | ISC; the published fork omits a separate license file, so its metadata and the [original upstream notice](packages/font-registry/licenses/runtime-notices.json) are retained together |

The vendored Askama patch replaces its build-time TOML parser without changing
the UniFFI binding interface. It is a build dependency and is not an extra
runtime component of the native renderer. `borrow-or-share` under MIT-0 is also
used by the schema-validation build tool, through `fluent-uri`.

The complete font and data notices are retained in the
[Expo package notices](packages/expo-oriha/THIRD_PARTY_NOTICES.md). Native
dependency license text is generated into the package's `licenses` directory
from its selected target graphs. Fonts installed later through a configured
registry are separate downloaded assets; their registry distribution must retain
its own source, content hashes and license records.
