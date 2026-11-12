// Render-setting choices, option rows and draft controls.
import { Pressable, ScrollView, Text, View } from 'react-native';
import {
  type BridgeEffectiveSettingsInput,
  type BridgeOption,
  type BridgeSettingsInput,
  type OrihaLocale,
  type OrihaRenderSettingsAvailableOptions,
  type OrihaRenderSettingsControls,
  type OrihaRenderSettingsDraft,
  type OrihaRenderSettingsDraftSetter,
  type TableBorderStyle,
} from '@craftguild/expo-oriha';
import { markdownSamples } from "./OrihaShowcase";
import { styles } from "./showcaseStyles";

export type MarkdownSampleId = 'standard' | 'eyecatch';

export type TypedBridgeOption<Id extends string> = Omit<BridgeOption, 'id'> & { id: Id };

export function SettingsControls({
  controls,
  draft,
  effectiveInput,
  effectiveLocale,
  effectiveTableBorderStyle,
  onDraftChange,
  onLocaleChange,
  onSampleChange,
  options,
  sampleId,
}: {
  controls: OrihaRenderSettingsControls;
  draft: OrihaRenderSettingsDraft;
  effectiveInput: BridgeSettingsInput | BridgeEffectiveSettingsInput;
  effectiveLocale: OrihaLocale;
  effectiveTableBorderStyle: TableBorderStyle;
  onDraftChange: OrihaRenderSettingsDraftSetter;
  onLocaleChange: (locale: OrihaLocale) => void;
  onSampleChange: (sampleId: MarkdownSampleId) => void;
  options: OrihaRenderSettingsAvailableOptions;
  sampleId: MarkdownSampleId;
}) {
  return (
    <>
      <OptionRow
        label="Markdown"
        options={markdownSamples}
        selectedId={sampleId}
        onSelect={onSampleChange}
      />
      <OptionRow
        disabled={controls.plan.disabled}
        disabledReason={controls.plan.reason}
        label="Plan"
        options={options.plan}
        selectedId={effectiveInput.plan}
        onSelect={(plan) => onDraftChange('plan', plan)}
      />
      <OptionRow
        disabled={controls.locale.disabled}
        disabledReason={controls.locale.reason}
        label="Locale"
        options={options.locale}
        selectedId={effectiveLocale}
        onSelect={onLocaleChange}
      />
      <OptionRow
        disabled={controls.colorScheme.disabled}
        disabledReason={controls.colorScheme.reason}
        label="Color"
        options={options.colorScheme}
        selectedId={effectiveInput.renderingOptions?.colorScheme ?? draft.colorScheme}
        onSelect={(colorScheme) => onDraftChange('colorScheme', colorScheme)}
      />
      <OptionRow
        disabled={controls.syntaxTheme.disabled}
        disabledReason={controls.syntaxTheme.reason}
        label="Syntax"
        options={options.syntaxTheme}
        selectedId={effectiveInput.renderingOptions?.syntaxTheme ?? draft.syntaxTheme}
        onSelect={(syntaxTheme) => onDraftChange('syntaxTheme', syntaxTheme)}
      />
      <OptionRow
        disabled={controls.tableBorderStyle.disabled}
        disabledReason={controls.tableBorderStyle.reason}
        label="Table Border"
        options={options.tableBorderStyle}
        selectedId={effectiveTableBorderStyle}
        onSelect={(tableBorderStyle) => onDraftChange('tableBorderStyle', tableBorderStyle)}
      />
      <OptionRow
        disabled={controls.headingFont.disabled}
        disabledReason={controls.headingFont.reason}
        label="Heading Font"
        options={options.headingFont}
        selectedId={draft.headingFont}
        onSelect={(headingFont) => onDraftChange('headingFont', headingFont)}
      />
      <OptionRow
        disabled={controls.bodyFont.disabled}
        disabledReason={controls.bodyFont.reason}
        label="Body Font"
        options={options.bodyFont}
        selectedId={draft.bodyFont}
        onSelect={(bodyFont) => onDraftChange('bodyFont', bodyFont)}
      />
      <OptionRow
        disabled={controls.codeFont.disabled}
        disabledReason={controls.codeFont.reason}
        label="Code Font"
        options={options.codeFont}
        selectedId={draft.codeFont}
        onSelect={(codeFont) => onDraftChange('codeFont', codeFont)}
      />
    </>
  );
}

export function OptionRow<T extends string>({
  disabled = false,
  disabledReason,
  label,
  onSelect,
  options,
  selectedId,
}: {
  disabled?: boolean;
  disabledReason?: string;
  label: string;
  onSelect: (id: T) => void;
  options: readonly TypedBridgeOption<T>[];
  selectedId: T;
}) {
  return (
    <View style={styles.optionRow}>
      <View style={styles.optionHeader}>
        <Text style={styles.optionLabel}>{label}</Text>
        {disabled && disabledReason ? (
          <Text style={styles.disabledReason}>{disabledReason}</Text>
        ) : null}
      </View>
      <ScrollView horizontal showsHorizontalScrollIndicator={false}>
        <View style={styles.optionList}>
          {options.map((option) => {
            const selected = option.id === selectedId;
            return (
              <Pressable
                accessibilityRole="button"
                accessibilityState={{ disabled, selected }}
                disabled={disabled}
                key={option.id}
                onPress={() => onSelect(option.id)}
                style={[
                  styles.optionButton,
                  selected ? styles.optionButtonSelected : null,
                  disabled ? styles.optionButtonDisabled : null,
                ]}
              >
                <Text
                  style={[
                    styles.optionText,
                    selected ? styles.optionTextSelected : null,
                    disabled ? styles.optionTextDisabled : null,
                  ]}
                >
                  {option.label}
                </Text>
              </Pressable>
            );
          })}
        </View>
      </ScrollView>
    </View>
  );
}
