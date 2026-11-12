import assert from "node:assert/strict";
import test from "node:test";

import {
  buildInstalledFontFaceCssRules,
  buildInstalledFontFaceRule,
  cssFontFamily,
  fontMimeType,
  selectedFontFamilies,
} from "../src/fonts/fontRegistryCss.ts";

test("cssFontFamily quotes and escapes unquoted font families", () => {
  assert.equal(cssFontFamily(" Inter "), "\"Inter\"");
  assert.equal(cssFontFamily("\"Already Quoted\""), "\"Already Quoted\"");
  assert.equal(cssFontFamily("A \"Quoted\" \\ Family"), "\"A \\\"Quoted\\\" \\\\ Family\"");
});

test("fontMimeType prefers explicit format and falls back to file extension", () => {
  assert.equal(fontMimeType({ fileName: "font.bin", uri: "file://font", format: "woff2" }), "font/woff2");
  assert.equal(fontMimeType({ fileName: "font.bin", uri: "file://font", format: "opentype" }), "font/otf");
  assert.equal(fontMimeType({ fileName: "font.woff", uri: "file://font" }), "font/woff");
  assert.equal(fontMimeType({ fileName: "font.ttf", uri: "file://font" }), "font/ttf");
  assert.equal(fontMimeType({ fileName: "font.unknown", uri: "file://font" }), "font/ttf");
});

test("buildInstalledFontFaceRule includes CSS metadata and escapes format strings", () => {
  const rule = buildInstalledFontFaceRule(
    { fontFamilySpec: { family: "A \"Family\"" }, files: [] },
    {
      fileName: "regular.woff2",
      uri: "file:///regular.woff2",
      format: "woff2\"test",
      style: "italic",
      unicodeRange: "U+0000-00FF",
      weight: "700",
    },
    "BASE64",
  );

  assert.match(rule, /font-family: "A \\"Family\\"";/);
  assert.match(rule, /data:font\/woff2;base64,BASE64/);
  assert.match(rule, /format\("woff2\\"test"\)/);
  assert.match(rule, /font-weight: 700;/);
  assert.match(rule, /font-style: italic;/);
  assert.match(rule, /unicode-range: U\+0000-00FF;/);
});

test("buildInstalledFontFaceCssRules reads only selected font families", async () => {
  const readUris = [];
  const css = await buildInstalledFontFaceCssRules(
    [
      {
        fontFamilySpec: { family: "Body" },
        files: [
          {
            fileName: "body.woff2",
            uri: "file:///body.woff2",
            format: "woff2",
          },
        ],
      },
      {
        fontFamilySpec: { family: "Code" },
        files: [
          {
            fileName: "code.woff2",
            uri: "file:///code.woff2",
            format: "woff2",
          },
        ],
      },
    ],
    [{ family: "Code" }],
    async (file) => {
      readUris.push(file.uri);
      return file.uri.includes("code") ? "CODE" : "BODY";
    },
  );

  assert.deepEqual(readUris, ["file:///code.woff2"]);
  assert.match(css, /font-family: "Code";/);
  assert.doesNotMatch(css, /font-family: "Body";/);
});

test("selectedFontFamilies returns null only when no selection was supplied", () => {
  assert.equal(selectedFontFamilies(undefined), null);
  assert.deepEqual([...selectedFontFamilies([{ family: "A" }, undefined])], ["A"]);
});
