# Emoji shortcode source data

This directory vendors pinned upstream data used to generate the Rust emoji
shortcode lookup table. The renderer does not read these files at runtime.

Sources:

- GitHub gemoji `db/emoji.json`
  - Commit: `0eca75db9301421efc8710baf7a7576793ae452a`
  - Source: https://github.com/github/gemoji
  - License: MIT
- Unicode CLDR JSON English annotations
  - Release: `48.2.0`
  - Source: https://github.com/unicode-org/cldr-json
  - License: Unicode-3.0
- Unicode emoji test data
  - Version: `16.0`
  - Source: https://unicode.org/Public/emoji/16.0/emoji-test.txt
  - License: Unicode-3.0

License texts are vendored next to each source and included in
`packages/expo-oriha/licenses/third-party-licenses.json`
by `packages/expo-oriha/scripts/generate-third-party-licenses.mjs`.

Generation:

```sh
node crates/core/scripts/generate-emoji-shortcodes.mjs --fetch
```

Run without `--fetch` to regenerate from vendored source files only.

Merge policy:

- GitHub aliases are loaded first and win conflicts.
- CLDR `tts` names are slugified and only included when the character appears
  in Unicode `emoji-test.txt`.
- Emojibase shortcodes such as `thumbs_up_tone1` are intentionally not included;
  CLDR skin tone names use forms such as `thumbs_up_light_skin_tone`.
