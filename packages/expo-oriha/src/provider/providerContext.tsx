// Provider context, public contracts and context access hooks.
import React, { createContext, useContext } from "react";
import type { StyleProp, ViewStyle } from "react-native";
import { OrihaOverlayPlacement, OrihaToastPosition, OrihaToastVariant } from "./OrihaEnums";
import { defaultOrihaFormatMessage } from "../runtime/expoRuntimeI18n";
import type { TextMeasurementRequest, TextMeasurementResult } from "../types";
import type {
  OrihaFontRegistryConfig,
  OrihaInstallFontOptions,
  OrihaInstallableFont,
  OrihaInstalledFont,
} from "../fonts/fontRegistry";
import type { OrihaFormatMessage, OrihaI18nState, OrihaLocale, OrihaMessages } from "../runtime/expoRuntimeI18n";

export type MeasureTextBatchOptions = {
  generationId?: number;
  timeoutMs?: number;
};

export type OrihaContextValue = {
  measureTextBatch: (
    requests: TextMeasurementRequest[],
    options?: MeasureTextBatchOptions,
  ) => Promise<TextMeasurementResult[]>;
  fontRegistry: OrihaFontsState;
  installedFontFaceCss: string;
  presentOverlay: (
    node: React.ReactNode,
    options?: OrihaOverlayOptions,
  ) => () => void;
  dismissOverlay: (id: string) => void;
  showToast: (message: string, options?: OrihaToastOptions) => () => void;
  dismissToast: (id: string) => void;
  i18n: OrihaI18nState;
  formatMessage: OrihaFormatMessage;
};

export type OrihaProviderProps = {
  children: React.ReactNode;
  measurementTimeoutMs?: number;
  debugMeasurementLogs?: boolean;
  fontRegistry?: OrihaFontRegistryConfig;
  locale?: OrihaLocale;
  messages?: OrihaMessages;
};

export type OrihaFontsState = {
  installedFonts: OrihaInstalledFont[];
  installableFonts: OrihaInstallableFont[];
  isLoading: boolean;
  error: unknown;
  listInstalledFonts: () => Promise<OrihaInstalledFont[]>;
  listInstallableFonts: () => Promise<OrihaInstallableFont[]>;
  refreshFontRegistry: () => Promise<OrihaInstallableFont[]>;
  installFont: (
    fontId: string,
    options?: OrihaInstallFontOptions,
  ) => Promise<OrihaInstalledFont>;
  uninstallFont: (fontId: string) => Promise<void>;
};

export type OrihaOverlayOptions = {
  id?: string;
  placement?: OrihaOverlayPlacement;
  pointerEvents?: "box-none" | "none" | "box-only" | "auto";
  style?: StyleProp<ViewStyle>;
};

export type OrihaToastOptions = {
  id?: string;
  durationMs?: number;
  position?: OrihaToastPosition;
  variant?: OrihaToastVariant;
};

export type OverlayEntry = {
  id: string;
  node: React.ReactNode;
  placement: OrihaOverlayPlacement;
  pointerEvents: "box-none" | "none" | "box-only" | "auto";
  style?: StyleProp<ViewStyle>;
};

export type ToastEntry = {
  id: string;
  message: string;
  durationMs: number;
  position: OrihaToastPosition;
  variant: OrihaToastVariant;
};

export const OrihaContext = createContext<OrihaContextValue | null>(
  null,
);

export function useOrihaContext() {
  return useContext(OrihaContext);
}

export function useOrihaFonts() {
  const context = useContext(OrihaContext);
  if (!context) {
    return {
      installedFonts: [],
      installableFonts: [],
      isLoading: false,
      error: null,
      listInstalledFonts: () => Promise.resolve([]),
      listInstallableFonts: () => Promise.resolve([]),
      refreshFontRegistry: () => Promise.resolve([]),
      installFont: () =>
        Promise.reject(
          new Error("useOrihaFonts must be used within OrihaProvider"),
        ),
      uninstallFont: () => Promise.resolve(),
    } satisfies OrihaFontsState;
  }
  return context.fontRegistry;
}

export function useOrihaOverlay() {
  const context = useContext(OrihaContext);
  if (!context) {
    return {
      presentOverlay: () => () => undefined,
      dismissOverlay: () => undefined,
      showToast: () => () => undefined,
      dismissToast: () => undefined,
    } satisfies Pick<
      OrihaContextValue,
      "presentOverlay" | "dismissOverlay" | "showToast" | "dismissToast"
    >;
  }
  return {
    presentOverlay: context.presentOverlay,
    dismissOverlay: context.dismissOverlay,
    showToast: context.showToast,
    dismissToast: context.dismissToast,
  };
}

export function useOrihaI18n() {
  const context = useContext(OrihaContext);
  if (!context) {
    return {
      locale: "en",
      formatMessage: defaultOrihaFormatMessage,
    } satisfies OrihaI18nState;
  }
  return context.i18n;
}
