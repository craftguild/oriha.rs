import type {
  RenderPatchOperation,
  RenderPatchOutput,
  RenderSnapshot,
} from "../types";
import type { ViewerPatchResultMessage } from "./orihaPreviewHtml";

export const PATCH_RESULT_TIMEOUT_MS = 1500;
const PATCH_RESULT_MESSAGE_TYPE = "oriha.patchResult";
const INCREMENTAL_PATCH_STRATEGY = "incremental";

export type PreviewPatchApplicationDecision =
  | {
      action: "inject";
      generationId: number;
      patchCount: number;
      script: string;
    }
  | {
      action: "fullReload";
      generationId: number;
      reason:
        | "missingPreviousSnapshot"
        | "replaceAllStrategy"
        | "webViewNotReady"
        | "webViewUnavailable";
    };

export type DecidePreviewPatchApplicationOptions = {
  generationId: number;
  hasWebView: boolean;
  patchOutput: RenderPatchOutput;
  previousSnapshot: RenderSnapshot | null | undefined;
  webViewReady: boolean;
};

export function decidePreviewPatchApplication({
  generationId,
  hasWebView,
  patchOutput,
  previousSnapshot,
  webViewReady,
}: DecidePreviewPatchApplicationOptions): PreviewPatchApplicationDecision {
  if (!previousSnapshot) {
    return {
      action: "fullReload",
      generationId,
      reason: "missingPreviousSnapshot",
    };
  }
  if (patchOutput.strategy !== INCREMENTAL_PATCH_STRATEGY) {
    return {
      action: "fullReload",
      generationId,
      reason: "replaceAllStrategy",
    };
  }
  if (!webViewReady) {
    return {
      action: "fullReload",
      generationId,
      reason: "webViewNotReady",
    };
  }
  if (!hasWebView) {
    return {
      action: "fullReload",
      generationId,
      reason: "webViewUnavailable",
    };
  }

  return {
    action: "inject",
    generationId,
    patchCount: patchOutput.patches.length,
    script: buildPatchInjectionScript({
      generationId,
      patches: patchOutput.patches,
    }),
  };
}

export type PreviewPatchResultDecision =
  | {
      action: "incrementalCommit";
      generationId: number;
    }
  | {
      action: "fullFallback";
      generationId: number;
      message?: string;
    }
  | {
      action: "stale";
      currentGenerationId: number;
      generationId: number;
      pendingGenerationId: number | null;
    };

export type DecidePreviewPatchResultOptions = {
  currentGenerationId: number;
  patchResult: ViewerPatchResultMessage;
  pendingGenerationId: number | null | undefined;
};

export function decidePreviewPatchResult({
  currentGenerationId,
  patchResult,
  pendingGenerationId,
}: DecidePreviewPatchResultOptions): PreviewPatchResultDecision {
  const patchGenerationId = patchResult.generationId;
  const normalizedPendingGenerationId = pendingGenerationId ?? null;
  if (typeof patchGenerationId !== "number") {
    return {
      action: "stale",
      currentGenerationId,
      generationId: -1,
      pendingGenerationId: normalizedPendingGenerationId,
    };
  }
  if (
    normalizedPendingGenerationId !== patchGenerationId ||
    currentGenerationId !== patchGenerationId
  ) {
    return {
      action: "stale",
      currentGenerationId,
      generationId: patchGenerationId,
      pendingGenerationId: normalizedPendingGenerationId,
    };
  }

  if (patchResult.ok) {
    return {
      action: "incrementalCommit",
      generationId: patchGenerationId,
    };
  }

  return {
    action: "fullFallback",
    generationId: patchGenerationId,
    message: patchResult.message,
  };
}

export type BuildPatchInjectionScriptOptions = {
  generationId: number;
  patches: RenderPatchOperation[];
};

export type ShouldRecoverPreviewSourceAfterLoadOptions = {
  hasOutput: boolean;
  webViewSourceStale: boolean;
};

export function shouldRecoverPreviewSourceAfterLoad({
  hasOutput,
  webViewSourceStale,
}: ShouldRecoverPreviewSourceAfterLoadOptions): boolean {
  return webViewSourceStale && hasOutput;
}

export function buildPatchInjectionScript({
  generationId,
  patches,
}: BuildPatchInjectionScriptOptions): string {
  const payload = JSON.stringify({
    generationId,
    patches,
  });
  return `
(function() {
  var payload = ${payload};
  var ok = false;
  var message = null;
  try {
    ok = Boolean(window.orihaApplyPatch && window.orihaApplyPatch(payload));
  } catch (error) {
    ok = false;
    message = error && error.message ? String(error.message) : String(error);
  }
  if (window.ReactNativeWebView && window.ReactNativeWebView.postMessage) {
    window.ReactNativeWebView.postMessage(JSON.stringify({
      type: "${PATCH_RESULT_MESSAGE_TYPE}",
      generationId: payload.generationId,
      ok: ok,
      message: message
    }));
  }
  true;
})();
`;
}
