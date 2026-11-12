import type {
  BridgeRenderInput,
  OrihaPdfExportOptions,
  OrihaPdfOrientation,
  OrihaPdfPaperSize,
  WatermarkOptions,
} from "../types";

export const DEFAULT_PDF_MARGIN = "12mm";
export const DEFAULT_PDF_WATERMARK_TEXT = "Generated with markup";
export const DEFAULT_PDF_WATERMARK_URL = "https://markup.progressiveworks.co";
const DEFAULT_PDF_ORIENTATION: OrihaPdfOrientation = "Portrait";
const DEFAULT_PDF_PAPER_SIZE: OrihaPdfPaperSize = "A4";
const LANDSCAPE_PDF_ORIENTATION: OrihaPdfOrientation = "Landscape";

export type NormalizedPdfExportOptions = {
  margin: string;
  orientation: OrihaPdfOrientation;
  paperSize: OrihaPdfPaperSize;
};

export function normalizePdfExportOptions(
  options: OrihaPdfExportOptions | undefined,
): NormalizedPdfExportOptions {
  return {
    margin: options?.margin ?? DEFAULT_PDF_MARGIN,
    orientation: options?.orientation ?? DEFAULT_PDF_ORIENTATION,
    paperSize: options?.paperSize ?? DEFAULT_PDF_PAPER_SIZE,
  };
}

export function buildPdfRenderInput(
  baseInput: BridgeRenderInput,
  options: OrihaPdfExportOptions | undefined,
  normalizedOptions: NormalizedPdfExportOptions,
): BridgeRenderInput {
  const renderingOptionsOverride = options?.renderingOptions;
  const renderingOptions = {
    ...baseInput.renderingOptions,
    ...(renderingOptionsOverride ?? {}),
  };
  const mergedInput: BridgeRenderInput = {
    ...baseInput,
    markdown: baseInput.markdown,
    renderingOptions,
  };
  const watermark = resolvePdfWatermark(
    baseInput.watermark,
    options?.watermark,
  );

  return {
    ...mergedInput,
    print: {
      ...(baseInput.print ?? {}),
      margin: normalizedOptions.margin,
      pageSize: pdfPageSizeCss(
        normalizedOptions.paperSize,
        normalizedOptions.orientation,
      ),
    },
    watermark,
  };
}

export function resolvePdfWatermark(
  existingWatermark: WatermarkOptions | undefined,
  requestedWatermark: OrihaPdfExportOptions["watermark"],
): WatermarkOptions | undefined {
  if (requestedWatermark === false) {
    return undefined;
  }
  if (requestedWatermark) {
    return normalizeWatermark({
      ...defaultPdfWatermark(),
      ...existingWatermark,
      ...requestedWatermark,
    });
  }
  return existingWatermark;
}

export function defaultPdfWatermark(): WatermarkOptions {
  return {
    text: DEFAULT_PDF_WATERMARK_TEXT,
  };
}

export function normalizeWatermark(
  watermark: Partial<WatermarkOptions>,
): WatermarkOptions {
  return {
    imageSrc: watermark.imageSrc,
    opacity: watermark.opacity,
    text:
      typeof watermark.text === "string" && watermark.text.trim().length > 0
        ? watermark.text
        : DEFAULT_PDF_WATERMARK_TEXT,
  };
}

export function pdfPageSizeCss(
  paperSize: OrihaPdfPaperSize,
  orientation: OrihaPdfOrientation,
): string {
  return `${paperSize} ${pdfOrientationCss(orientation)}`;
}

export function pdfOrientationCss(orientation: OrihaPdfOrientation): string {
  switch (orientation) {
    case LANDSCAPE_PDF_ORIENTATION:
      return "landscape";
    default:
      return "portrait";
  }
}

export function pdfPageDimensions(
  paperSize: OrihaPdfPaperSize,
  orientation: OrihaPdfOrientation,
): { height: number; width: number } {
  const dimensions = PDF_PAGE_DIMENSIONS[paperSize];
  if (orientation === LANDSCAPE_PDF_ORIENTATION) {
    return {
      height: dimensions.width,
      width: dimensions.height,
    };
  }
  return dimensions;
}

const PDF_PAGE_DIMENSIONS: Record<
  OrihaPdfPaperSize,
  { height: number; width: number }
> = {
  A4: { height: 842, width: 595 },
  B4: { height: 1032, width: 729 },
  Letter: { height: 792, width: 612 },
};
