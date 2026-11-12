// Preview render state, font readiness and full/incremental patch commits.
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { WebView } from "react-native-webview";
import { useOrihaContext } from "../provider/OrihaProvider";
import { defaultOrihaFormatMessage } from "../runtime/expoRuntimeI18n";
import { normalizeBridgeRenderInput } from "../bridge/render";
import { buildInstalledFontFaceCss } from "../fonts/fontRegistry";
import { ColorScheme, Plan } from "../types";
import { normalizeOrihaPreviewDebugOptions } from "./orihaPreviewDebug";
import { buildViewerHtml } from "./orihaPreviewHtml";
import { PATCH_RESULT_TIMEOUT_MS } from "./orihaPreviewPatchController";
import { stableSerialize } from "./stableSerialize";
import type {
  BridgeRenderInput,
  OrihaPreviewProps,
  RenderOutput,
  RenderPatchOutput,
  RenderSnapshot,
} from "../types";
import type { OrihaInstalledFont } from "../fonts/fontRegistry";
import type { PendingPreviewPatch } from "./previewRendering";
import { debugLogIfEnabled, formatDebugError } from "./previewLogs";
import { usePreviewRenderingEffect } from "./previewRenderEffect";

export const EMPTY_INSTALLED_FONTS: readonly OrihaInstalledFont[] = [];

export function usePreviewRenderController({
  markdown, input, debug, onRendered, onRenderError, oriha,
}: Pick<OrihaPreviewProps, "markdown" | "input" | "debug" | "onRendered" | "onRenderError"> & {
  oriha: ReturnType<typeof useOrihaContext>;
}) {
  const generationRef = useRef(0);

  const latestOnRenderedRef = useRef(onRendered);

  const latestOnRenderErrorRef = useRef(onRenderError);

  const latestPreviewInputRef = useRef<BridgeRenderInput | null>(null);

  const latestOutputRef = useRef<RenderOutput | null>(null);

  const latestPatchSnapshotRef = useRef<RenderSnapshot | null>(null);

  const latestViewerFontFaceCssRef = useRef("");

  const pendingPatchRef = useRef<PendingPreviewPatch | null>(null);

  const webViewReadyRef = useRef(false);

  const webViewSourceStaleRef = useRef(false);

  const webViewRef = useRef<WebView>(null);

  const [output, setOutput] = useState<RenderOutput | null>(null);

  const [error, setError] = useState<unknown>(null);

  const [webViewHtml, setWebViewHtml] = useState("");

  const [renderedInput, setRenderedInput] = useState<BridgeRenderInput | null>(
    null,
  );

  const [renderedInputKey, setRenderedInputKey] = useState<string | null>(null);

  const [viewerFontFaceCss, setViewerFontFaceCss] = useState("");

  const [viewerFontFaceCssReady, setViewerFontFaceCssReady] = useState(false);

  const [viewerFontFaceCssError, setViewerFontFaceCssError] =
    useState<unknown>(null);

  const formatMessage =
    oriha?.formatMessage ?? defaultOrihaFormatMessage;

  const debugOptions = normalizeOrihaPreviewDebugOptions(debug, formatMessage);

  const lifecycleDebugEnabled =
    debug === true || (typeof debug === "object" && debug !== null);

  const installedFonts =
    oriha?.fontRegistry.installedFonts ?? EMPTY_INSTALLED_FONTS;

  const latestInstalledFontsRef = useRef(installedFonts);

  const previewInput = useMemo<BridgeRenderInput>(
    () =>
      normalizeBridgeRenderInput({
        plan: Plan.Free,
        locale: oriha?.i18n.locale,
        ...input,
        renderingOptions: {
          colorScheme: ColorScheme.Light,
          ...(input?.renderingOptions ?? {}),
        },
        markdown,
      }),
    [input, markdown, oriha?.i18n.locale],
  );

  const previewInputKey = useMemo(
    () => stableSerialize(previewInput),
    [previewInput],
  );

  const installedFontsKey = useMemo(
    () => stableSerialize(installedFonts),
    [installedFonts],
  );

  latestPreviewInputRef.current = previewInput;

  latestOnRenderedRef.current = onRendered;

  latestOnRenderErrorRef.current = onRenderError;

  latestViewerFontFaceCssRef.current = viewerFontFaceCss;

  const selectedPreviewFontSpecs = useMemo(
    () =>
      [
        previewInput.renderingOptions?.customHeadingFont,
        previewInput.renderingOptions?.customBodyFont,
        previewInput.renderingOptions?.customCodeFont,
      ] as const,
    [
      previewInput.renderingOptions?.customBodyFont,
      previewInput.renderingOptions?.customCodeFont,
      previewInput.renderingOptions?.customHeadingFont,
    ],
  );

  const latestSelectedPreviewFontSpecsRef = useRef(selectedPreviewFontSpecs);

  latestInstalledFontsRef.current = installedFonts;

  latestOutputRef.current = output;

  latestSelectedPreviewFontSpecsRef.current = selectedPreviewFontSpecs;

  const selectedPreviewFontSpecsKey = useMemo(
    () => stableSerialize(selectedPreviewFontSpecs),
    [selectedPreviewFontSpecs],
  );

  const clearPendingPatch = useCallback(function clearPendingPatch(): PendingPreviewPatch | null {
    const pendingPatch = pendingPatchRef.current;
    if (pendingPatch?.timeoutId) {
      clearTimeout(pendingPatch.timeoutId);
    }
    pendingPatchRef.current = null;
    return pendingPatch;
  }, []);

  const commitFullPatchRender = useCallback(function commitFullPatchRender(
    patchOutput: RenderPatchOutput,
    nextInput: BridgeRenderInput,
    nextInputKey: string,
    fontFaceCss: string,
  ) {
    const nextOutput = patchOutput.output;
    const nextHtml = buildViewerHtml(nextOutput, fontFaceCss);
    latestOutputRef.current = nextOutput;
    latestPatchSnapshotRef.current = patchOutput.snapshot;
    webViewSourceStaleRef.current = false;
    clearPendingPatch();
    webViewReadyRef.current = false;
    setWebViewHtml(nextHtml);
    setOutput(nextOutput);
    setError(null);
    setRenderedInput(nextInput);
    setRenderedInputKey(nextInputKey);
  }, [clearPendingPatch]);

  const commitIncrementalPatchRender = useCallback(function commitIncrementalPatchRender(
    patchOutput: RenderPatchOutput,
    nextInput: BridgeRenderInput,
    nextInputKey: string,
  ) {
    const nextOutput = patchOutput.output;
    latestOutputRef.current = nextOutput;
    latestPatchSnapshotRef.current = patchOutput.snapshot;
    webViewSourceStaleRef.current = true;
    clearPendingPatch();
    setOutput(nextOutput);
    setError(null);
    setRenderedInput(nextInput);
    setRenderedInputKey(nextInputKey);
  }, [clearPendingPatch]);

  const setPendingPatch = useCallback(function setPendingPatch(pendingPatch: PendingPreviewPatch) {
    pendingPatch.timeoutId = setTimeout(() => {
      if (
        pendingPatchRef.current !== pendingPatch ||
        generationRef.current !== pendingPatch.generationId
      ) {
        return;
      }
      commitFullPatchRender(
        pendingPatch.patchOutput,
        pendingPatch.input,
        pendingPatch.inputKey,
        latestViewerFontFaceCssRef.current,
      );
      debugLogIfEnabled(lifecycleDebugEnabled, "render:patch-timeout", {
        generationId: pendingPatch.generationId,
      });
      latestOnRenderedRef.current?.(pendingPatch.patchOutput.output);
    }, PATCH_RESULT_TIMEOUT_MS);
    pendingPatchRef.current = pendingPatch;
  }, [commitFullPatchRender, lifecycleDebugEnabled]);

  useEffect(
    () => () => {
      clearPendingPatch();
    },
    [clearPendingPatch],
  );

  useEffect(() => {
    let cancelled = false;
    const activeInstalledFonts = latestInstalledFontsRef.current;
    const activeSelectedPreviewFontSpecs =
      latestSelectedPreviewFontSpecsRef.current;
    setViewerFontFaceCssReady(false);
    setViewerFontFaceCssError(null);

    buildInstalledFontFaceCss(
      activeInstalledFonts,
      activeSelectedPreviewFontSpecs,
    )
      .then((nextFontFaceCss) => {
        if (cancelled) {
          return;
        }
        setViewerFontFaceCss(nextFontFaceCss);
        setViewerFontFaceCssReady(true);
      })
      .catch((nextError: unknown) => {
        if (cancelled) {
          return;
        }
        setViewerFontFaceCss("");
        setViewerFontFaceCssError(nextError);
        setViewerFontFaceCssReady(true);
        debugLogIfEnabled(lifecycleDebugEnabled, "render:font-css-error", {
          message: formatDebugError(nextError),
        });
        latestOnRenderErrorRef.current?.(nextError);
      });

    return () => {
      cancelled = true;
    };
  }, [
    installedFontsKey,
    lifecycleDebugEnabled,
    selectedPreviewFontSpecsKey,
  ]);

  usePreviewRenderingEffect({
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
    measureTextBatch: oriha?.measureTextBatch,
  });

  useEffect(() => {
    if (!viewerFontFaceCssReady || !latestOutputRef.current) {
      return;
    }
    const pendingPatch = pendingPatchRef.current;
    if (pendingPatch) {
      commitFullPatchRender(
        pendingPatch.patchOutput,
        pendingPatch.input,
        pendingPatch.inputKey,
        viewerFontFaceCss,
      );
      latestOnRenderedRef.current?.(pendingPatch.patchOutput.output);
      return;
    }
    const nextHtml = buildViewerHtml(
      latestOutputRef.current,
      viewerFontFaceCss,
    );
    clearPendingPatch();
    webViewSourceStaleRef.current = false;
    webViewReadyRef.current = false;
    setWebViewHtml(nextHtml);
  }, [viewerFontFaceCss, viewerFontFaceCssReady, commitFullPatchRender, clearPendingPatch]);
  return {
    generationRef,
    latestOnRenderedRef,
    latestOnRenderErrorRef,
    latestOutputRef,
    latestViewerFontFaceCssRef,
    pendingPatchRef,
    webViewReadyRef,
    webViewSourceStaleRef,
    webViewRef,
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
    commitFullPatchRender,
    commitIncrementalPatchRender,
    clearPendingPatch,
    setWebViewHtml,
  };
}
