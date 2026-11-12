package co.progressiveworks.oriha

import expo.modules.kotlin.modules.Module
import expo.modules.kotlin.modules.ModuleDefinition

class ExpoOrihaModule : Module() {
  override fun definition() = ModuleDefinition {
    Name("ExpoOriha")

    OnCreate {
      OrihaRustBridge.configureNativeLibraryLoader()
    }

    AsyncFunction("nativeBridgeVersionInfo") {
      OrihaRustBridge.versionInfo()
    }

    AsyncFunction("prepareMarkdownLayout") { inputJson: String ->
      OrihaRustBridge.prepareLayoutJson(inputJson)
    }

    AsyncFunction("prepareMarkdownPatch") { previousSnapshotJson: String?, inputJson: String ->
      OrihaRustBridge.preparePatchJson(previousSnapshotJson, inputJson)
    }

    AsyncFunction("prepareHtmlExport") { inputJson: String, optionsJson: String? ->
      OrihaRustBridge.prepareHtmlExportJson(inputJson, optionsJson)
    }

    AsyncFunction("resolveRenderSettings") { inputJson: String ->
      OrihaRustBridge.resolveSettingsJson(inputJson)
    }

    AsyncFunction("renderPreparedMarkdown") { preparedJson: String, measureResultsJson: String ->
      OrihaRustBridge.renderPreparedJson(preparedJson, measureResultsJson)
    }

    AsyncFunction("renderPreparedMarkdownPatch") { preparedJson: String, measureResultsJson: String ->
      OrihaRustBridge.renderPreparedPatchJson(preparedJson, measureResultsJson)
    }

    AsyncFunction("renderPreparedHtmlExport") { preparedJson: String, measureResultsJson: String ->
      OrihaRustBridge.renderPreparedHtmlExportJson(preparedJson, measureResultsJson)
    }

    AsyncFunction("usageAnalyticsOutboxStartEnqueue") { inputJson: String, nowMs: Double ->
      OrihaRustBridge.usageAnalyticsOutboxStartEnqueueJson(inputJson, nowMs)
    }

    AsyncFunction("usageAnalyticsOutboxStartDrain") { inputJson: String, nowMs: Double ->
      OrihaRustBridge.usageAnalyticsOutboxStartDrainJson(inputJson, nowMs)
    }

    AsyncFunction("usageAnalyticsOutboxStartDeliveryStateChange") { inputJson: String ->
      OrihaRustBridge.usageAnalyticsOutboxStartDeliveryStateChangeJson(inputJson)
    }

    AsyncFunction("usageAnalyticsOutboxResume") { inputJson: String, nowMs: Double ->
      OrihaRustBridge.usageAnalyticsOutboxResumeJson(inputJson, nowMs)
    }
  }
}
