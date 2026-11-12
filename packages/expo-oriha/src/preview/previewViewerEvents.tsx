// WebView patch acknowledgements, recovery, messages and navigation events.
import { useMemo } from "react";
import type { WebViewMessageEvent, WebViewProps } from "react-native-webview";
import { useOrihaContext } from "../provider/OrihaProvider";
import { buildViewerHtml, parseViewerPatchResultMessage, shouldOpenExternally } from "./orihaPreviewHtml";
import { decidePreviewPatchResult, shouldRecoverPreviewSourceAfterLoad } from "./orihaPreviewPatchController";
import type { OrihaPreviewProps } from "../types";
import { usePreviewRenderController } from "./previewRenderController";
import { debugLogIfEnabled } from "./previewLogs";
import { handleExternalLinkMessage, handleCodeCopyMessage, openExternalUrl } from "./previewLinks";

export function usePreviewViewerEvents({ state, webViewProps, oriha }: {
  state: ReturnType<typeof usePreviewRenderController>;
  webViewProps: OrihaPreviewProps["webViewProps"];
  oriha: ReturnType<typeof useOrihaContext>;
}) {
  const {
    pendingPatchRef,
    generationRef,
    commitIncrementalPatchRender,
    commitFullPatchRender,
    latestViewerFontFaceCssRef,
    latestOnRenderedRef,
    webViewReadyRef,
    latestOutputRef,
    webViewSourceStaleRef,
    setWebViewHtml,
    formatMessage,
    lifecycleDebugEnabled,
  } = state;
  const handleViewerMessage = useMemo(() => {
    const onMessage = webViewProps?.onMessage;
    return (event: WebViewMessageEvent) => {
      const patchResult = parseViewerPatchResultMessage(event.nativeEvent.data);
      if (patchResult) {
        const pendingPatch = pendingPatchRef.current;
        const patchResultDecision = decidePreviewPatchResult({
          currentGenerationId: generationRef.current,
          patchResult,
          pendingGenerationId: pendingPatch?.generationId,
        });
        if (pendingPatch) {
          if (patchResultDecision.action === "incrementalCommit") {
            commitIncrementalPatchRender(
              pendingPatch.patchOutput,
              pendingPatch.input,
              pendingPatch.inputKey,
            );
            debugLogIfEnabled(lifecycleDebugEnabled, "render:patch-ready", {
              generationId: pendingPatch.generationId,
              patchCount: pendingPatch.patchOutput.patches.length,
            });
          } else if (patchResultDecision.action === "fullFallback") {
            commitFullPatchRender(
              pendingPatch.patchOutput,
              pendingPatch.input,
              pendingPatch.inputKey,
              latestViewerFontFaceCssRef.current,
            );
            debugLogIfEnabled(lifecycleDebugEnabled, "render:patch-fallback", {
              generationId: pendingPatch.generationId,
              message: patchResultDecision.message ?? null,
            });
          } else {
            return;
          }
          latestOnRenderedRef.current?.(pendingPatch.patchOutput.output);
        }
        return;
      }
      handleExternalLinkMessage(
        event.nativeEvent.data,
        oriha?.showToast,
        formatMessage,
      );
      handleCodeCopyMessage(
        event.nativeEvent.data,
        oriha?.showToast,
        formatMessage,
      );
      onMessage?.(event);
    };
  }, [
    commitFullPatchRender,
    commitIncrementalPatchRender,
    formatMessage,
    generationRef,
    latestOnRenderedRef,
    latestViewerFontFaceCssRef,
    lifecycleDebugEnabled,
    oriha?.showToast,
    pendingPatchRef,
    webViewProps?.onMessage,
  ]);

  const handleViewerLoadStart = useMemo(() => {
    const onLoadStart = webViewProps?.onLoadStart;
    return (
      event: Parameters<NonNullable<WebViewProps["onLoadStart"]>>[0],
    ) => {
      webViewReadyRef.current = false;
      onLoadStart?.(event);
    };
  }, [webViewReadyRef, webViewProps?.onLoadStart]);

  const handleViewerLoadEnd = useMemo(() => {
    const onLoadEnd = webViewProps?.onLoadEnd;
    return (
      event: Parameters<NonNullable<WebViewProps["onLoadEnd"]>>[0],
    ) => {
      webViewReadyRef.current = true;
      const latestOutput = latestOutputRef.current;
      if (
        latestOutput &&
        shouldRecoverPreviewSourceAfterLoad({
          hasOutput: true,
          webViewSourceStale: webViewSourceStaleRef.current,
        })
      ) {
        webViewSourceStaleRef.current = false;
        webViewReadyRef.current = false;
        setWebViewHtml(
          buildViewerHtml(
            latestOutput,
            latestViewerFontFaceCssRef.current,
          ),
        );
      }
      onLoadEnd?.(event);
    };
  }, [
    latestOutputRef,
    latestViewerFontFaceCssRef,
    setWebViewHtml,
    webViewReadyRef,
    webViewSourceStaleRef,
    webViewProps?.onLoadEnd,
  ]);

  const handleShouldStartLoadWithRequest = useMemo(() => {
    const onShouldStartLoadWithRequest =
      webViewProps?.onShouldStartLoadWithRequest;
    return (
      request: Parameters<
        NonNullable<WebViewProps["onShouldStartLoadWithRequest"]>
      >[0],
    ) => {
      if (shouldOpenExternally(request.url)) {
        openExternalUrl(request.url, oriha?.showToast, formatMessage);
        return false;
      }
      return onShouldStartLoadWithRequest?.(request) ?? true;
    };
  }, [
    formatMessage,
    oriha?.showToast,
    webViewProps?.onShouldStartLoadWithRequest,
  ]);
  return {
    handleViewerMessage,
    handleViewerLoadStart,
    handleViewerLoadEnd,
    handleShouldStartLoadWithRequest,
  };
}
