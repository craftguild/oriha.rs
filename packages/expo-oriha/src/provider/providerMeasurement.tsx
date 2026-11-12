// Measurement batch lifecycle, font readiness, generation guards and cancellation.
import React, { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { WebView } from "react-native-webview";
import { buildInstalledFontFaceCss } from "../fonts/fontRegistry";
import type { TextMeasurementRequest, TextMeasurementResult } from "../types";
import type { OrihaInstalledFont } from "../fonts/fontRegistry";
import type { OrihaFormatMessage } from "../runtime/expoRuntimeI18n";
import { buildMeasurementWebViewHtml, buildMeasureBatchScript, MESSAGE_TYPE_BATCH_RESULT } from "./measurementDocument";
import type { MeasureBatchResultMessage } from "./measurementDocument";
import type { MeasureTextBatchOptions } from "./providerContext";

export type PendingMeasureBatch = {
  id: string;
  generationId: number;
  requests: TextMeasurementRequest[];
  injected: boolean;
  resolve: (results: TextMeasurementResult[]) => void;
  reject: (error: Error) => void;
  timer: ReturnType<typeof setTimeout>;
};

export type WebViewRef = React.ElementRef<typeof WebView>;

export function logMeasurementDebug(kind: string, payload: unknown) {
  console.log(`[OrihaEngine] measureTextBatch ${kind}`, payload);
}

export function useTextMeasurementBridge({
  installedFonts,
  measurementTimeoutMs,
  debugMeasurementLogs,
  formatMessage,
  setFontRegistryError,
}: {
  installedFonts: OrihaInstalledFont[];
  measurementTimeoutMs: number;
  debugMeasurementLogs: boolean;
  formatMessage: OrihaFormatMessage;
  setFontRegistryError: (error: unknown) => void;
}) {
  const webViewRef = useRef<WebViewRef>(null);

  const pendingBatches = useRef(new Map<string, PendingMeasureBatch>());

  const nextBatchId = useRef(0);

  const providerGeneration = useRef(0);

  const installedFontFaceCssRef = useRef("");

  const measurementFontFaceCssLoading = useRef(false);

  const measurementWebViewLoaded = useRef(false);

  const measurementWebViewReady = useRef(false);

  const [installedFontFaceCss, setInstalledFontFaceCss] = useState("");

  const formatMessageRef = useRef<OrihaFormatMessage>(
    formatMessage,
  );

  useEffect(() => {
    formatMessageRef.current = formatMessage;
  }, [formatMessage]);

  const measurementWebViewHtml = useMemo(
    () => buildMeasurementWebViewHtml(installedFontFaceCss),
    [installedFontFaceCss],
  );

  const injectBatch = useCallback((batch: PendingMeasureBatch) => {
    const webView = webViewRef.current;
    if (!webView) {
      return;
    }

    batch.injected = true;
    webView.injectJavaScript(
      buildMeasureBatchScript({
        id: batch.id,
        generationId: batch.generationId,
        requests: batch.requests,
      }),
    );
  }, []);

  const flushPendingBatches = useCallback(() => {
    for (const batch of pendingBatches.current.values()) {
      if (!batch.injected) {
        injectBatch(batch);
      }
    }
  }, [injectBatch]);

  useEffect(() => {
    installedFontFaceCssRef.current = installedFontFaceCss;
  }, [installedFontFaceCss]);

  useEffect(() => {
    let cancelled = false;
    measurementFontFaceCssLoading.current = true;
    measurementWebViewReady.current = false;

    const complete = (nextFontFaceCss: string) => {
      measurementFontFaceCssLoading.current = false;
      if (nextFontFaceCss === installedFontFaceCssRef.current) {
        if (measurementWebViewLoaded.current) {
          measurementWebViewReady.current = true;
          flushPendingBatches();
        }
        return;
      }
      measurementWebViewReady.current = false;
      setInstalledFontFaceCss(nextFontFaceCss);
    };

    buildInstalledFontFaceCss(installedFonts)
      .then((nextFontFaceCss) => {
        if (!cancelled) {
          complete(nextFontFaceCss);
        }
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          setFontRegistryError(error);
          complete("");
        }
      });

    return () => {
      cancelled = true;
    };
  }, [flushPendingBatches, installedFonts, setFontRegistryError]);

  const measureTextBatch = useCallback(
    (
      requests: TextMeasurementRequest[],
      options?: MeasureTextBatchOptions,
    ): Promise<TextMeasurementResult[]> => {
      if (requests.length === 0) {
        return Promise.resolve([]);
      }

      const id = `measure-${Date.now()}-${nextBatchId.current++}`;
      const generationId = options?.generationId ?? providerGeneration.current++;
      const timeoutMs = options?.timeoutMs ?? measurementTimeoutMs;

      if (debugMeasurementLogs) {
        logMeasurementDebug("request", {
          id,
          generationId,
          requestCount: requests.length,
          requests,
        });
      }

      return new Promise((resolve, reject) => {
        const timer = setTimeout(() => {
          pendingBatches.current.delete(id);
          if (debugMeasurementLogs) {
            logMeasurementDebug("timeout", {
              id,
              generationId,
              timeoutMs,
              requestCount: requests.length,
            });
          }
          reject(
            new Error(
              formatMessageRef.current("measurement.timeout", { timeoutMs }),
            ),
          );
        }, timeoutMs);

        const batch: PendingMeasureBatch = {
          id,
          generationId,
          requests,
          injected: false,
          resolve,
          reject,
          timer,
        };

        pendingBatches.current.set(id, batch);

        if (measurementWebViewReady.current) {
          injectBatch(batch);
        }
      });
    },
    [debugMeasurementLogs, injectBatch, measurementTimeoutMs],
  );

  const handleMessage = useCallback(
    (event: { nativeEvent: { data: string } }) => {
      let message: MeasureBatchResultMessage;
      try {
        message = JSON.parse(event.nativeEvent.data) as MeasureBatchResultMessage;
      } catch {
        return;
      }

      if (message.type !== MESSAGE_TYPE_BATCH_RESULT) {
        return;
      }

      const batch = pendingBatches.current.get(message.id);
      if (!batch || batch.generationId !== message.generationId) {
        return;
      }

      clearTimeout(batch.timer);
      pendingBatches.current.delete(message.id);

      if (debugMeasurementLogs) {
        logMeasurementDebug("result", {
          id: message.id,
          generationId: message.generationId,
          resultCount: message.results.length,
          results: message.results,
          error: message.error,
        });
      }

      if (message.error) {
        batch.reject(new Error(message.error));
        return;
      }

      batch.resolve(message.results);
    },
    [debugMeasurementLogs],
  );

  const handleMeasurementWebViewLoad = useCallback(() => {
    measurementWebViewLoaded.current = true;
    if (measurementFontFaceCssLoading.current) {
      return;
    }
    measurementWebViewReady.current = true;
    flushPendingBatches();
  }, [flushPendingBatches]);

  useEffect(() => {
    measurementWebViewLoaded.current = false;
    measurementWebViewReady.current = false;
  }, [measurementWebViewHtml]);

  useEffect(() => {
    const batches = pendingBatches.current;
    return () => {
      for (const batch of batches.values()) {
        clearTimeout(batch.timer);
        batch.reject(
          new Error(formatMessageRef.current("measurement.cancelled")),
        );
      }
      batches.clear();
    };
  }, []);
  return { webViewRef, installedFontFaceCss, measurementWebViewHtml, measureTextBatch, handleMessage, handleMeasurementWebViewLoad };
}
