# Markdown Render Samples

`comprehensive.md` matches the Expo preview stress sample and checks the parser,
theme engine, diagrams, callout extension, and dense document rendering through
the `oriha render` subcommand.

`paywall-eyecatch.md` is a longer real-world article sample about cats, with a
photo, tables, sequence diagram, callouts, task list, and playful math.

Default decorated HTML:

```sh
cargo run -p oriha -- render examples/markdown/comprehensive.md
```

Paywall eyecatch HTML:

```sh
cargo run -p oriha -- render examples/markdown/paywall-eyecatch.md
```

Plain semantic HTML without generated CSS:

```sh
cargo run -p oriha -- render --plain examples/markdown/comprehensive.md
```

Dark/simple themed HTML with print CSS and watermark:

```sh
cargo run -p oriha -- render \
  --color-scheme dark \
  --style simple \
  --syntax-theme monokai \
  --table-border-style all \
  --print \
  --page-size "A4 landscape" \
  --margin 10mm \
  --watermark-text "Draft preview" \
  --watermark-opacity 0.35 \
  examples/markdown/comprehensive.md
```
