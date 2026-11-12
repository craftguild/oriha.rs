import ExpoModulesCore

public class ExpoOrihaModule: Module {
  public func definition() -> ModuleDefinition {
    Name("ExpoOriha")

    AsyncFunction("nativeBridgeVersionInfo") { () -> [String: String] in
      OrihaRustBridge.versionInfo()
    }

    AsyncFunction("prepareMarkdownLayout") { (inputJson: String) -> String in
      OrihaRustBridge.prepareLayoutJson(inputJson)
    }

    AsyncFunction("prepareMarkdownPatch") { (previousSnapshotJson: String?, inputJson: String) -> String in
      OrihaRustBridge.preparePatchJson(previousSnapshotJson, inputJson)
    }

    AsyncFunction("prepareHtmlExport") { (inputJson: String, optionsJson: String?) -> String in
      OrihaRustBridge.prepareExportHtmlJson(inputJson, optionsJson)
    }

    AsyncFunction("resolveRenderSettings") { (inputJson: String) -> String in
      OrihaRustBridge.resolveSettingsJson(inputJson)
    }

    AsyncFunction("renderPreparedMarkdown") { (preparedJson: String, measureResultsJson: String) -> String in
      OrihaRustBridge.renderPreparedJson(preparedJson, measureResultsJson)
    }

    AsyncFunction("renderPreparedMarkdownPatch") { (preparedJson: String, measureResultsJson: String) -> String in
      OrihaRustBridge.renderPreparedPatchJson(preparedJson, measureResultsJson)
    }

    AsyncFunction("renderPreparedHtmlExport") { (preparedJson: String, measureResultsJson: String) -> String in
      OrihaRustBridge.renderPreparedExportHtmlJson(preparedJson, measureResultsJson)
    }

    AsyncFunction("usageAnalyticsOutboxStartEnqueue") { (inputJson: String, nowMs: Double) -> String in
      OrihaRustBridge.startUsageAnalyticsOutboxEnqueueJson(inputJson, nowMs)
    }

    AsyncFunction("usageAnalyticsOutboxStartDrain") { (inputJson: String, nowMs: Double) -> String in
      OrihaRustBridge.startUsageAnalyticsOutboxDrainJson(inputJson, nowMs)
    }

    AsyncFunction("usageAnalyticsOutboxStartDeliveryStateChange") { (inputJson: String) -> String in
      OrihaRustBridge.startUsageAnalyticsOutboxDeliveryStateChangeJson(inputJson)
    }

    AsyncFunction("usageAnalyticsOutboxResume") { (inputJson: String, nowMs: Double) -> String in
      OrihaRustBridge.resumeUsageAnalyticsOutboxJson(inputJson, nowMs)
    }
  }
}
