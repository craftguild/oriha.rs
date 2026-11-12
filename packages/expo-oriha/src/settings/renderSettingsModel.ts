// Render-settings drafts, bridge inputs, option lists and control availability.
import type { Dispatch, SetStateAction } from "react";
import {
  BodyFont,
  CodeFont,
  ColorScheme,
  HeadingFont,
  Plan,
  RenderMode,
  SyntaxTheme,
  TableBorderStyle,
} from "../types";
import { normalizeBridgeRenderingOptions } from "../bridge/render";
import type {
  BridgeAvailableOptions,
  BridgeControlAvailability,
  BridgeEffectiveSettingsInput,
  BridgeOption,
  BridgeRenderInput,
  BridgeResolvedSettings,
  BridgeSettingsInput,
} from "../types";
import type { OrihaInstalledFont, OrihaFontKind } from "../fonts/fontRegistry";
import type { OrihaLocale } from "../runtime/expoRuntimeI18n";

export type TypedBridgeOption<T extends string> = Omit<BridgeOption, "id"> & {
  id: T;
};

export type OrihaInstalledFontSelection = `installed:${string}`;

export type OrihaHeadingFontSelection =
  | HeadingFont
  | OrihaInstalledFontSelection;

export type OrihaBodyFontSelection = BodyFont | OrihaInstalledFontSelection;

export type OrihaCodeFontSelection = CodeFont | OrihaInstalledFontSelection;

export type OrihaRenderSettingsFontKind = Extract<
  OrihaFontKind,
  "heading" | "body" | "code"
>;

export type OrihaRenderSettingsDraft = {
  plan: Plan;
  colorScheme: ColorScheme;
  syntaxTheme: SyntaxTheme;
  headingFont: OrihaHeadingFontSelection;
  bodyFont: OrihaBodyFontSelection;
  bodyFontSizePx: number;
  codeFont: OrihaCodeFontSelection;
  tableBorderStyle: TableBorderStyle;
  appearanceStyle?: NonNullable<BridgeSettingsInput["renderingOptions"]>["appearanceStyle"];
  customSyntaxTheme?: string;
  mode: RenderMode;
  viewport?: BridgeSettingsInput["viewport"];
  roundedCorners?: boolean;
  shadows?: boolean;
  fullTableData?: boolean;
  blockBorders?: boolean;
};

export type OrihaRenderSettingsControlState = {
  disabled: boolean;
  reason?: string;
};

export type OrihaRenderSettingsControls = {
  plan: OrihaRenderSettingsControlState;
  colorScheme: OrihaRenderSettingsControlState;
  locale: OrihaRenderSettingsControlState;
  syntaxTheme: OrihaRenderSettingsControlState;
  headingFont: OrihaRenderSettingsControlState;
  bodyFont: OrihaRenderSettingsControlState;
  codeFont: OrihaRenderSettingsControlState;
  tableBorderStyle: OrihaRenderSettingsControlState;
};

export type OrihaRenderSettingsAvailableOptions = {
  plan: readonly TypedBridgeOption<Plan>[];
  colorScheme: readonly TypedBridgeOption<ColorScheme>[];
  locale: readonly TypedBridgeOption<OrihaLocale>[];
  syntaxTheme: readonly TypedBridgeOption<SyntaxTheme>[];
  headingFont: readonly TypedBridgeOption<OrihaHeadingFontSelection>[];
  bodyFont: readonly TypedBridgeOption<OrihaBodyFontSelection>[];
  codeFont: readonly TypedBridgeOption<OrihaCodeFontSelection>[];
  tableBorderStyle: readonly TypedBridgeOption<TableBorderStyle>[];
};

export type OrihaRenderSettingsDraftPatch =
  Partial<OrihaRenderSettingsDraft>;

export type OrihaRenderSettingsDraftSetter = <
  K extends keyof OrihaRenderSettingsDraft,
>(
  key: K,
  value: OrihaRenderSettingsDraft[K],
) => void;

export type UseOrihaRenderSettingsOptions = {
  locale?: OrihaLocale;
  initialDraft?: OrihaRenderSettingsDraftPatch;
  installedFonts?: readonly OrihaInstalledFont[];
  resolvingReason?: string;
  unavailableReason?: string;
};

export type OrihaRenderSettingsState = {
  draft: OrihaRenderSettingsDraft;
  setDraft: Dispatch<SetStateAction<OrihaRenderSettingsDraft>>;
  setDraftValue: OrihaRenderSettingsDraftSetter;
  selectedInput: BridgeSettingsInput;
  resolvedSettings: BridgeResolvedSettings | null;
  effectiveInput: BridgeSettingsInput | BridgeEffectiveSettingsInput;
  effectiveLocale: OrihaLocale;
  effectiveTableBorderStyle: TableBorderStyle;
  previewInput: Omit<BridgeRenderInput, "markdown">;
  availableOptions: OrihaRenderSettingsAvailableOptions;
  options: OrihaRenderSettingsAvailableOptions;
  controls: OrihaRenderSettingsControls;
  isResolving: boolean;
  error: string | null;
};

export const DEFAULT_ORIHA_RENDER_SETTINGS_DRAFT: OrihaRenderSettingsDraft =
{
  plan: Plan.Free,
  colorScheme: ColorScheme.Light,
  syntaxTheme: SyntaxTheme.GitHubLight,
  headingFont: HeadingFont.SystemDefault,
  bodyFont: BodyFont.SystemDefault,
  bodyFontSizePx: 16,
  codeFont: CodeFont.SystemDefault,
  tableBorderStyle: TableBorderStyle.Inner,
  mode: RenderMode.StyledHtml,
};

export const EMPTY_INSTALLED_ORIHA_FONTS: readonly OrihaInstalledFont[] = [];

export function createOrihaRenderSettingsDraft(
  patch: OrihaRenderSettingsDraftPatch = {},
): OrihaRenderSettingsDraft {
  return {
    ...DEFAULT_ORIHA_RENDER_SETTINGS_DRAFT,
    ...patch,
  };
}

export function createOrihaRenderSettingsInput({
  draft,
  installedFonts = [],
  locale,
}: {
  draft: OrihaRenderSettingsDraft;
  installedFonts?: readonly OrihaInstalledFont[];
  locale?: OrihaLocale;
}): BridgeSettingsInput {
  const selectedHeadingInstalledFont = installedOrihaFontBySelection(
    installedFonts,
    draft.headingFont,
    "heading",
  );
  const selectedBodyInstalledFont = installedOrihaFontBySelection(
    installedFonts,
    draft.bodyFont,
    "body",
  );
  const selectedCodeInstalledFont = installedOrihaFontBySelection(
    installedFonts,
    draft.codeFont,
    "code",
  );

  return {
    mode: draft.mode,
    locale,
    plan: draft.plan,
    renderingOptions: normalizeBridgeRenderingOptions({
      colorScheme: draft.colorScheme,
      appearanceStyle: draft.appearanceStyle,
      syntaxTheme: draft.syntaxTheme,
      customSyntaxTheme: draft.customSyntaxTheme,
      headingFont: selectedHeadingInstalledFont
        ? HeadingFont.Custom
        : bridgeOrihaFontSelection(draft.headingFont, HeadingFont.SystemDefault),
      customHeadingFont: selectedHeadingInstalledFont?.fontFamilySpec,
      bodyFont: selectedBodyInstalledFont
        ? BodyFont.Custom
        : bridgeOrihaFontSelection(draft.bodyFont, BodyFont.SystemDefault),
      customBodyFont: selectedBodyInstalledFont?.fontFamilySpec,
      bodyFontSizePx: draft.bodyFontSizePx,
      codeFont: selectedCodeInstalledFont
        ? CodeFont.Custom
        : bridgeOrihaFontSelection(draft.codeFont, CodeFont.SystemDefault),
      customCodeFont: selectedCodeInstalledFont?.fontFamilySpec,
      tableBorderStyle: draft.tableBorderStyle,
      roundedCorners: draft.roundedCorners,
      shadows: draft.shadows,
      fullTableData: draft.fullTableData,
      blockBorders: draft.blockBorders,
    }),
    viewport: draft.viewport,
  };
}

export function createOrihaRenderSettingsControls({
  controls,
  resolved,
  resolvingReason,
  unavailableReason,
}: {
  controls?: {
    plan?: BridgeControlAvailability;
    colorScheme?: BridgeControlAvailability;
    syntaxTheme?: BridgeControlAvailability;
    headingFont?: BridgeControlAvailability;
    bodyFont?: BridgeControlAvailability;
    codeFont?: BridgeControlAvailability;
    tableBorderStyle?: BridgeControlAvailability;
  };
  resolved: boolean;
  resolvingReason?: string;
  unavailableReason?: string;
}): OrihaRenderSettingsControls {
  return {
    plan: orihaRenderSettingsControlState(controls?.plan, resolved, {
      resolvingReason,
      unavailableReason,
    }),
    colorScheme: orihaRenderSettingsControlState(
      controls?.colorScheme,
      resolved,
      { resolvingReason, unavailableReason },
    ),
    locale: { disabled: false },
    syntaxTheme: orihaRenderSettingsControlState(
      controls?.syntaxTheme,
      resolved,
      { resolvingReason, unavailableReason },
    ),
    headingFont: orihaRenderSettingsControlState(
      controls?.headingFont,
      resolved,
      { resolvingReason, unavailableReason },
    ),
    bodyFont: orihaRenderSettingsControlState(controls?.bodyFont, resolved, {
      resolvingReason,
      unavailableReason,
    }),
    codeFont: orihaRenderSettingsControlState(controls?.codeFont, resolved, {
      resolvingReason,
      unavailableReason,
    }),
    tableBorderStyle: orihaRenderSettingsControlState(
      controls?.tableBorderStyle,
      resolved,
      { resolvingReason, unavailableReason },
    ),
  };
}

export function createOrihaRenderSettingsOptions({
  availableOptions,
  installedFonts = [],
}: {
  availableOptions?: Partial<BridgeAvailableOptions>;
  installedFonts?: readonly OrihaInstalledFont[];
}): OrihaRenderSettingsAvailableOptions {
  return {
    plan: typedOptions<Plan>(availableOptions?.plan ?? []),
    colorScheme: typedOptions<ColorScheme>(availableOptions?.colorScheme ?? []),
    locale: markdownLocaleOptions(availableOptions?.locale ?? []),
    syntaxTheme: typedOptions<SyntaxTheme>(availableOptions?.syntaxTheme ?? []),
    headingFont: orihaFontOptionsForKind(
      typedOptions<HeadingFont>(availableOptions?.headingFont ?? []),
      installedFonts,
      "heading",
    ),
    bodyFont: orihaFontOptionsForKind(
      typedOptions<BodyFont>(availableOptions?.bodyFont ?? []),
      installedFonts,
      "body",
    ),
    codeFont: orihaFontOptionsForKind(
      typedOptions<CodeFont>(availableOptions?.codeFont ?? []),
      installedFonts,
      "code",
    ),
    tableBorderStyle: typedOptions<TableBorderStyle>(
      availableOptions?.tableBorderStyle ?? [],
    ),
  };
}

export function orihaFontOptionsForKind<T extends string>(
  bridgeOptions: readonly TypedBridgeOption<T>[],
  installedFonts: readonly OrihaInstalledFont[],
  kind: OrihaRenderSettingsFontKind,
): TypedBridgeOption<T | OrihaInstalledFontSelection>[] {
  return [
    ...bridgeOptions.filter((option) => option.id !== "custom"),
    ...installedFonts
      .filter((font) => font.kinds.includes(kind))
      .map((font) => ({
        id: installedOrihaFontSelectionId(font.id),
        label: font.label,
        description: font.description,
      })),
  ];
}

export function installedOrihaFontBySelection(
  fonts: readonly OrihaInstalledFont[],
  selectedId: string,
  kind: OrihaRenderSettingsFontKind,
): OrihaInstalledFont | null {
  if (!selectedId.startsWith("installed:")) {
    return null;
  }

  const fontId = selectedId.slice("installed:".length);
  return fonts.find((font) => font.id === fontId && font.kinds.includes(kind)) ?? null;
}

export function installedOrihaFontSelectionId(
  fontId: string,
): OrihaInstalledFontSelection {
  return `installed:${fontId}`;
}

export function bridgeOrihaFontSelection<TGenerated extends string>(
  selectedId: TGenerated | OrihaInstalledFontSelection,
  defaultValue: TGenerated,
): TGenerated {
  return selectedId.startsWith("installed:")
    ? defaultValue
    : (selectedId as TGenerated);
}

export function orihaRenderSettingsControlState(
  control: BridgeControlAvailability | undefined,
  resolved: boolean,
  options: {
    resolvingReason?: string;
    unavailableReason?: string;
  } = {},
): OrihaRenderSettingsControlState {
  if (!resolved) {
    return {
      disabled: true,
      reason: options.resolvingReason ?? "Resolving settings...",
    };
  }

  if (control?.enabled === false) {
    return {
      disabled: true,
      reason: control.reason ?? options.unavailableReason ?? "Unavailable for this plan.",
    };
  }

  return { disabled: false };
}

export function markdownLocaleOptions(
  options: readonly BridgeOption[],
): TypedBridgeOption<OrihaLocale>[] {
  return typedOptions<OrihaLocale>(options);
}

export function typedOptions<T extends string>(
  options: readonly BridgeOption[],
): TypedBridgeOption<T>[] {
  return options.map((option) => ({ ...option, id: option.id as T }));
}

export function formatRenderSettingsError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
