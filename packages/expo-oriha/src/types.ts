import type React from "react";
import type { WebViewProps } from "react-native-webview";
import type {
  BridgeRenderInput,
  BridgeRenderingOptions,
  RenderOutput,
  WatermarkOptions,
} from "../generated/generatedBridgeTypes";

export type {
  BridgeAvailableOptions,
  BridgeControlAvailability,
  BridgeEffectiveSettingsInput,
  BridgeFunctionMap,
  BridgeFontOption,
  BridgeOption,
  BridgeRenderInput,
  BridgeRenderingOptions,
  BridgeResolvedSettings,
  BridgeSettingsControls,
  BridgeSettingsInput,
  ExportHtmlInput,
  ExportHtmlOutput,
  ExportOptions,
  FontFamilySpec,
  HtmlExportOptions,
  KindCount,
  OrihaBuiltinLocale,
  NativeBridgeVersionInfo,
  PreparedHtmlExport,
  PreparedMarkdownLayout,
  PreparedMarkdownPatch,
  PrintOptions,
  RenderAsset,
  RenderDiagnostic,
  RenderOptions,
  RenderOutput,
  RenderPatchOperation,
  RenderPatchOperationKind,
  RenderPatchOutput,
  RenderSnapshot,
  RenderSnapshotBlock,
  RenderStats,
  RenderedBlock,
  SourceRange,
  TextMeasurementRect,
  TextMeasurementBatch,
  TextMeasurementRequest,
  TextMeasurementResult,
  TextMeasurementRun,
  TextMeasurementRunResult,
  TextMeasurementStyle,
  HttpHeader,
  SendHttpContext,
  UsageAnalyticsOutboxCommand,
  UsageAnalyticsOutboxCommandResult,
  UsageAnalyticsOutboxConfig,
  UsageAnalyticsOutboxDeliveryStateInput,
  UsageAnalyticsOutboxEnqueueStartInput,
  UsageAnalyticsOutboxResumeInput,
  UsageAnalyticsOutboxStartInput,
  UsageAnalyticsOutboxStep,
  UsageAnalyticsRenderPayloadInput,
  UsageAnalyticsStoredEnvelope,
  Viewport,
  WatermarkOptions,
} from "../generated/generatedBridgeTypes";
export {
  AppearanceStyle,
  BodyFont,
  BridgeFeature,
  CodeFont,
  ColorScheme,
  HeadingFont,
  Plan,
  RenderAssetKind,
  RenderDiagnosticEngine,
  RenderDiagnosticSeverity,
  RenderMode,
  RenderPatchFallbackReason,
  RenderPatchStrategy,
  SyntaxTheme,
  TableBorderStyle,
  UsageAnalyticsOutboxCommandKind,
} from "../generated/generatedBridgeTypes";

export type RenderInput = BridgeRenderInput;
export type RenderingOptions = BridgeRenderingOptions;

export const OrihaPdfPaperSize: {
  readonly A4: "A4";
  readonly B4: "B4";
  readonly Letter: "Letter";
} = {
  A4: "A4",
  B4: "B4",
  Letter: "Letter",
};

export type OrihaPdfPaperSize =
  (typeof OrihaPdfPaperSize)[keyof typeof OrihaPdfPaperSize];

export const OrihaPdfOrientation: {
  readonly Portrait: "Portrait";
  readonly Landscape: "Landscape";
} = {
  Portrait: "Portrait",
  Landscape: "Landscape",
};

export type OrihaPdfOrientation =
  (typeof OrihaPdfOrientation)[keyof typeof OrihaPdfOrientation];

export type OrihaPdfExportOptions = {
  paperSize?: OrihaPdfPaperSize;
  orientation?: OrihaPdfOrientation;
  margin?: string;
  watermark?: false | Partial<WatermarkOptions>;
  renderingOptions?: Partial<BridgeRenderingOptions>;
};

export type OrihaPdfExportResult = {
  blob: Blob;
  mimeType: "application/pdf";
  byteLength: number;
  pageCount?: number;
  temporaryUri: string;
  dispose: () => Promise<void>;
};

export type OrihaPreviewHandle = {
  exportPdf: (
    options?: OrihaPdfExportOptions,
  ) => Promise<OrihaPdfExportResult>;
};

export type OrihaPreviewDebugOptions = {
  renderedHtmlFileName?: string;
  shareRenderedHtmlLabel?: string;
  enabled?: boolean;
};

export type OrihaPreviewInput = Partial<
  Omit<BridgeRenderInput, "markdown" | "renderingOptions">
> & {
  renderingOptions?: Partial<BridgeRenderingOptions>;
};

export type OrihaPreviewProps = {
  markdown: string;
  input?: OrihaPreviewInput;
  loadingFallback?: React.ReactNode;
  debug?: boolean | OrihaPreviewDebugOptions;
  onRendered?: (output: RenderOutput) => void;
  onRenderError?: (error: unknown) => void;
  webViewProps?: Omit<WebViewProps, "source">;
};
