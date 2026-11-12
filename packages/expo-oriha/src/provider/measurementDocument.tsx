// Complete measurement document and batch injection, preserving the WebView wire protocol.
import { measurementStyleSource } from "./measurementStyleSource";
import { measurementFontSource } from "./measurementFontSource";
import { measurementResultSource } from "./measurementResultSource";
import { measurementHtmlSource } from "./measurementHtmlSource";
import { measurementDiagramSource } from "./measurementDiagramSource";
import { measurementBatchSource } from "./measurementBatchSource";
import type { TextMeasurementRequest, TextMeasurementResult } from "../types";

export const MESSAGE_TYPE_BATCH_RESULT = "oriha.measureTextBatchResult";

export type MeasureBatchInjectionPayload = {
  id: string;
  generationId: number;
  requests: TextMeasurementRequest[];
};

export type MeasureBatchResultMessage = {
  type: typeof MESSAGE_TYPE_BATCH_RESULT;
  id: string;
  generationId: number;
  results: TextMeasurementResult[];
  error?: string;
};

export function buildMeasureBatchScript(payload: MeasureBatchInjectionPayload): string {
  return `
(function() {
  if (window.__orihaMeasureTextBatch) {
    window.__orihaMeasureTextBatch(${JSON.stringify(payload)});
  }
})();
true;`;
}

export function buildMeasurementWebViewHtml(fontFaceCss: string): string {
  const escapedFontFaceCss = fontFaceCss.replace(/<\/style/gi, "<\\/style");
  return `<!doctype html>
<html>
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <style>
    ${escapedFontFaceCss}
    html, body {
      margin: 0;
      padding: 0;
      background: transparent;
    }
  </style>
</head>
<body>
  <script>
    (function() {
${measurementStyleSource}${measurementFontSource}${measurementResultSource}${measurementHtmlSource}${measurementDiagramSource}${measurementBatchSource(MESSAGE_TYPE_BATCH_RESULT)}    })();
  </script>
</body>
</html>`;
}
