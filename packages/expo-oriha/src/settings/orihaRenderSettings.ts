import { useCallback, useEffect, useMemo, useState } from "react";
import { DEFAULT_ORIHA_LOCALE } from "../runtime/expoRuntimeI18n";
import { useOrihaFonts } from "../provider/OrihaProvider";
import { normalizeBridgeRenderingOptions, resolveRenderSettings } from "../bridge/render";
import type { BridgeRenderInput, BridgeResolvedSettings } from "../types";
import type { OrihaLocale } from "../runtime/expoRuntimeI18n";
import {
  EMPTY_INSTALLED_ORIHA_FONTS,
  createOrihaRenderSettingsDraft,
  createOrihaRenderSettingsInput,
  formatRenderSettingsError,
  createOrihaRenderSettingsControls,
  createOrihaRenderSettingsOptions,
} from "./renderSettingsModel";
import type {
  UseOrihaRenderSettingsOptions,
  OrihaRenderSettingsState,
  OrihaRenderSettingsDraft,
  OrihaRenderSettingsDraftSetter,
} from "./renderSettingsModel";
export {
  DEFAULT_ORIHA_RENDER_SETTINGS_DRAFT,
  createOrihaRenderSettingsDraft,
  createOrihaRenderSettingsInput,
  createOrihaRenderSettingsControls,
  createOrihaRenderSettingsOptions,
  orihaFontOptionsForKind,
  installedOrihaFontBySelection,
  installedOrihaFontSelectionId,
  bridgeOrihaFontSelection,
  orihaRenderSettingsControlState,
} from "./renderSettingsModel";
export type {
  OrihaInstalledFontSelection,
  OrihaHeadingFontSelection,
  OrihaBodyFontSelection,
  OrihaCodeFontSelection,
  OrihaRenderSettingsFontKind,
  OrihaRenderSettingsDraft,
  OrihaRenderSettingsControlState,
  OrihaRenderSettingsControls,
  OrihaRenderSettingsAvailableOptions,
  OrihaRenderSettingsDraftPatch,
  OrihaRenderSettingsDraftSetter,
  UseOrihaRenderSettingsOptions,
  OrihaRenderSettingsState,
} from "./renderSettingsModel";

export function useOrihaRenderSettings({
  locale,
  initialDraft,
  installedFonts,
  resolvingReason,
  unavailableReason,
}: UseOrihaRenderSettingsOptions = {}): OrihaRenderSettingsState {
  const fontRegistry = useOrihaFonts();
  const activeInstalledFonts =
    installedFonts ??
    (fontRegistry.installedFonts.length > 0
      ? fontRegistry.installedFonts
      : EMPTY_INSTALLED_ORIHA_FONTS);
  const activeLocale = locale ?? DEFAULT_ORIHA_LOCALE;
  const [draft, setDraft] = useState<OrihaRenderSettingsDraft>(() =>
    createOrihaRenderSettingsDraft(initialDraft),
  );
  const setDraftValue = useCallback<OrihaRenderSettingsDraftSetter>(
    (key, value) => {
      setDraft((current) =>
        current[key] === value ? current : { ...current, [key]: value },
      );
    },
    [],
  );
  const selectedInput = useMemo(
    () =>
      createOrihaRenderSettingsInput({
        draft,
        installedFonts: activeInstalledFonts,
        locale: activeLocale,
      }),
    [activeInstalledFonts, activeLocale, draft],
  );
  const [resolverState, setResolverState] = useState<{
    resolvedSettings: BridgeResolvedSettings | null;
    error: string | null;
    isResolving: boolean;
  }>({
    resolvedSettings: null,
    error: null,
    isResolving: true,
  });

  useEffect(() => {
    let cancelled = false;
    setResolverState((current) => ({
      ...current,
      error: null,
      isResolving: true,
    }));

    resolveRenderSettings(selectedInput)
      .then((nextSettings) => {
        if (!cancelled) {
          setResolverState({
            resolvedSettings: nextSettings,
            error: null,
            isResolving: false,
          });
        }
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          setResolverState((current) => ({
            ...current,
            error: formatRenderSettingsError(error),
            isResolving: false,
          }));
        }
      });

    return () => {
      cancelled = true;
    };
  }, [selectedInput]);

  const resolvedSettings = resolverState.resolvedSettings;
  const effectiveInput = resolvedSettings?.effectiveInput ?? selectedInput;
  const effectiveLocale = (effectiveInput.locale ??
    activeLocale) as OrihaLocale;
  const effectiveTableBorderStyle =
    effectiveInput.renderingOptions.tableBorderStyle ?? draft.tableBorderStyle;
  const previewInput = useMemo<Omit<BridgeRenderInput, "markdown">>(
    () => ({
      locale: effectiveLocale,
      plan: effectiveInput.plan,
      renderingOptions: normalizeBridgeRenderingOptions(
        effectiveInput.renderingOptions,
      ),
      print: undefined,
      renderMode: effectiveInput.mode,
      watermark: undefined,
      sourceAttributes: false,
      tableOfContents: false,
      viewport: effectiveInput.viewport,
    }),
    [effectiveInput, effectiveLocale],
  );
  const controls = useMemo(
    () =>
      createOrihaRenderSettingsControls({
        controls: resolvedSettings?.controls,
        resolved: resolvedSettings !== null,
        resolvingReason,
        unavailableReason,
      }),
    [resolvedSettings, resolvingReason, unavailableReason],
  );
  const options = useMemo(
    () =>
      createOrihaRenderSettingsOptions({
        availableOptions: resolvedSettings?.availableOptions,
        installedFonts: activeInstalledFonts,
      }),
    [activeInstalledFonts, resolvedSettings],
  );

  return {
    draft,
    setDraft,
    setDraftValue,
    selectedInput,
    resolvedSettings,
    effectiveInput,
    effectiveLocale,
    effectiveTableBorderStyle,
    previewInput,
    availableOptions: options,
    options,
    controls,
    isResolving: resolverState.isResolving,
    error: resolverState.error,
  };
}
