# Rename to Oriha

Markdown Document Engine is now Oriha. This is a breaking rename with no
compatibility aliases. The npm package version is unchanged by this source
change; publishing a new package is a separate step.

| Previous name | Oriha name |
| --- | --- |
| `mde-*` crates and `mde_*` Rust imports | `oriha-*` and `oriha_*` |
| CLI crate `mde-cli` | `oriha` |
| `mde` CLI | `oriha` |
| Rust `MarkdownEngine` | `OrihaEngine` |
| `@progressiveworks-co/expo-markdown-document-engine` | `@craftguild/expo-oriha` |
| `MarkdownEngineProvider`, `useMarkdownEngineFonts` | `OrihaProvider`, `useOrihaFonts` |
| `MarkdownPreview`, `MarkdownPreviewProps` | `OrihaPreview`, `OrihaPreviewProps` |
| `MarkdownPdf*`, `MarkdownRenderSettings*` | `OrihaPdf*`, `OrihaRenderSettings*` |
| `ExpoMarkdownDocumentEngine`, `MdeRustBridge` | `ExpoOriha`, `OrihaRustBridge` |
| `MdeNativeBridge.xcframework` | `OrihaNativeBridge.xcframework` |
| `mde-*`, `data-mde-*`, `--mde-*` output identifiers | `oriha-*`, `data-oriha-*`, `--oriha-*` |
| `MDE_*` environment variables | `ORIHA_*` |
| `X-MDE-Stats-Key` HTTP header | `X-Oriha-Stats-Key` |
| `strip_mde_data_attributes` / `stripMdeDataAttributes` | `strip_oriha_data_attributes` / `stripOrihaDataAttributes` |

Update imports, native linking, custom CSS, DOM queries, export options, and
analytics clients and servers together. Rebuild the Expo development client;
the old native module cannot load the renamed library. Markdown format names,
including the `markdown` input field and `prepareMarkdownLayout`, still describe
the supported document format and keep their names.

Render snapshots now use schema version 3. Snapshots from schema version 2 are
incompatible and trigger a full replacement rather than reusing old DOM IDs.
Prepared HTML exports now use version 2 for the renamed export option.

Font files and the analytics outbox start in the new
`co.progressiveworks.expo-oriha` storage directory. The old directory is neither
read nor deleted. Reinstall fonts as needed; pending analytics records in the
old directory are not delivered by Oriha.

The repository URL is `https://github.com/craftguild/oriha.rs`. The Expo package
uses the `@craftguild` npm scope. Company attribution remains Progressive Works, Inc.
This source rename does not publish the package to npm.
