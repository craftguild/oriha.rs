# oriha.rs Preview

Minimal Expo showcase for
`@craftguild/expo-oriha`.

The app intentionally keeps the React Native side small. The important pieces
are:

- `OrihaProvider` wrapping the preview surface
- `OrihaPreview` rendering a Markdown string
- a typed `BridgeRenderInput` built from package enums/catalog options
- `resolveRenderSettings` disabling controls and filtering options by the
  package's plan policy
- `useOrihaFonts` refreshing, installing, uninstalling, and selecting
  local fonts
- a compact progress-gauge button used by the font installer
- a Premium CTA banner with an animated hue-gradient sparkle icon
- `createUsageAnalyticsReporter` queuing render statistics from `onRendered`
- `nativeBridgeVersionInfo` showing the bundled Rust bridge version
- `onRendered` / `onRenderError` callbacks for render state

## Quick start

The example depends on the local `packages/expo-oriha` package. From the
repository root, build that package and run the iOS example:

```bash
cd packages/expo-oriha
npm ci --ignore-scripts
node scripts/generate-heroicons.mjs
node scripts/generate-bridge-types.mjs
node scripts/generate-uniffi-bindings.mjs
npx --no-install tsdown
node scripts/build-rust-ios.mjs
cd ../../examples/expo-preview
npm ci --ignore-scripts
npm run ios
```

For Android, use `build-rust-android.mjs` and `npm run android` in place of the
iOS commands. Native builds require Rust and the platform SDK; see the
[package README](../../packages/expo-oriha/README.md) for build details. No npm
publication or registry login is needed to use this local dependency.

To enable the Usage Analytics control, pass both your ingestion endpoint and
public write token at start time:

```bash
EXPO_PUBLIC_MARKUP_STATS_INGEST_URL=https://stats.example.invalid/ingest \
EXPO_PUBLIC_MARKUP_STATS_API_KEY=xxxxx npm start
```

The analytics example sends renderer statistics only; it does not send the
Markdown document text.

Because the package contains a native Expo module, use a development build or a
simulator/device build for runtime verification. Expo Go does not include this
native module.

## Optional network services

Copy [.env.example](.env.example) to `.env.local` in this app directory. Leave
values empty to render without sample-service connections.

- Set `EXPO_PUBLIC_MARKUP_STATS_INGEST_URL` and
  `EXPO_PUBLIC_MARKUP_STATS_API_KEY` for statistics delivery, then explicitly
  enable **Usage Analytics** in the app. Both values are required; delivery is
  disabled on startup. Only render statistics are queued, without document text.
- Set `EXPO_PUBLIC_FONT_REGISTRY_URL` to your registry index URL to refresh and
  install font packs. Without it, ordinary rendering and local fonts still work.

[Expo environment variables](https://docs.expo.dev/guides/environment-variables/)
use static `process.env.EXPO_PUBLIC_*` references and are embedded in the client
bundle. Use only a client-distributable ingestion key, never a server secret.
Reload the app after changing values. There are no default service URLs.

## Files To Read

- `components/showcase/OrihaShowcase.tsx` contains the sample app: settings on top, rendered
  Markdown below.
- `index.js` registers the sample app with Expo.
- `lib/sample-markdown.ts` and `lib/paywall-eyecatch-markdown.ts` contain sample
  Markdown documents.


## System requirements

Use an Expo development build containing the Oriha native module, rather than
Expo Go. Analytics is optional and sends render statistics without document text.

## Build requirements

Use the committed Node lockfiles plus Rust and the applicable Xcode/Android SDK.
The [development guide](../../docs/guides/development.md) lists native targets.

## Known limitations

Device execution is separate from typechecks and package builds. This app is a
local showcase and is not published as an npm package.

## Community

Report ordinary issues in [Oriha](https://github.com/craftguild/oriha.rs/issues).

## License

First-party example code is [MIT](../../LICENSE-MIT) OR
[Apache-2.0](../../LICENSE-APACHE); retained third-party terms still apply.
