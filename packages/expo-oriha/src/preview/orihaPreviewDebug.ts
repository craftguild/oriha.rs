import type { OrihaFormatMessage } from "../runtime/expoRuntimeI18n";
import type {
  OrihaPreviewDebugOptions,
  OrihaPreviewProps,
} from "../types";

export const DEFAULT_DEBUG_RENDERED_HTML_FILE_NAME = "oriha-rendered-preview.html";

export function normalizeOrihaPreviewDebugOptions(
  debug: OrihaPreviewProps["debug"],
  formatMessage: OrihaFormatMessage,
): Required<OrihaPreviewDebugOptions> {
  const shareRenderedHtmlLabel = formatMessage("debug.shareRenderedHtml");
  if (debug === true) {
    return {
      enabled: true,
      renderedHtmlFileName: DEFAULT_DEBUG_RENDERED_HTML_FILE_NAME,
      shareRenderedHtmlLabel,
    };
  }
  if (!debug) {
    return {
      enabled: false,
      renderedHtmlFileName: DEFAULT_DEBUG_RENDERED_HTML_FILE_NAME,
      shareRenderedHtmlLabel,
    };
  }
  return {
    enabled: debug.enabled ?? true,
    renderedHtmlFileName:
      debug.renderedHtmlFileName ?? DEFAULT_DEBUG_RENDERED_HTML_FILE_NAME,
    shareRenderedHtmlLabel:
      debug.shareRenderedHtmlLabel ?? shareRenderedHtmlLabel,
  };
}

export function ensureTrailingSlash(value: string): string {
  return value.endsWith("/") ? value : `${value}/`;
}

export function sanitizeDebugRenderedHtmlFileName(value: string): string {
  const name = value.trim().replace(/[^A-Za-z0-9._-]+/g, "-");
  const htmlName =
    name.length > 0 ? name : DEFAULT_DEBUG_RENDERED_HTML_FILE_NAME;
  return htmlName.toLowerCase().endsWith(".html")
    ? htmlName
    : `${htmlName}.html`;
}
