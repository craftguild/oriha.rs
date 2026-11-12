# CommonMark character-reference vocabulary

`whatwg-entities.json` is a verbatim snapshot downloaded on 2026-10-29 from
<https://html.spec.whatwg.org/entities.json>, the authoritative vocabulary named
by [CommonMark 0.31.2 section 2.5](https://spec.commonmark.org/0.31.2/#entity-and-numeric-character-references).
Attribution: Copyright © WHATWG (Apple, Google, Mozilla, Microsoft), HTML Living
Standard. The source-code portions use BSD-3-Clause; the document uses CC-BY-4.0.
The upstream notices and licenses are retained in [LICENSE-WHATWG](LICENSE-WHATWG).

SHA-256: `d741d877ac77c4194c4ad526b5b4a19aef8dfe411ab840a466891cdbb9f362e6`.

The snapshot contains 2,231 spellings. CommonMark requires the terminating
semicolon, so the generated table contains its 2,125 semicolon-terminated names,
including 93 two-code-point values. Names remain case-sensitive. The maximum name
length is 31 ASCII bytes; the decoder uses this bound before looking for a
semicolon. Decimal and hexadecimal references use the separate CommonMark 1–7
and 1–6 digit grammars, respectively. HTML's missing-semicolon recovery is not used.

Regenerate or verify from this pinned input, with no network access needed:

```sh
python3 crates/core/scripts/generate-character-references.py
python3 crates/core/scripts/generate-character-references.py --check
```

`src/character_reference/names.rs` is generated. Change the pinned input and
regenerate when updating the vocabulary; do not add individual names by hand.
