import Foundation

enum OrihaRustBridge {
  static func versionInfo() -> [String: String] {
    let info = nativeBridgeVersionInfo()
    return [
      "version": info.version,
      "gitShortHash": info.gitShortHash,
    ]
  }

  static func prepareLayoutJson(_ inputJson: String) -> String {
    prepareMarkdownLayoutJson(inputJson: inputJson)
  }

  static func preparePatchJson(_ previousSnapshotJson: String?, _ inputJson: String) -> String {
    prepareMarkdownPatchJson(previousSnapshotJson: previousSnapshotJson, inputJson: inputJson)
  }

  static func prepareExportHtmlJson(_ inputJson: String, _ optionsJson: String?) -> String {
    prepareHtmlExportJson(inputJson: inputJson, optionsJson: optionsJson)
  }

  static func resolveSettingsJson(_ inputJson: String) -> String {
    resolveRenderSettingsJson(inputJson: inputJson)
  }

  static func renderPreparedJson(_ preparedJson: String, _ measureResultsJson: String) -> String {
    renderPreparedMarkdownJson(preparedJson: preparedJson, textMetricsJson: measureResultsJson)
  }

  static func renderPreparedPatchJson(_ preparedJson: String, _ measureResultsJson: String) -> String {
    renderPreparedMarkdownPatchJson(preparedJson: preparedJson, textMetricsJson: measureResultsJson)
  }

  static func renderPreparedExportHtmlJson(_ preparedJson: String, _ measureResultsJson: String) -> String {
    renderPreparedHtmlExportJson(preparedJson: preparedJson, textMetricsJson: measureResultsJson)
  }

  static func startUsageAnalyticsOutboxEnqueueJson(_ inputJson: String, _ nowMs: Double) -> String {
    usageAnalyticsOutboxStartEnqueueJson(inputJson: inputJson, nowMs: analyticsMillis(nowMs))
  }

  static func startUsageAnalyticsOutboxDrainJson(_ inputJson: String, _ nowMs: Double) -> String {
    usageAnalyticsOutboxStartDrainJson(inputJson: inputJson, nowMs: analyticsMillis(nowMs))
  }

  static func startUsageAnalyticsOutboxDeliveryStateChangeJson(_ inputJson: String) -> String {
    usageAnalyticsOutboxStartDeliveryStateChangeJson(inputJson: inputJson)
  }

  static func resumeUsageAnalyticsOutboxJson(_ inputJson: String, _ nowMs: Double) -> String {
    usageAnalyticsOutboxResumeJson(inputJson: inputJson, nowMs: analyticsMillis(nowMs))
  }

  private static func analyticsMillis(_ value: Double) -> UInt64 {
    guard value.isFinite, value > 0 else {
      return 0
    }
    return UInt64(value)
  }
}
