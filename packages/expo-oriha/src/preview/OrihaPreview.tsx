import React, { useEffect, useImperativeHandle, useMemo, useRef } from "react";
import * as Sharing from "expo-sharing";
import { ActivityIndicator, Pressable, Text, View } from "react-native";
import { WebView } from "react-native-webview";
import { OrihaToastPosition, OrihaToastVariant } from "../provider/OrihaEnums";
import { useOrihaContext } from "../provider/OrihaProvider";
import { buildViewerHtml } from "./orihaPreviewHtml";
import type { OrihaPdfExportOptions, OrihaPreviewHandle, OrihaPreviewProps, RenderOutput } from "../types";
import { exportPreviewSnapshotToPdf } from "./previewPdfExport";
import type { OrihaPreviewExportSnapshot } from "./previewPdfExport";
import { usePreviewRenderController } from "./previewRenderController";
import { debugLogIfEnabled, formatError, debugLog } from "./previewLogs";
import { usePreviewViewerEvents } from "./previewViewerEvents";
import { saveRenderedHtmlDebugFile } from "./previewDebugExport";
import { styles } from "./previewStyles";

export const OrihaPreview = React.forwardRef<
  OrihaPreviewHandle,
  OrihaPreviewProps
>(function OrihaPreview(
  {
    markdown,
    input,
    loadingFallback,
    debug,
    onRendered,
    onRenderError,
    webViewProps,
  },
  ref,
) {
  const oriha = useOrihaContext();

  const pdfGenerationRef = useRef(0);

  const exportSnapshotRef = useRef<OrihaPreviewExportSnapshot | null>(null);

  const renderState = usePreviewRenderController({ markdown, input, debug, onRendered, onRenderError, oriha });
  const {
    output,
    error,
    webViewHtml,
    renderedInput,
    renderedInputKey,
    viewerFontFaceCss,
    viewerFontFaceCssReady,
    viewerFontFaceCssError,
    formatMessage,
    debugOptions,
    lifecycleDebugEnabled,
    installedFonts,
    previewInput,
    previewInputKey,
    webViewRef,
  } = renderState;

  const { style: webViewStyle, ...webViewRestProps } = webViewProps ?? {};

  useEffect(() => {
    exportSnapshotRef.current = {
      input: renderedInput ?? previewInput,
      installedFonts: [...installedFonts],
      isReady:
        Boolean(output) &&
        !error &&
        !viewerFontFaceCssError &&
        viewerFontFaceCssReady &&
        renderedInputKey === previewInputKey,
      measureTextBatch: oriha?.measureTextBatch,
    };
  }, [
    error,
    installedFonts,
    oriha?.measureTextBatch,
    output,
    previewInput,
    previewInputKey,
    renderedInput,
    renderedInputKey,
    viewerFontFaceCssError,
    viewerFontFaceCssReady,
  ]);

  useImperativeHandle(
    ref,
    () => ({
      exportPdf: async (options?: OrihaPdfExportOptions) => {
        const snapshot = exportSnapshotRef.current;
        debugLogIfEnabled(lifecycleDebugEnabled, "exportPdf:requested", {
          hasSnapshot: Boolean(snapshot),
          isReady: snapshot?.isReady ?? false,
          options,
        });
        if (!snapshot?.isReady) {
          throw new Error("Markdown preview is not ready for PDF export");
        }
        pdfGenerationRef.current += 1;
        const generationId = -pdfGenerationRef.current;
        const result = await exportPreviewSnapshotToPdf({
          generationId: -pdfGenerationRef.current,
          options,
          snapshot,
        });
        debugLogIfEnabled(lifecycleDebugEnabled, "exportPdf:ready", {
          byteLength: result.byteLength,
          generationId,
          pageCount: result.pageCount ?? null,
        });
        return result;
      },
    }),
    [lifecycleDebugEnabled],
  );

  const {
    handleViewerMessage,
    handleViewerLoadStart,
    handleViewerLoadEnd,
    handleShouldStartLoadWithRequest,
  } = usePreviewViewerEvents({ state: renderState, webViewProps, oriha });

  const shareRenderedHtml = async () => {
    if (!debugViewerHtml) {
      return;
    }
    try {
      const fileUri = await saveRenderedHtmlDebugFile(
        debugViewerHtml,
        debugOptions.renderedHtmlFileName,
        formatMessage("debug.fileSystemDirectoryUnavailable"),
      );
      if (await Sharing.isAvailableAsync()) {
        await Sharing.shareAsync(fileUri, {
          dialogTitle: formatMessage("debug.shareDialogTitle"),
          mimeType: "text/html",
          UTI: "public.html",
        });
      } else {
        oriha?.showToast?.(
          formatMessage("debug.renderedHtmlSaved", { fileUri }),
          {
            position: OrihaToastPosition.Top,
            variant: OrihaToastVariant.Success,
          },
        );
      }
    } catch (nextError: unknown) {
      const message = formatMessage("debug.shareFailed", {
        message: formatError(nextError),
      });
      oriha?.showToast?.(message, {
        position: OrihaToastPosition.Top,
        variant: OrihaToastVariant.Err,
      });
    }
  };

  const previewError = error ?? viewerFontFaceCssError;

  const debugViewerHtml = useMemo(
    () => (output ? buildViewerHtml(output, viewerFontFaceCss) : webViewHtml),
    [output, viewerFontFaceCss, webViewHtml],
  );

  const webViewSource = useMemo(() => ({ html: webViewHtml }), [webViewHtml]);

  const showPreviewLoading = shouldShowPreviewLoading(output, previewError);

  useEffect(() => {
    if (!lifecycleDebugEnabled || (!showPreviewLoading && !previewError)) {
      return;
    }

    debugLog("view-state", {
      hasOutput: Boolean(output),
      hasPreviewError: Boolean(previewError),
      renderedInputMatches: renderedInputKey === previewInputKey,
      showPreviewLoading,
      viewerHtmlLength: debugViewerHtml.length,
    });
  }, [
    debugViewerHtml.length,
    lifecycleDebugEnabled,
    output,
    previewError,
    previewInputKey,
    renderedInputKey,
    showPreviewLoading,
  ]);

  if (showPreviewLoading) {
    return (
      <View style={[styles.previewContainer, webViewStyle]}>
        {loadingFallback ?? <ActivityIndicator />}
      </View>
    );
  }

  if (!output || previewError) {
    return <View style={[styles.previewContainer, webViewStyle]} />;
  }

  return (
    <View style={[styles.previewContainer, webViewStyle]}>
      <WebView
        {...webViewRestProps}
        ref={webViewRef}
        allowFileAccess={webViewProps?.allowFileAccess ?? true}
        originWhitelist={["*"]}
        onLoadEnd={handleViewerLoadEnd}
        onLoadStart={handleViewerLoadStart}
        onMessage={handleViewerMessage}
        onShouldStartLoadWithRequest={handleShouldStartLoadWithRequest}
        source={webViewSource}
        style={styles.webView}
      />
      {debugOptions.enabled ? (
        <View pointerEvents="box-none" style={styles.debugOverlay}>
          <Pressable
            accessibilityRole="button"
            onPress={shareRenderedHtml}
            style={({ pressed }) => [
              styles.debugButton,
              pressed ? styles.debugButtonPressed : null,
            ]}
          >
            <Text style={styles.debugButtonText}>
              {debugOptions.shareRenderedHtmlLabel}
            </Text>
          </Pressable>
        </View>
      ) : null}
    </View>
  );
});

OrihaPreview.displayName = "OrihaPreview";

export function shouldShowPreviewLoading(
  output: RenderOutput | null,
  previewError: unknown,
): boolean {
  return !output && !previewError;
}
