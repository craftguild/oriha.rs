# Expo Oriha

Expo module wrapper for the Oriha Rust document engine.

The public JS API accepts a typed bridge input. Callers choose a plan and
catalog options; the Rust `oriha-native-bridge` crate resolves those choices into
the native core input and appends bridge policy warnings to
`RenderOutput.diagnostics` when a locked option is clamped.

```ts
import {
  AppearanceStyle,
  ColorScheme,
  OrihaPdfOrientation,
  OrihaPdfPaperSize,
  Plan,
  SyntaxTheme,
  TableBorderStyle,
  prepareMarkdownLayout,
  renderPreparedMarkdown,
} from "@craftguild/expo-oriha";

const layout = await prepareMarkdownLayout({
  markdown,
  plan: Plan.Premium,
  renderingOptions: {
    colorScheme: ColorScheme.Dark,
    syntaxTheme: SyntaxTheme.TokyoNight,
  },
});
const output = await renderPreparedMarkdown(layout, []);
```

Use `nativeBridgeVersionInfo()` to inspect the bundled Rust bridge version and
build git hash.

Use `resolveRenderSettings()` to build UI controls from the bridge-resolved
`availableOptions` and `controls` payload. The Rust bridge localizes option
labels for the requested locale and reports locked controls when the selected
plan clamps a setting. Free enables GFM, raw HTML, syntax highlighting, dark
mode, tables, and task lists, but clamps callouts, diagrams, math, custom syntax
themes, and fonts. Premium enables callouts, math, Mermaid, PlantUML, syntax
theme customization, and font customization.

For React Native callers, `useOrihaRenderSettings()` wraps the default
settings draft, bridge resolution, localized options, control availability, and
installed-font selection mapping without imposing any UI. Build your own
controls from `settings.availableOptions` and `settings.controls`, then pass
`settings.previewInput` to `OrihaPreview`.

```tsx
const settings = useOrihaRenderSettings({ locale });

<OrihaPreview
  markdown={markdown}
  input={settings.previewInput}
/>;
```

Rendering appearance lives under `renderingOptions`. The same options are used
by preview rendering and by PDF export from a preview ref. PDF export-specific
settings such as paper size, orientation, margin, and watermark stay on the PDF
export options.

```tsx
<OrihaPreview
  markdown={markdown}
  input={{
    plan: Plan.Premium,
    renderingOptions: {
      colorScheme: ColorScheme.Dark,
      appearanceStyle: AppearanceStyle.Rich,
      syntaxTheme: SyntaxTheme.TokyoNight,
      tableBorderStyle: TableBorderStyle.Inner,
      roundedCorners: true,
      shadows: true,
      fullTableData: false,
      blockBorders: false,
    },
  }}
/>
```

`fullTableData` is intended for export-friendly table rendering. It keeps table
width constrained to the document width and wraps cell text instead of relying
on horizontal scrolling.

```tsx
await previewRef.current?.exportPdf({
  paperSize: OrihaPdfPaperSize.A4,
  orientation: OrihaPdfOrientation.Portrait,
  margin: "12mm",
  renderingOptions: {
    fullTableData: true,
    shadows: true,
  },
});
```

When the bridge applies a default free-plan PDF watermark, the watermark is
centered at the bottom of the screen and at the bottom of each printed/PDF page.
The default content is a QR code pointing to
`https://markup.progressiveworks.co` with the text `Generated with markup`.
Set `MARKUP_WATERMARK_URL` in the native runtime environment to override the QR
payload URL. Passing an explicit PDF `watermark` option overrides the default;
passing `watermark: false` removes it when the plan allows removal.

`OrihaProvider` owns one hidden offscreen measurement WebView shared by
`OrihaPreview` instances, and also installs a root overlay host for package
toasts and caller-provided overlays. `OrihaPreview` prepares the Markdown
layout, measures text internally when measurements are requested, finalizes the
rendered HTML, and displays it in a visible `react-native-webview`.

```tsx
<OrihaProvider>
  <OrihaPreview
    markdown={markdown}
    loadingFallback={<Loading />}
  />
</OrihaProvider>
```

`loadingFallback` is optional; the default is `ActivityIndicator`.

Local font installers are storage APIs only; the package does not provide font
picker UI. Configure an optional registry endpoint on the provider, then build
your own controls from `useOrihaFonts()`.

```tsx
<OrihaProvider
  fontRegistry={{
    baseUrl: "https://example.com/markdown-fonts/",
    storageNamespace: "my-app",
  }}
>
  <App />
</OrihaProvider>
```

```ts
const {
  installedFonts,
  installableFonts,
  refreshFontRegistry,
  installFont,
  uninstallFont,
} = useOrihaFonts();
```

Installed fonts are stored under the package-owned Expo document directory.
Downloaded files are verified with SHA-256 before they are added to the local
installed index. The index stores file paths relative to that package-owned
storage root, and runtime APIs rebuild absolute file URIs from the current Expo
document directory. If an installed-index entry exists but one of its font files
is missing, the entry is pruned and treated as not installed. `installFont(fontId,
{ onProgress })` reports per-file download/write progress so callers can decide
how much detail to display. To render with an installed font, pass
`HeadingFont.Custom`, `BodyFont.Custom`, or `CodeFont.Custom` with the installed
font's `fontFamilySpec`.

Code-block copy actions show a top toast through the provider instead of
mutating the copy button label. Apps can also use `useOrihaOverlay()`
to present custom overlay content from the provider host.

```tsx
import {
  OrihaToastPosition,
  OrihaToastVariant,
  useOrihaOverlay,
} from "@craftguild/expo-oriha";

const { showToast } = useOrihaOverlay();

showToast("Saved", {
  position: OrihaToastPosition.Top,
  variant: OrihaToastVariant.Success,
});
```

Toast variants are `Neutral` (the default when `variant` is omitted), `Info`,
`Note`, `Success`, `Warn`, and `Err`. `Warning` and `Error` are aliases for
`Warn` and `Err`.


## Building and release preparation

The root `make build-expo` builds actual JavaScript/declarations and native
payloads, composes their SBOMs and inspects a local tarball. See
[development](development.md) for native target requirements. Optional macOS
Swift builds are separate from the Expo npm payload.

[Release preparation](releases.md) owns versions and notes through the pinned
standards tool. It is separate from artifact building and npm publication.

Heroicons and typed bridge catalogs follow their canonical generators and
ownership manifests. UniFFI Swift/Kotlin outputs retain the original Rust type
and function namespaces. Never edit generated output to conceal an ABI change.
