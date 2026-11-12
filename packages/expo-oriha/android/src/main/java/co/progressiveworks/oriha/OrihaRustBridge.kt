package co.progressiveworks.oriha

import java.util.concurrent.atomic.AtomicBoolean
import uniffi.oriha_native_bridge.nativeBridgeVersionInfo as bridgeNativeBridgeVersionInfo
import uniffi.oriha_native_bridge.prepareHtmlExportJson as bridgePrepareHtmlExportJson
import uniffi.oriha_native_bridge.prepareMarkdownLayoutJson as bridgePrepareMarkdownLayoutJson
import uniffi.oriha_native_bridge.prepareMarkdownPatchJson as bridgePrepareMarkdownPatchJson
import uniffi.oriha_native_bridge.renderPreparedHtmlExportJson as bridgeRenderPreparedHtmlExportJson
import uniffi.oriha_native_bridge.renderPreparedMarkdownJson as bridgeRenderPreparedMarkdownJson
import uniffi.oriha_native_bridge.renderPreparedMarkdownPatchJson as bridgeRenderPreparedMarkdownPatchJson
import uniffi.oriha_native_bridge.resolveRenderSettingsJson as bridgeResolveRenderSettingsJson
import uniffi.oriha_native_bridge.usageAnalyticsOutboxResumeJson as bridgeUsageAnalyticsOutboxResumeJson
import uniffi.oriha_native_bridge.usageAnalyticsOutboxStartDeliveryStateChangeJson as bridgeUsageAnalyticsOutboxStartDeliveryStateChangeJson
import uniffi.oriha_native_bridge.usageAnalyticsOutboxStartDrainJson as bridgeUsageAnalyticsOutboxStartDrainJson
import uniffi.oriha_native_bridge.usageAnalyticsOutboxStartEnqueueJson as bridgeUsageAnalyticsOutboxStartEnqueueJson

object OrihaRustBridge {
  private val nativeLoaderConfigured = AtomicBoolean(false)

  fun configureNativeLibraryLoader() {
    if (!nativeLoaderConfigured.compareAndSet(false, true)) {
      return
    }

    // Keep JNA on its normal Android native loader path. The 16 KB page-size
    // fix comes from the JNA AAR version, not from custom library extraction.
    System.setProperty("jna.nosys", "false")
  }

  fun versionInfo(): Map<String, String> {
    val info = bridgeNativeBridgeVersionInfo()
    return mapOf(
      "version" to info.version,
      "gitShortHash" to info.gitShortHash,
    )
  }

  fun prepareLayoutJson(inputJson: String): String =
    bridgePrepareMarkdownLayoutJson(inputJson)

  fun preparePatchJson(previousSnapshotJson: String?, inputJson: String): String =
    bridgePrepareMarkdownPatchJson(previousSnapshotJson, inputJson)

  fun prepareHtmlExportJson(inputJson: String, optionsJson: String?): String =
    bridgePrepareHtmlExportJson(inputJson, optionsJson)

  fun resolveSettingsJson(inputJson: String): String =
    bridgeResolveRenderSettingsJson(inputJson)

  fun renderPreparedJson(preparedJson: String, measureResultsJson: String): String =
    bridgeRenderPreparedMarkdownJson(preparedJson, measureResultsJson)

  fun renderPreparedPatchJson(preparedJson: String, measureResultsJson: String): String =
    bridgeRenderPreparedMarkdownPatchJson(preparedJson, measureResultsJson)

  fun renderPreparedHtmlExportJson(preparedJson: String, measureResultsJson: String): String =
    bridgeRenderPreparedHtmlExportJson(preparedJson, measureResultsJson)

  fun usageAnalyticsOutboxStartEnqueueJson(inputJson: String, nowMs: Double): String =
    bridgeUsageAnalyticsOutboxStartEnqueueJson(inputJson, nowMs.toAnalyticsMillis())

  fun usageAnalyticsOutboxStartDrainJson(inputJson: String, nowMs: Double): String =
    bridgeUsageAnalyticsOutboxStartDrainJson(inputJson, nowMs.toAnalyticsMillis())

  fun usageAnalyticsOutboxStartDeliveryStateChangeJson(inputJson: String): String =
    bridgeUsageAnalyticsOutboxStartDeliveryStateChangeJson(inputJson)

  fun usageAnalyticsOutboxResumeJson(inputJson: String, nowMs: Double): String =
    bridgeUsageAnalyticsOutboxResumeJson(inputJson, nowMs.toAnalyticsMillis())

  private fun Double.toAnalyticsMillis(): ULong =
    if (isFinite() && this > 0.0) toULong() else 0UL
}
