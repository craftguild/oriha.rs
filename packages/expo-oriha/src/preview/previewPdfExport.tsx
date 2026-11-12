// PDF snapshot validation, render input, printing and shared export results.
import { File as ExpoFile } from "expo-file-system";
import * as FileSystem from "expo-file-system/legacy";
import * as Print from "expo-print";
import { buildInstalledFontFaceCss } from "../fonts/fontRegistry";
import { buildPdfHtml } from "./orihaPreviewHtml";
import { buildPdfRenderInput, normalizePdfExportOptions, pdfPageDimensions } from "./orihaPreviewPdf";
import type { BridgeRenderInput, OrihaPdfExportOptions, OrihaPdfExportResult } from "../types";
import type { OrihaInstalledFont } from "../fonts/fontRegistry";
import { renderPreview } from "./previewRendering";
import type { OrihaMeasureTextBatch } from "./previewRendering";

export const PDF_MIME_TYPE = "application/pdf";

export type OrihaPreviewExportSnapshot = {
  input: BridgeRenderInput;
  installedFonts: OrihaInstalledFont[];
  isReady: boolean;
  measureTextBatch?: OrihaMeasureTextBatch;
};

export type ExportPreviewSnapshotToPdfOptions = {
  generationId: number;
  options?: OrihaPdfExportOptions;
  snapshot: OrihaPreviewExportSnapshot;
};

export async function exportPreviewSnapshotToPdf({
  generationId,
  options,
  snapshot,
}: ExportPreviewSnapshotToPdfOptions): Promise<OrihaPdfExportResult> {
  const normalizedOptions = normalizePdfExportOptions(options);
  const pdfInput = buildPdfRenderInput(
    snapshot.input,
    options,
    normalizedOptions,
  );
  const output = await renderPreview({
    generationId,
    input: pdfInput,
    measureTextBatch: snapshot.measureTextBatch,
  });
  const localFontFaceCss = await buildInstalledFontFaceCss(
    snapshot.installedFonts,
    [
      pdfInput.renderingOptions?.customHeadingFont,
      pdfInput.renderingOptions?.customBodyFont,
      pdfInput.renderingOptions?.customCodeFont,
    ],
  );
  const html = buildPdfHtml(output, localFontFaceCss);
  const dimensions = pdfPageDimensions(
    normalizedOptions.paperSize,
    normalizedOptions.orientation,
  );
  const printed = await Print.printToFileAsync({
    base64: false,
    height: dimensions.height,
    html,
    width: dimensions.width,
  });
  const temporaryUri = printed.uri;

  const pdfFile = new ExpoFile(temporaryUri);
  const blob = pdfFile as Blob;
  const byteLength =
    typeof blob.size === "number"
      ? blob.size
      : pdfFile.info().size ?? 0;
  return {
    blob,
    byteLength,
    dispose: () => FileSystem.deleteAsync(temporaryUri, { idempotent: true }),
    mimeType: PDF_MIME_TYPE,
    pageCount: printed.numberOfPages,
    temporaryUri,
  };
}
