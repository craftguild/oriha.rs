import { requireNativeModule } from "expo";
import type { NativeBridgeVersionInfo } from "../types";

export type ExpoOrihaNativeModule = {
  nativeBridgeVersionInfo(): Promise<NativeBridgeVersionInfo>;
  prepareMarkdownLayout(inputJson: string): Promise<string>;
  prepareMarkdownPatch(
    previousSnapshotJson: string | null,
    inputJson: string,
  ): Promise<string>;
  prepareHtmlExport(
    inputJson: string,
    optionsJson: string | null,
  ): Promise<string>;
  resolveRenderSettings(inputJson: string): Promise<string>;
  renderPreparedMarkdown(
    preparedJson: string,
    measureResultsJson: string,
  ): Promise<string>;
  renderPreparedMarkdownPatch(
    preparedJson: string,
    measureResultsJson: string,
  ): Promise<string>;
  renderPreparedHtmlExport(
    preparedJson: string,
    measureResultsJson: string,
  ): Promise<string>;
  usageAnalyticsOutboxStartEnqueue(
    inputJson: string,
    nowMs: number,
  ): Promise<string>;
  usageAnalyticsOutboxStartDrain(
    inputJson: string,
    nowMs: number,
  ): Promise<string>;
  usageAnalyticsOutboxStartDeliveryStateChange(
    inputJson: string,
  ): Promise<string>;
  usageAnalyticsOutboxResume(inputJson: string, nowMs: number): Promise<string>;
};

export default requireNativeModule<ExpoOrihaNativeModule>(
  "ExpoOriha",
);
