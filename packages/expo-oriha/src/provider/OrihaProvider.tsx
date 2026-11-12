import React, { useCallback, useMemo, useRef, useState } from "react";
import { View } from "react-native";
import { WebView } from "react-native-webview";
import { OrihaOverlayPlacement, OrihaToastPosition, OrihaToastVariant } from "./OrihaEnums";
import { createOrihaI18n } from "../runtime/expoRuntimeI18n";
import { OrihaContext } from "./providerContext";
import type {
  OrihaProviderProps,
  OverlayEntry,
  ToastEntry,
  OrihaOverlayOptions,
  OrihaToastOptions,
  OrihaContextValue,
} from "./providerContext";
import { useProviderFonts, useInitializeProviderFonts } from "./providerFonts";
import { useTextMeasurementBridge } from "./providerMeasurement";
import { styles } from "./providerStyles";
import { overlayPlacementStyle, TOAST_POSITIONS, toastPositionStyle, OrihaToast } from "./providerOverlays";
export { useOrihaContext, useOrihaFonts, useOrihaOverlay, useOrihaI18n } from "./providerContext";
export type { OrihaProviderProps, OrihaFontsState, OrihaOverlayOptions, OrihaToastOptions } from "./providerContext";

const DEFAULT_MEASURE_TIMEOUT_MS = 5_000;

export function OrihaProvider({
  children,
  measurementTimeoutMs = DEFAULT_MEASURE_TIMEOUT_MS,
  debugMeasurementLogs = false,
  fontRegistry,
  locale,
  messages,
}: OrihaProviderProps) {
  const nextOverlayId = useRef(0);

  const nextToastId = useRef(0);

  const [overlays, setOverlays] = useState<OverlayEntry[]>([]);

  const [toasts, setToasts] = useState<ToastEntry[]>([]);

  const fontState = useProviderFonts(fontRegistry);
  const { fontRegistryState, installedFonts, setFontRegistryError } = fontState;

  const i18n = useMemo(
    () => createOrihaI18n(locale, messages),
    [locale, messages],
  );

  const {
    webViewRef,
    installedFontFaceCss,
    measurementWebViewHtml,
    measureTextBatch,
    handleMessage,
    handleMeasurementWebViewLoad,
  } = useTextMeasurementBridge({
    installedFonts,
    measurementTimeoutMs,
    debugMeasurementLogs,
    formatMessage: i18n.formatMessage,
    setFontRegistryError,
  });

  const dismissOverlay = useCallback((id: string) => {
    setOverlays((current) => current.filter((entry) => entry.id !== id));
  }, []);

  const presentOverlay = useCallback(
    (node: React.ReactNode, options?: OrihaOverlayOptions) => {
      const id = options?.id ?? `overlay-${Date.now()}-${nextOverlayId.current++}`;
      const entry: OverlayEntry = {
        id,
        node,
        placement: options?.placement ?? OrihaOverlayPlacement.Center,
        pointerEvents: options?.pointerEvents ?? "box-none",
        style: options?.style,
      };
      setOverlays((current) => [
        ...current.filter((existing) => existing.id !== id),
        entry,
      ]);
      return () => {
        dismissOverlay(id);
      };
    },
    [dismissOverlay],
  );

  const dismissToast = useCallback((id: string) => {
    setToasts((current) => current.filter((toast) => toast.id !== id));
  }, []);

  const showToast = useCallback(
    (message: string, options?: OrihaToastOptions) => {
      const id = options?.id ?? `toast-${Date.now()}-${nextToastId.current++}`;
      const entry: ToastEntry = {
        id,
        message,
        durationMs: options?.durationMs ?? 1_800,
        position: options?.position ?? OrihaToastPosition.Top,
        variant: options?.variant ?? OrihaToastVariant.Neutral,
      };
      setToasts((current) => [
        ...current.filter((toast) => toast.id !== id),
        entry,
      ]);
      return () => {
        dismissToast(id);
      };
    },
    [dismissToast],
  );

  useInitializeProviderFonts(fontRegistry, fontState);

  const contextValue = useMemo<OrihaContextValue>(
    () => ({
      measureTextBatch,
      fontRegistry: fontRegistryState,
      installedFontFaceCss,
      presentOverlay,
      dismissOverlay,
      showToast,
      dismissToast,
      i18n,
      formatMessage: i18n.formatMessage,
    }),
    [
      dismissOverlay,
      dismissToast,
      fontRegistryState,
      i18n,
      installedFontFaceCss,
      measureTextBatch,
      presentOverlay,
      showToast,
    ],
  );

  return (
    <OrihaContext.Provider value={contextValue}>
      <View style={styles.root}>
        {children}
        <View pointerEvents="box-none" style={styles.overlayHost}>
          {overlays.map((entry) => (
            <View
              key={entry.id}
              pointerEvents={entry.pointerEvents}
              style={[
                styles.overlaySlot,
                overlayPlacementStyle(entry.placement),
                entry.style,
              ]}
            >
              {entry.node}
            </View>
          ))}
          <View pointerEvents="box-none" style={styles.toastHost}>
            {TOAST_POSITIONS.map((position) => {
              const positionedToasts = toasts.filter(
                (toast) => toast.position === position,
              );
              if (positionedToasts.length === 0) {
                return null;
              }
              return (
                <View
                  key={position}
                  pointerEvents="box-none"
                  style={[styles.toastSlot, toastPositionStyle(position)]}
                >
                  {positionedToasts.map((toast) => (
                    <View key={toast.id} style={styles.toastItem}>
                      <OrihaToast
                        toast={toast}
                        onDismiss={dismissToast}
                      />
                    </View>
                  ))}
                </View>
              );
            })}
          </View>
        </View>
        <View
          collapsable={false}
          pointerEvents="none"
          style={styles.measurementHost}
        >
          <WebView
            ref={webViewRef}
            allowFileAccess
            originWhitelist={["*"]}
            source={{ html: measurementWebViewHtml }}
            onLoadEnd={handleMeasurementWebViewLoad}
            onMessage={handleMessage}
            scrollEnabled={false}
            style={styles.measurementWebView}
          />
        </View>
      </View>
    </OrihaContext.Provider>
  );
}
