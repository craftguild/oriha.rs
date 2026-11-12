import type { RenderOutput } from "../types";

export const CODE_COPY_MESSAGE_TYPE = "oriha.codeCopyResult";
export const EXTERNAL_LINK_MESSAGE_TYPE =
  "oriha.openExternalLink";
export const PATCH_RESULT_MESSAGE_TYPE =
  "oriha.patchResult";
export const MATH_FONT_ASSET_ID = "oriha-stix-two-math";
export const MATH_FONT_MIME_TYPE = "font/otf";
export const MATH_FONT_FAMILY = "ORIHA STIX Two Math";

export type ViewerPatchResultMessage = {
  type?: string;
  generationId?: number;
  ok?: boolean;
  message?: string;
};

export function parseViewerPatchResultMessage(
  data: string,
): ViewerPatchResultMessage | null {
  let message: ViewerPatchResultMessage;
  try {
    message = JSON.parse(data) as ViewerPatchResultMessage;
  } catch {
    return null;
  }
  if (
    message.type !== PATCH_RESULT_MESSAGE_TYPE ||
    typeof message.generationId !== "number" ||
    typeof message.ok !== "boolean"
  ) {
    return null;
  }
  return message;
}

export function shouldOpenExternally(url: string | undefined): url is string {
  if (!url) {
    return false;
  }

  const lowerUrl = url.trim().toLowerCase();
  if (
    lowerUrl.startsWith("#") ||
    lowerUrl.startsWith("about:") ||
    lowerUrl.startsWith("blob:") ||
    lowerUrl.startsWith("data:") ||
    lowerUrl.startsWith("file:")
  ) {
    return false;
  }

  return /^(https?:|mailto:|tel:|sms:|geo:|maps:)/i.test(lowerUrl);
}

export function buildViewerHtml(
  output: RenderOutput,
  localFontFaceCss: string,
): string {
  return buildDocumentHtml({
    body: `<div data-oriha-patch-root="true">${output.html}</div>
${viewerExternalLinkScript()}
${viewerPatchScript()}
${viewerCodeCopyScript()}`,
    localFontFaceCss,
    output,
  });
}

export function buildPdfHtml(
  output: RenderOutput,
  localFontFaceCss: string,
): string {
  return buildDocumentHtml({
    body: output.html,
    localFontFaceCss,
    output,
  });
}

export function buildDocumentHtml({
  body,
  localFontFaceCss,
  output,
}: {
  body: string;
  localFontFaceCss: string;
  output: RenderOutput;
}): string {
  const outputCss = output.css ?? "";
  const bundledFontFaceCss = outputCss.includes(MATH_FONT_FAMILY)
    ? ""
    : buildBundledFontFaceCss(output);
  const styleCss = [
    bundledFontFaceCss,
    localFontFaceCss,
    outputCss,
  ]
    .filter((chunk) => chunk.length > 0)
    .join("\n")
    .replace(/<\/style/gi, "<\\/style");
  return `<!doctype html>
<html>
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=5" />
  <style>
    ${styleCss}
    html, body { margin: 0; padding: 0; background: transparent; }
    body { -webkit-text-size-adjust: 100%; }
  </style>
</head>
<body>
${body}
</body>
</html>`;
}

export function buildBundledFontFaceCss(output: RenderOutput): string {
  const asset = output.assets.find(
    (entry) =>
      entry.id === MATH_FONT_ASSET_ID &&
      entry.mimeType === MATH_FONT_MIME_TYPE &&
      entry.content.length > 0,
  );
  if (!asset) {
    return "";
  }
  return [
    "@font-face {",
    `  font-family: "${MATH_FONT_FAMILY}";`,
    `  src: url("data:${MATH_FONT_MIME_TYPE};base64,${asset.content}") format("opentype");`,
    "  font-weight: 400;",
    "  font-style: normal;",
    "  font-display: block;",
    "}",
    `.oriha-math-inline svg text, .oriha-math-block svg text { font-family: "${MATH_FONT_FAMILY}", "Cambria Math", "STIX Two Math", serif; }`,
  ].join("\n");
}

export function viewerExternalLinkScript(): string {
  return `<script>
(function() {
  var EXTERNAL_SCHEME = /^(https?:|mailto:|tel:|sms:|geo:|maps:)/i;

  function postExternalLink(url) {
    if (!window.ReactNativeWebView || !window.ReactNativeWebView.postMessage) {
      return;
    }
    window.ReactNativeWebView.postMessage(JSON.stringify({
      type: "${EXTERNAL_LINK_MESSAGE_TYPE}",
      url: url
    }));
  }

  function externalHref(anchor) {
    var rawHref = anchor && anchor.getAttribute ? anchor.getAttribute("href") : "";
    if (!rawHref || rawHref.charAt(0) === "#") {
      return null;
    }
    if (/^\\s*javascript:/i.test(rawHref)) {
      return null;
    }
    try {
      var resolved = new URL(rawHref, window.location.href);
      if (!EXTERNAL_SCHEME.test(resolved.protocol)) {
        return null;
      }
      return resolved.href;
    } catch (_) {
      return EXTERNAL_SCHEME.test(rawHref) ? rawHref : null;
    }
  }

  function configureExternalLinks(root) {
    var anchors = (root || document).querySelectorAll("a[href]");
    for (var index = 0; index < anchors.length; index += 1) {
      var href = externalHref(anchors[index]);
      if (!href) {
        continue;
      }
      anchors[index].setAttribute("target", "_blank");
      anchors[index].setAttribute("rel", "noopener noreferrer");
    }
  }

  window.orihaConfigureExternalLinks = configureExternalLinks;
  configureExternalLinks(document);
  document.addEventListener("DOMContentLoaded", function() {
    configureExternalLinks(document);
  });

  document.addEventListener("click", function(event) {
    var anchor = event.target.closest && event.target.closest("a[href]");
    var href = externalHref(anchor);
    if (!href) {
      return;
    }

    event.preventDefault();
    postExternalLink(href);
  }, true);
})();
</script>`;
}

export function viewerPatchScript(): string {
  return `<script>
(function() {
  function patchRoot() {
    return document.querySelector("[data-oriha-patch-root]") ||
      document.querySelector(".oriha-document") ||
      document.body;
  }

  function findStableBlock(root, stableId) {
    if (!stableId) {
      return null;
    }
    var blocks = root.querySelectorAll("[data-oriha-stable-id]");
    var expected = String(stableId);
    for (var index = 0; index < blocks.length; index += 1) {
      if (blocks[index].getAttribute("data-oriha-stable-id") === expected) {
        return blocks[index];
      }
    }
    return null;
  }

  function nodesFromHtml(html) {
    var template = document.createElement("template");
    template.innerHTML = html || "";
    return Array.prototype.slice.call(template.content.childNodes);
  }

  function firstVisibleStableBlock(root) {
    var blocks = root.querySelectorAll("[data-oriha-stable-id]");
    var viewportHeight = window.innerHeight || document.documentElement.clientHeight || 0;
    for (var index = 0; index < blocks.length; index += 1) {
      var rect = blocks[index].getBoundingClientRect();
      if (rect.bottom >= 0 && rect.top <= viewportHeight) {
        return {
          stableId: blocks[index].getAttribute("data-oriha-stable-id"),
          top: rect.top
        };
      }
    }
    return null;
  }

  function restoreScroll(root, anchor) {
    if (!anchor || !anchor.stableId) {
      return;
    }
    var block = findStableBlock(root, anchor.stableId);
    if (!block) {
      return;
    }
    var nextTop = block.getBoundingClientRect().top;
    window.scrollBy(0, nextTop - anchor.top);
  }

  function configureExternalLinks(root) {
    if (window.orihaConfigureExternalLinks) {
      window.orihaConfigureExternalLinks(root || document);
    }
  }

  function replaceAll(root, html) {
    var anchor = firstVisibleStableBlock(root);
    root.innerHTML = html || "";
    configureExternalLinks(root);
    restoreScroll(root, anchor);
    return true;
  }

  function replaceBlocks(root, operation) {
    var removeStableIds = operation.removeStableIds || [];
    var insertAfter = operation.insertAfterStableId ?
      findStableBlock(root, operation.insertAfterStableId) :
      null;
    var insertBefore = operation.insertBeforeStableId ?
      findStableBlock(root, operation.insertBeforeStableId) :
      null;

    if (operation.insertAfterStableId && !insertAfter) {
      return false;
    }
    if (operation.insertBeforeStableId && !insertBefore) {
      return false;
    }

    var anchor = firstVisibleStableBlock(root);
    var fallbackParent = null;
    for (var index = 0; index < removeStableIds.length; index += 1) {
      var block = findStableBlock(root, removeStableIds[index]);
      if (!block) {
        return false;
      }
      if (block.parentNode) {
        if (!fallbackParent) {
          fallbackParent = block.parentNode;
        }
        block.parentNode.removeChild(block);
      }
    }

    var nodes = nodesFromHtml(operation.html);
    if (nodes.length > 0) {
      var parent = fallbackParent || root.querySelector(".oriha-document") || root;
      var reference = null;
      if (insertBefore && insertBefore.parentNode) {
        parent = insertBefore.parentNode;
        reference = insertBefore;
      } else if (insertAfter && insertAfter.parentNode) {
        parent = insertAfter.parentNode;
        reference = insertAfter.nextSibling;
      }
      for (var nodeIndex = 0; nodeIndex < nodes.length; nodeIndex += 1) {
        parent.insertBefore(nodes[nodeIndex], reference);
      }
      configureExternalLinks(parent);
    }

    restoreScroll(root, anchor);
    return true;
  }

  window.orihaApplyPatch = function(payload) {
    var root = patchRoot();
    if (!root || !payload || !Array.isArray(payload.patches)) {
      return false;
    }

    for (var index = 0; index < payload.patches.length; index += 1) {
      var operation = payload.patches[index];
      var ok = false;
      if (operation && operation.op === "replaceAll") {
        ok = replaceAll(root, operation.html);
      } else if (operation && operation.op === "replaceBlocks") {
        ok = replaceBlocks(root, operation);
      }
      if (!ok) {
        return false;
      }
    }
    return true;
  };
})();
</script>`;
}

export function viewerCodeCopyScript(): string {
  return `<script>
(function() {
  function postCodeCopyResult(status) {
    if (!window.ReactNativeWebView || !window.ReactNativeWebView.postMessage) {
      return;
    }
    window.ReactNativeWebView.postMessage(JSON.stringify({
      type: "${CODE_COPY_MESSAGE_TYPE}",
      status: status
    }));
  }

  function fallbackCopy(text) {
    var textarea = document.createElement("textarea");
    textarea.value = text;
    textarea.setAttribute("readonly", "");
    textarea.style.position = "fixed";
    textarea.style.left = "-9999px";
    document.body.appendChild(textarea);
    textarea.select();
    try {
      if (document.execCommand("copy")) {
        return Promise.resolve();
      }
      return Promise.reject(new Error("copy command rejected"));
    } finally {
      document.body.removeChild(textarea);
    }
  }

  function copyText(text) {
    if (navigator.clipboard && navigator.clipboard.writeText) {
      return navigator.clipboard.writeText(text);
    }
    return fallbackCopy(text);
  }

  document.addEventListener("click", function(event) {
    var button = event.target.closest && event.target.closest("[data-oriha-copy-button]");
    if (!button) {
      return;
    }

    var wrapper = button.closest(".oriha-code-block-wrapper");
    if (!wrapper) {
      return;
    }

    event.preventDefault();
    copyText(wrapper.getAttribute("data-oriha-code-copy") || "").then(function() {
      postCodeCopyResult("success");
    }, function() {
      postCodeCopyResult("error");
    });
  }, true);
})();
</script>`;
}
