// Asynchronous render requests with cancellation and generation guards.
import React, { useEffect } from "react";
import { WebView } from "react-native-webview";
import { decidePreviewPatchApplication } from "./orihaPreviewPatchController";
import type {
  BridgeRenderInput,
  OrihaPreviewProps,
  RenderOutput,
  RenderPatchOutput,
  RenderSnapshot,
} from "../types";
import { renderPreviewPatch } from "./previewRendering";
import type { PendingPreviewPatch, OrihaMeasureTextBatch } from "./previewRendering";
import { debugLogIfEnabled, formatDebugError } from "./previewLogs";

export type PreviewRenderingEffectContext = {
  generationRef: React.RefObject<number>;
  latestPreviewInputRef: React.RefObject<BridgeRenderInput | null>;
  clearPendingPatch: () => PendingPreviewPatch | null;
  latestPatchSnapshotRef: React.RefObject<RenderSnapshot | null>;
  latestOutputRef: React.RefObject<RenderOutput | null>;
  setError: (error: unknown) => void;
  setOutput: (output: RenderOutput | null) => void;
  setRenderedInput: (input: BridgeRenderInput | null) => void;
  setRenderedInputKey: (key: string | null) => void;
  webViewRef: React.RefObject<WebView | null>;
  webViewReadyRef: React.RefObject<boolean>;
  setPendingPatch: (patch: PendingPreviewPatch) => void;
  commitFullPatchRender: (
    output: RenderPatchOutput, input: BridgeRenderInput, key: string, css: string,
  ) => void;
  latestViewerFontFaceCssRef: React.RefObject<string>;
  latestOnRenderedRef: React.RefObject<OrihaPreviewProps["onRendered"]>;
  latestOnRenderErrorRef: React.RefObject<OrihaPreviewProps["onRenderError"]>;
  lifecycleDebugEnabled: boolean;
  previewInputKey: string;
  measureTextBatch: OrihaMeasureTextBatch | undefined;
};

export function usePreviewRenderingEffect({
  generationRef,
  latestPreviewInputRef,
  clearPendingPatch,
  latestPatchSnapshotRef,
  latestOutputRef,
  setError,
  setOutput,
  setRenderedInput,
  setRenderedInputKey,
  webViewRef,
  webViewReadyRef,
  setPendingPatch,
  commitFullPatchRender,
  latestViewerFontFaceCssRef,
  latestOnRenderedRef,
  latestOnRenderErrorRef,
  lifecycleDebugEnabled,
  previewInputKey,
  measureTextBatch,
}: PreviewRenderingEffectContext) {
  useEffect(() => {
    let cancelled = false;
    const generationId = generationRef.current + 1;
    generationRef.current = generationId;
    const inputForRender = latestPreviewInputRef.current;
    const stalePendingPatch = clearPendingPatch();
    const hadPendingPatch = stalePendingPatch !== null;
    const previousSnapshot = hadPendingPatch
      ? null
      : latestPatchSnapshotRef.current;

    if (!inputForRender) {
      return undefined;
    }

    debugLogIfEnabled(lifecycleDebugEnabled, "render:start", {
      generationId,
      inputKeyLength: previewInputKey.length,
      markdownLength: inputForRender.markdown.length,
      hadOutput: Boolean(latestOutputRef.current),
      hadPendingPatch,
      hadSnapshot: Boolean(previousSnapshot),
    });
    setError(null);
    if (!latestOutputRef.current) {
      setOutput(null);
      setRenderedInput(null);
      setRenderedInputKey(null);
    }

    renderPreviewPatch({
      input: inputForRender,
      generationId,
      measureTextBatch: measureTextBatch,
      previousSnapshot,
    })
      .then((nextPatchOutput) => {
        if (cancelled) {
          return;
        }
        if (generationRef.current !== generationId) {
          return;
        }

        const patchApplicationDecision = decidePreviewPatchApplication({
          generationId,
          hasWebView: Boolean(webViewRef.current),
          patchOutput: nextPatchOutput,
          previousSnapshot,
          webViewReady: webViewReadyRef.current,
        });

        if (patchApplicationDecision.action === "inject") {
          setPendingPatch({
            generationId,
            input: inputForRender,
            inputKey: previewInputKey,
            patchOutput: nextPatchOutput,
          });
          webViewRef.current?.injectJavaScript(patchApplicationDecision.script);
          debugLogIfEnabled(lifecycleDebugEnabled, "render:patch-injected", {
            generationId,
            patchCount: patchApplicationDecision.patchCount,
          });
          return;
        }

        commitFullPatchRender(
          nextPatchOutput,
          inputForRender,
          previewInputKey,
          latestViewerFontFaceCssRef.current,
        );
        debugLogIfEnabled(lifecycleDebugEnabled, "render:ready", {
          generationId,
          cssLength: nextPatchOutput.output.css?.length ?? 0,
          htmlLength: nextPatchOutput.output.html.length,
          strategy: nextPatchOutput.strategy,
        });
        latestOnRenderedRef.current?.(nextPatchOutput.output);
      })
      .catch((nextError: unknown) => {
        if (cancelled) {
          return;
        }
        if (generationRef.current !== generationId) {
          return;
        }
        setError(nextError);
        if (!latestOutputRef.current) {
          setRenderedInput(null);
          setRenderedInputKey(null);
        }
        debugLogIfEnabled(lifecycleDebugEnabled, "render:error", {
          generationId,
          message: formatDebugError(nextError),
        });
        latestOnRenderErrorRef.current?.(nextError);
      });

    return () => {
      cancelled = true;
      debugLogIfEnabled(lifecycleDebugEnabled, "render:cancel", {
        generationId,
      });
    };
  }, [
    clearPendingPatch,
    commitFullPatchRender,
    generationRef,
    latestOnRenderErrorRef,
    latestOnRenderedRef,
    latestOutputRef,
    latestPatchSnapshotRef,
    latestPreviewInputRef,
    latestViewerFontFaceCssRef,
    lifecycleDebugEnabled,
    measureTextBatch,
    previewInputKey,
    setError,
    setOutput,
    setPendingPatch,
    setRenderedInput,
    setRenderedInputKey,
    webViewReadyRef,
    webViewRef,
  ]);
}
