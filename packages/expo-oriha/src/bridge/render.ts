import ExpoOrihaModule from "./ExpoOrihaModule";
import {
  parseBridgePayload,
  stringifyBridgePayload,
  stringifyOptionalBridgePayload,
} from "./json";
import {
  AppearanceStyle,
  BodyFont,
  CodeFont,
  ColorScheme,
  HeadingFont,
  Plan,
  RenderMode,
  SyntaxTheme,
  TableBorderStyle,
} from "../types";
import type {
  BridgeRenderingOptions,
  BridgeRenderInput,
  BridgeResolvedSettings,
  BridgeSettingsInput,
  ExportOptions,
  ExportHtmlOutput,
  HtmlExportOptions,
  PreparedHtmlExport,
  PreparedMarkdownPatch,
  PreparedMarkdownLayout,
  RenderOutput,
  RenderPatchOutput,
  RenderSnapshot,
  TextMeasurementRequest,
  TextMeasurementResult,
} from "../types";

export type BridgeRenderInputDraft = Partial<
  Omit<BridgeRenderInput, "markdown" | "renderingOptions">
> & {
  markdown: string;
  renderingOptions?: Partial<BridgeRenderingOptions>;
};

type LegacyBridgeRenderInputDraft = BridgeRenderInputDraft & {
  exportOptions?: ExportOptions;
  mode?: RenderMode;
};

export type HtmlExportMeasureTextBatch = (
  requests: TextMeasurementRequest[],
) => Promise<TextMeasurementResult[]>;

export async function prepareMarkdownLayout(
  input: BridgeRenderInputDraft,
): Promise<PreparedMarkdownLayout> {
  return parseBridgePayload(
    await ExpoOrihaModule.prepareMarkdownLayout(
      stringifyBridgePayload(normalizeBridgeRenderInput(input)),
    ),
  );
}

export async function prepareMarkdownPatch(
  previousSnapshot: RenderSnapshot | null | undefined,
  input: BridgeRenderInputDraft,
): Promise<PreparedMarkdownPatch> {
  return parseBridgePayload(
    await ExpoOrihaModule.prepareMarkdownPatch(
      stringifyOptionalBridgePayload(previousSnapshot),
      stringifyBridgePayload(normalizeBridgeRenderInput(input)),
    ),
  );
}

export async function prepareHtmlExport(
  input: BridgeRenderInputDraft,
  options?: HtmlExportOptions,
): Promise<PreparedHtmlExport> {
  return parseBridgePayload(
    await ExpoOrihaModule.prepareHtmlExport(
      stringifyBridgePayload(normalizeBridgeRenderInput(input)),
      stringifyOptionalBridgePayload(options),
    ),
  );
}

export async function resolveRenderSettings(
  input: BridgeSettingsInput,
): Promise<BridgeResolvedSettings> {
  return parseBridgePayload(
    await ExpoOrihaModule.resolveRenderSettings(
      stringifyBridgePayload(input),
    ),
  );
}

export async function renderPreparedMarkdown(
  layout: PreparedMarkdownLayout,
  measureResults: TextMeasurementResult[],
): Promise<RenderOutput> {
  return parseBridgePayload(
    await ExpoOrihaModule.renderPreparedMarkdown(
      stringifyBridgePayload(layout),
      stringifyBridgePayload(measureResults),
    ),
  );
}

export async function renderPreparedMarkdownPatch(
  preparedPatch: PreparedMarkdownPatch,
  measureResults: TextMeasurementResult[],
): Promise<RenderPatchOutput> {
  return parseBridgePayload(
    await ExpoOrihaModule.renderPreparedMarkdownPatch(
      stringifyBridgePayload(preparedPatch),
      stringifyBridgePayload(measureResults),
    ),
  );
}

export async function renderPreparedHtmlExport(
  preparedExport: PreparedHtmlExport,
  measureResults: TextMeasurementResult[],
): Promise<ExportHtmlOutput> {
  return parseBridgePayload(
    await ExpoOrihaModule.renderPreparedHtmlExport(
      stringifyBridgePayload(preparedExport),
      stringifyBridgePayload(measureResults),
    ),
  );
}

export async function exportHtml(
  input: BridgeRenderInputDraft,
  options?: HtmlExportOptions,
  measureTextBatch?: HtmlExportMeasureTextBatch,
): Promise<ExportHtmlOutput> {
  const preparedExport = await prepareHtmlExport(input, options);
  const measureRequests = preparedExport.textMeasurements.requests;
  const measureResults =
    measureRequests.length > 0
      ? await measureRequiredHtmlExportText(preparedExport, measureTextBatch)
      : [];
  return renderPreparedHtmlExport(preparedExport, measureResults);
}

async function measureRequiredHtmlExportText(
  preparedExport: PreparedHtmlExport,
  measureTextBatch: HtmlExportMeasureTextBatch | undefined,
): Promise<TextMeasurementResult[]> {
  if (!measureTextBatch) {
    throw new Error(
      "exportHtml requires measureTextBatch when the prepared export has text measurement requests",
    );
  }
  return measureTextBatch(preparedExport.textMeasurements.requests);
}

export function normalizeBridgeRenderInput(
  input: BridgeRenderInputDraft,
): BridgeRenderInput {
  const legacyInput = input as LegacyBridgeRenderInputDraft;
  const legacyExportOptions = legacyInput.exportOptions;
  return {
    markdown: input.markdown,
    locale: input.locale,
    plan: input.plan ?? Plan.Free,
    renderingOptions: normalizeBridgeRenderingOptions(input.renderingOptions),
    print: input.print ?? legacyExportOptions?.print,
    renderMode: input.renderMode ?? legacyInput.mode ?? RenderMode.StyledHtml,
    watermark: input.watermark ?? legacyExportOptions?.watermark,
    sourceAttributes:
      input.sourceAttributes ?? legacyExportOptions?.sourceAttributes ?? false,
    tableOfContents:
      input.tableOfContents ?? legacyExportOptions?.tableOfContents ?? false,
    viewport: input.viewport,
  };
}

export function normalizeBridgeRenderingOptions(
  options?: Partial<BridgeRenderingOptions>,
): BridgeRenderingOptions {
  return {
    colorScheme: options?.colorScheme ?? ColorScheme.Light,
    appearanceStyle: options?.appearanceStyle ?? AppearanceStyle.Rich,
    syntaxTheme: options?.syntaxTheme ?? SyntaxTheme.GitHubLight,
    customSyntaxTheme: options?.customSyntaxTheme,
    headingFont: options?.headingFont ?? HeadingFont.SystemDefault,
    customHeadingFont: options?.customHeadingFont,
    bodyFont: options?.bodyFont ?? BodyFont.SystemDefault,
    customBodyFont: options?.customBodyFont,
    bodyFontSizePx: options?.bodyFontSizePx,
    codeFont: options?.codeFont ?? CodeFont.SystemDefault,
    customCodeFont: options?.customCodeFont,
    tableBorderStyle: options?.tableBorderStyle ?? TableBorderStyle.Inner,
    roundedCorners: options?.roundedCorners ?? true,
    shadows: options?.shadows ?? true,
    fullTableData: options?.fullTableData ?? false,
    blockBorders: options?.blockBorders ?? false,
  };
}
