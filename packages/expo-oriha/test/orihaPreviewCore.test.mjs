import assert from "node:assert/strict";
import test from "node:test";

import {
  DEFAULT_DEBUG_RENDERED_HTML_FILE_NAME,
  ensureTrailingSlash,
  normalizeOrihaPreviewDebugOptions,
  sanitizeDebugRenderedHtmlFileName,
} from "../src/preview/orihaPreviewDebug.ts";
import {
  MATH_FONT_ASSET_ID,
  MATH_FONT_FAMILY,
  MATH_FONT_MIME_TYPE,
  CODE_COPY_MESSAGE_TYPE,
  EXTERNAL_LINK_MESSAGE_TYPE,
  PATCH_RESULT_MESSAGE_TYPE,
  buildPdfHtml,
  buildViewerHtml,
  parseViewerPatchResultMessage,
  shouldOpenExternally,
} from "../src/preview/orihaPreviewHtml.ts";
import {
  buildPdfRenderInput,
  normalizePdfExportOptions,
  pdfPageDimensions,
  resolvePdfWatermark,
} from "../src/preview/orihaPreviewPdf.ts";
import {
  buildPatchInjectionScript,
  decidePreviewPatchApplication,
  decidePreviewPatchResult,
  shouldRecoverPreviewSourceAfterLoad,
} from "../src/preview/orihaPreviewPatchController.ts";
import { stableSerialize } from "../src/preview/stableSerialize.ts";

const patchControllerSnapshot = {
  schemaVersion: 3,
  settingsFingerprint: "settings",
  nextStableIndex: 2,
  blocks: [],
  tailHash: "tail",
  tailHtmlByteLength: 0,
  outputResourceFingerprint: "resources",
};

function patchControllerOutput(overrides = {}) {
  return {
    fallbackReason: undefined,
    output: {
      html: "<main>full output should not be injected</main>",
      css: ".secret-output-css {}",
      assets: [],
      diagnostics: [],
      sourceMap: [],
      stats: {},
    },
    patches: [
      {
        op: "replaceBlocks",
        html: "<p>patch html</p>",
        removeStableIds: ["a"],
      },
    ],
    snapshot: patchControllerSnapshot,
    strategy: "incremental",
    ...overrides,
  };
}

function extractPatchInjectionPayload(script) {
  const match = script.match(/var payload = (.*);\n  var ok = false;/s);
  assert.ok(match, "expected injection script payload assignment");
  return JSON.parse(match[1]);
}
test("normalizeOrihaPreviewDebugOptions resolves defaults and overrides", () => {
  const formatMessage = (key) => `message:${key}`;

  assert.deepEqual(normalizeOrihaPreviewDebugOptions(false, formatMessage), {
    enabled: false,
    renderedHtmlFileName: DEFAULT_DEBUG_RENDERED_HTML_FILE_NAME,
    shareRenderedHtmlLabel: "message:debug.shareRenderedHtml",
  });
  assert.deepEqual(normalizeOrihaPreviewDebugOptions(true, formatMessage), {
    enabled: true,
    renderedHtmlFileName: DEFAULT_DEBUG_RENDERED_HTML_FILE_NAME,
    shareRenderedHtmlLabel: "message:debug.shareRenderedHtml",
  });
  assert.deepEqual(
    normalizeOrihaPreviewDebugOptions(
      {
        enabled: false,
        renderedHtmlFileName: "custom",
        shareRenderedHtmlLabel: "Share",
      },
      formatMessage,
    ),
    {
      enabled: false,
      renderedHtmlFileName: "custom",
      shareRenderedHtmlLabel: "Share",
    },
  );
});

test("debug file helpers normalize storage paths and filenames", () => {
  assert.equal(ensureTrailingSlash("file:///cache"), "file:///cache/");
  assert.equal(ensureTrailingSlash("file:///cache/"), "file:///cache/");
  assert.equal(
    sanitizeDebugRenderedHtmlFileName(" Report 2026 "),
    "Report-2026.html",
  );
  assert.equal(
    sanitizeDebugRenderedHtmlFileName("report.HTML"),
    "report.HTML",
  );
  assert.equal(
    sanitizeDebugRenderedHtmlFileName("   "),
    DEFAULT_DEBUG_RENDERED_HTML_FILE_NAME,
  );
});

test("stableSerialize ignores object key order and undefined fields", () => {
  const left = stableSerialize({
    markdown: "# Source",
    locale: "ja",
    renderingOptions: {
      shadows: true,
      roundedCorners: undefined,
      tableBorderStyle: "inner",
    },
  });
  const right = stableSerialize({
    renderingOptions: {
      tableBorderStyle: "inner",
      shadows: true,
    },
    locale: "ja",
    markdown: "# Source",
  });

  assert.equal(left, right);
});

test("stableSerialize keeps semantic render input changes visible", () => {
  assert.notEqual(
    stableSerialize({
      markdown: "# Source",
      renderingOptions: { tableBorderStyle: "inner" },
    }),
    stableSerialize({
      markdown: "# Source",
      renderingOptions: { tableBorderStyle: "all" },
    }),
  );
});

test("shouldOpenExternally allows only supported external URL schemes", () => {
  for (const url of [
    "https://example.com",
    "mailto:hello@example.com",
    "tel:+8112345678",
    "geo:35,139",
    "maps:q=tokyo",
  ]) {
    assert.equal(shouldOpenExternally(url), true, url);
  }

  for (const url of [
    undefined,
    "#heading",
    "javascript:alert(1)",
    "file:///tmp/report.html",
    "data:text/html,hello",
    "blob:https://example.com/id",
    "about:blank",
    "ftp://example.com",
  ]) {
    assert.equal(shouldOpenExternally(url), false, String(url));
  }
});

test("buildPdfRenderInput keeps source markdown and applies print options", () => {
  const normalizedOptions = normalizePdfExportOptions({
    margin: "20mm",
    orientation: "Landscape",
    paperSize: "Letter",
  });
  const input = buildPdfRenderInput(
    {
      markdown: "# Source",
      plan: "premium",
      renderingOptions: {
        colorScheme: "light",
        appearanceStyle: "rich",
        syntaxTheme: "tokyo-night",
        headingFont: "custom",
        customHeadingFont: { family: "Heading Face" },
        bodyFont: "custom",
        customBodyFont: { family: "Body Face" },
        codeFont: "custom",
        customCodeFont: { family: "Code Face", fallbacks: ["monospace"] },
        tableBorderStyle: "all",
        roundedCorners: true,
        shadows: true,
        fullTableData: false,
        blockBorders: false,
      },
      renderMode: "styledHtml",
      print: { margin: "1mm", pageSize: "A4 portrait" },
      watermark: { text: "Existing" },
      sourceAttributes: false,
      tableOfContents: false,
    },
    {
      renderingOptions: {
        colorScheme: "dark",
        shadows: false,
        fullTableData: true,
      },
      watermark: false,
    },
    normalizedOptions,
  );

  assert.equal(input.markdown, "# Source");
  assert.equal(input.renderingOptions?.colorScheme, "dark");
  assert.equal(input.renderingOptions?.appearanceStyle, "rich");
  assert.equal(input.renderingOptions?.syntaxTheme, "tokyo-night");
  assert.equal(input.renderingOptions?.headingFont, "custom");
  assert.deepEqual(input.renderingOptions?.customHeadingFont, { family: "Heading Face" });
  assert.equal(input.renderingOptions?.bodyFont, "custom");
  assert.deepEqual(input.renderingOptions?.customBodyFont, { family: "Body Face" });
  assert.equal(input.renderingOptions?.codeFont, "custom");
  assert.deepEqual(input.renderingOptions?.customCodeFont, {
    family: "Code Face",
    fallbacks: ["monospace"],
  });
  assert.equal(input.renderingOptions?.tableBorderStyle, "all");
  assert.equal(input.renderingOptions?.roundedCorners, true);
  assert.equal(input.renderingOptions?.shadows, false);
  assert.equal(input.renderingOptions?.fullTableData, true);
  assert.equal(input.renderingOptions?.blockBorders, false);
  assert.equal(input.renderMode, "styledHtml");
  assert.deepEqual(input.print, {
    margin: "20mm",
    pageSize: "Letter landscape",
  });
  assert.equal(input.watermark, undefined);
});

test("PDF watermark option normalization stays plan-agnostic", () => {
  assert.equal(resolvePdfWatermark(undefined, undefined), undefined);
  assert.deepEqual(
    resolvePdfWatermark(
      { text: "Existing", opacity: 0.1 },
      { text: "Requested", imageSrc: "data:image/png;base64,abc" },
    ),
    {
      imageSrc: "data:image/png;base64,abc",
      opacity: 0.1,
      text: "Requested",
    },
  );
  assert.equal(
    resolvePdfWatermark({ text: "Existing" }, false),
    undefined,
  );
  assert.deepEqual(resolvePdfWatermark(undefined, { text: " " }), {
    imageSrc: undefined,
    opacity: undefined,
    text: "Generated with markup",
  });
});

test("pdfPageDimensions swaps dimensions for landscape orientation", () => {
  assert.deepEqual(
    pdfPageDimensions("A4", "Portrait"),
    { height: 842, width: 595 },
  );
  assert.deepEqual(
    pdfPageDimensions("A4", "Landscape"),
    { height: 595, width: 842 },
  );
});

test("buildPdfHtml escapes closing style tags and injects bundled math font CSS", () => {
  const html = buildPdfHtml(
    {
      html: "<main>body</main>",
      css: ".x::after { content: \"</style>\"; }",
      assets: [
        {
          id: MATH_FONT_ASSET_ID,
          mimeType: MATH_FONT_MIME_TYPE,
          content: "MATHBASE64",
        },
      ],
    },
    ".local { content: \"</style>\"; }",
  );

  assert.match(html, /<!doctype html>/);
  assert.match(html, new RegExp(`font-family: "${MATH_FONT_FAMILY}"`));
  assert.match(html, /data:font\/otf;base64,MATHBASE64/);
  assert.match(html, /<main>body<\/main>/);
  assert.equal(html.includes("</style>\""), false);
  assert.match(html, /<\\\/style>"/);
});

test("buildViewerHtml injects viewer scripts with native message types", () => {
  const html = buildViewerHtml(
    {
      html: "<a href=\"https://example.com\">link</a>",
      css: `body { font-family: "${MATH_FONT_FAMILY}"; }`,
      assets: [
        {
          id: MATH_FONT_ASSET_ID,
          mimeType: MATH_FONT_MIME_TYPE,
          content: "MATHBASE64",
        },
      ],
    },
    "",
  );

  assert.match(html, new RegExp(EXTERNAL_LINK_MESSAGE_TYPE));
  assert.match(html, new RegExp(CODE_COPY_MESSAGE_TYPE));
  assert.match(html, /data-oriha-patch-root="true"/);
  assert.match(html, /orihaApplyPatch/);
  assert.match(html, /orihaConfigureExternalLinks/);
  assert.match(html, /fallbackParent = block\.parentNode/);
  assert.match(
    html,
    /fallbackParent \|\| root\.querySelector\("\.oriha-document"\) \|\| root/,
  );
  assert.equal(html.includes("data:font/otf;base64,MATHBASE64"), false);
});

test("parseViewerPatchResultMessage accepts only internal patch results", () => {
  assert.deepEqual(
    parseViewerPatchResultMessage(
      JSON.stringify({
        type: PATCH_RESULT_MESSAGE_TYPE,
        generationId: 7,
        ok: false,
        message: "anchor missing",
      }),
    ),
    {
      type: PATCH_RESULT_MESSAGE_TYPE,
      generationId: 7,
      ok: false,
      message: "anchor missing",
    },
  );
  assert.equal(
    parseViewerPatchResultMessage(
      JSON.stringify({
        type: EXTERNAL_LINK_MESSAGE_TYPE,
        generationId: 7,
        ok: true,
      }),
    ),
    null,
  );
  assert.equal(parseViewerPatchResultMessage("{"), null);
});

test("decidePreviewPatchApplication injects incremental patches only when the viewer is ready", () => {
  const decision = decidePreviewPatchApplication({
    generationId: 11,
    hasWebView: true,
    patchOutput: patchControllerOutput(),
    previousSnapshot: patchControllerSnapshot,
    webViewReady: true,
  });

  assert.equal(decision.action, "inject");
  assert.equal(decision.generationId, 11);
  assert.equal(decision.patchCount, 1);
  assert.match(decision.script, /orihaApplyPatch/);
  assert.deepEqual(extractPatchInjectionPayload(decision.script), {
    generationId: 11,
    patches: [
      {
        op: "replaceBlocks",
        html: "<p>patch html</p>",
        removeStableIds: ["a"],
      },
    ],
  });
});

test("decidePreviewPatchApplication uses full reload when incremental patching is unsafe", () => {
  assert.deepEqual(
    decidePreviewPatchApplication({
      generationId: 12,
      hasWebView: true,
      patchOutput: patchControllerOutput(),
      previousSnapshot: null,
      webViewReady: true,
    }),
    {
      action: "fullReload",
      generationId: 12,
      reason: "missingPreviousSnapshot",
    },
  );
  assert.deepEqual(
    decidePreviewPatchApplication({
      generationId: 13,
      hasWebView: true,
      patchOutput: patchControllerOutput({ strategy: "replaceAll" }),
      previousSnapshot: patchControllerSnapshot,
      webViewReady: true,
    }),
    {
      action: "fullReload",
      generationId: 13,
      reason: "replaceAllStrategy",
    },
  );
  assert.deepEqual(
    decidePreviewPatchApplication({
      generationId: 14,
      hasWebView: true,
      patchOutput: patchControllerOutput(),
      previousSnapshot: patchControllerSnapshot,
      webViewReady: false,
    }),
    {
      action: "fullReload",
      generationId: 14,
      reason: "webViewNotReady",
    },
  );
  assert.deepEqual(
    decidePreviewPatchApplication({
      generationId: 15,
      hasWebView: false,
      patchOutput: patchControllerOutput(),
      previousSnapshot: patchControllerSnapshot,
      webViewReady: true,
    }),
    {
      action: "fullReload",
      generationId: 15,
      reason: "webViewUnavailable",
    },
  );
});

test("decidePreviewPatchResult maps viewer responses to commit, fallback, or stale", () => {
  assert.deepEqual(
    decidePreviewPatchResult({
      currentGenerationId: 21,
      patchResult: {
        type: PATCH_RESULT_MESSAGE_TYPE,
        generationId: 21,
        ok: true,
      },
      pendingGenerationId: 21,
    }),
    {
      action: "incrementalCommit",
      generationId: 21,
    },
  );
  assert.deepEqual(
    decidePreviewPatchResult({
      currentGenerationId: 22,
      patchResult: {
        type: PATCH_RESULT_MESSAGE_TYPE,
        generationId: 22,
        ok: false,
        message: "missing stable block",
      },
      pendingGenerationId: 22,
    }),
    {
      action: "fullFallback",
      generationId: 22,
      message: "missing stable block",
    },
  );
  assert.deepEqual(
    decidePreviewPatchResult({
      currentGenerationId: 24,
      patchResult: {
        type: PATCH_RESULT_MESSAGE_TYPE,
        generationId: 23,
        ok: true,
      },
      pendingGenerationId: 23,
    }),
    {
      action: "stale",
      currentGenerationId: 24,
      generationId: 23,
      pendingGenerationId: 23,
    },
  );
});

test("buildPatchInjectionScript serializes only generation id and patches", () => {
  const script = buildPatchInjectionScript({
    generationId: 31,
    patches: patchControllerOutput().patches,
  });
  const payload = extractPatchInjectionPayload(script);

  assert.deepEqual(Object.keys(payload), ["generationId", "patches"]);
  assert.equal(payload.generationId, 31);
  assert.deepEqual(payload.patches, patchControllerOutput().patches);
  assert.equal(script.includes("full output should not be injected"), false);
  assert.equal(script.includes("secret-output-css"), false);
  assert.match(script, new RegExp(PATCH_RESULT_MESSAGE_TYPE));
});

test("shouldRecoverPreviewSourceAfterLoad reloads only stale sources with output", () => {
  assert.equal(
    shouldRecoverPreviewSourceAfterLoad({
      hasOutput: true,
      webViewSourceStale: true,
    }),
    true,
  );
  assert.equal(
    shouldRecoverPreviewSourceAfterLoad({
      hasOutput: false,
      webViewSourceStale: true,
    }),
    false,
  );
  assert.equal(
    shouldRecoverPreviewSourceAfterLoad({
      hasOutput: true,
      webViewSourceStale: false,
    }),
    false,
  );
});
