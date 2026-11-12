import { useCallback, useState } from 'react';
import { ActivityIndicator, Alert, ScrollView, Text, useWindowDimensions, View } from 'react-native';
import {
  OrihaProvider,
  OrihaPreview,
  Plan,
  useOrihaFonts,
  useOrihaRenderSettings,
  type OrihaLocale,
  type RenderOutput,
} from '@craftguild/expo-oriha';
import { PremiumCtaBanner } from '@/components/premium-cta-banner';
import { paywallEyecatchMarkdown } from '@/lib/paywall-eyecatch-markdown';
import { sampleMarkdown } from '@/lib/sample-markdown';
import { SettingsControls } from "./showcaseSettings";
import type { TypedBridgeOption, MarkdownSampleId } from "./showcaseSettings";
import { PHONE_LAYOUT_MAX_WIDTH, styles } from "./showcaseStyles";
import { useFontInstallerActions, FontInstaller } from "./showcaseFonts";
import { useUsageAnalytics, UsageAnalyticsControl } from "./showcaseAnalytics";
import { useBridgeVersionInfo, BridgeVersionSection } from "./showcaseVersion";
import { formatError } from "./showcaseRuntime";
import { resolveFontRegistryConfig } from "./showcaseConfig";
import { ComponentsSection, RouterTabsPreview } from "./showcaseComponents";

const fontRegistryConfig = resolveFontRegistryConfig();

export const markdownSamples = [
  {
    id: 'standard',
    label: 'Standard',
    markdown: sampleMarkdown,
  },
  {
    id: 'eyecatch',
    label: 'Eyecatch',
    markdown: paywallEyecatchMarkdown,
  },
] satisfies readonly (TypedBridgeOption<MarkdownSampleId> & { markdown: string })[];

export default function OrihaPreviewExample() {
  const [locale, setLocale] = useState<OrihaLocale>('en');

  return (
    <OrihaProvider
      fontRegistry={fontRegistryConfig}
      locale={locale}
    >
      <OrihaPreviewShowcase locale={locale} onLocaleChange={setLocale} />
    </OrihaProvider>
  );
}

function OrihaPreviewShowcase({
  locale,
  onLocaleChange,
}: {
  locale: OrihaLocale;
  onLocaleChange: (locale: OrihaLocale) => void;
}) {
  const {
    error: fontRegistryError,
    installableFonts,
    installedFonts,
    installFont,
    isLoading: fontRegistryLoading,
    refreshFontRegistry,
    uninstallFont,
  } = useOrihaFonts();
  const [sampleId, setSampleId] = useState<MarkdownSampleId>('standard');
  const [stats, setStats] = useState<RenderOutput['stats'] | null>(null);
  const [renderError, setRenderError] = useState<string | null>(null);
  const { width } = useWindowDimensions();
  const showsRouterTabsPreview = width <= PHONE_LAYOUT_MAX_WIDTH;
  // The SDK owns bridge resolution, plan clamping, localized option labels, and
  // installed-font selection mapping. The preview keeps only UI-only state here.
  const {
    controls,
    draft,
    effectiveInput,
    effectiveLocale,
    effectiveTableBorderStyle,
    options,
    previewInput,
    setDraftValue,
    error: settingsError,
  } = useOrihaRenderSettings({
    installedFonts,
    locale,
  });
  const {
    action: fontAction,
    progress: fontProgress,
    runFontInstall,
    runFontRefresh,
    runFontUninstall,
    status: fontStatus,
  } = useFontInstallerActions({
    installFont,
    refreshFontRegistry,
    uninstallFont,
  });
  const {
    enabled: usageAnalyticsEnabled,
    isConfigured: usageAnalyticsConfigured,
    recordRender: recordUsageAnalyticsRender,
    status: usageAnalyticsStatus,
    toggle: toggleUsageAnalytics,
  } = useUsageAnalytics({
    effectiveLocale,
    plan: draft.plan,
  });
  const bridgeVersion = useBridgeVersionInfo();
  const selectedMarkdown =
    markdownSamples.find((sample) => sample.id === sampleId)?.markdown ?? sampleMarkdown;

  const handleRendered = useCallback(
    (output: RenderOutput) => {
      setRenderError(null);
      setStats(output.stats);
      recordUsageAnalyticsRender(output);
    },
    [recordUsageAnalyticsRender],
  );

  const handleRenderError = useCallback((error: unknown) => {
    setRenderError(formatError(error));
  }, []);

  return (
    <View style={styles.screen}>
      <View style={styles.appContent}>
        <View style={styles.settingsPanel}>
          <ScrollView contentContainerStyle={styles.controls}>
            <SettingsControls
              controls={controls}
              draft={draft}
              effectiveInput={effectiveInput}
              effectiveLocale={effectiveLocale}
              effectiveTableBorderStyle={effectiveTableBorderStyle}
              onDraftChange={setDraftValue}
              onLocaleChange={onLocaleChange}
              onSampleChange={setSampleId}
              options={options}
              sampleId={sampleId}
            />

            <FontInstaller
              isConfigured={fontRegistryConfig !== undefined}
              action={fontAction}
              error={fontRegistryError}
              installableFonts={installableFonts}
              installedFonts={installedFonts}
              isLoading={fontRegistryLoading}
              onInstall={runFontInstall}
              onRefresh={runFontRefresh}
              onUninstall={runFontUninstall}
              progress={fontProgress}
              status={fontStatus}
            />

            <UsageAnalyticsControl
              enabled={usageAnalyticsEnabled}
              isConfigured={usageAnalyticsConfigured}
              onToggle={toggleUsageAnalytics}
              status={usageAnalyticsStatus}
            />

            <ComponentsSection />

            <Text style={styles.statusText}>
              {settingsError
                ? `Settings error: ${settingsError}`
                : renderError
                  ? `Render error: ${renderError}`
                  : stats
                    ? `${stats.totalBlocks} blocks rendered`
                    : 'Waiting for first render'}
            </Text>

            <BridgeVersionSection error={bridgeVersion.error} info={bridgeVersion.info} />
          </ScrollView>
        </View>

        <View style={styles.previewPanel}>
          <OrihaPreview
            markdown={selectedMarkdown}
            input={previewInput}
            loadingFallback={
              <View style={styles.loading}>
                <ActivityIndicator />
                <Text style={styles.loadingText}>Rendering Markdown...</Text>
              </View>
            }
            onRendered={handleRendered}
            onRenderError={handleRenderError}
            webViewProps={{
              style: styles.preview,
              scrollEnabled: true,
            }}
          />
        </View>
        {draft.plan === Plan.Free ? (
          <PremiumCtaBanner
            colorScheme={draft.colorScheme}
            locale={locale}
            onPress={() => {
              Alert.alert('Premium CTA', 'Connect a paywall here.');
            }}
          />
        ) : null}
      </View>
      {showsRouterTabsPreview ? <RouterTabsPreview colorScheme={draft.colorScheme} /> : null}
    </View>
  );
}
