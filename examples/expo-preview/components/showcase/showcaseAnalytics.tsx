// Opt-in usage analytics reporter state and its settings control.
import Constants from 'expo-constants';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { Text, View } from 'react-native';
import {
  Plan,
  createUsageAnalyticsReporter,
  type OrihaLocale,
  type RenderOutput,
  type UsageAnalyticsReporter,
  type UsageAnalyticsReporterContext,
} from '@craftguild/expo-oriha';
import { analyticsPlatform, buildVersion, formatError } from "./showcaseRuntime";
import { resolveUsageAnalyticsConfig } from "./showcaseConfig";
import { styles } from "./showcaseStyles";
import { SmallButton } from "./showcaseFonts";

export function useUsageAnalytics({
  effectiveLocale,
  plan,
}: {
  effectiveLocale: OrihaLocale;
  plan: Plan;
}) {
  const [enabled, setEnabled] = useState(false);
  const [status, setStatus] = useState<string | null>(null);
  const config = useMemo(resolveUsageAnalyticsConfig, []);
  const context = useMemo<UsageAnalyticsReporterContext>(
    () => ({
      platform: analyticsPlatform(),
      appVersion: Constants.expoConfig?.version ?? '1.0.0',
      buildVersion: buildVersion(),
      locale: effectiveLocale,
      subscriptionPlan: plan,
    }),
    [effectiveLocale, plan],
  );
  // Own one outbox so disabling delivery also stops its pending retries.
  const [reporter] = useState<UsageAnalyticsReporter | null>(() => {
    if (!config) {
      return null;
    }

    return createUsageAnalyticsReporter(config, context, {
      isDeliveryEnabled: false,
      logger: {
        debug: (...messages: unknown[]) => {
          console.debug('[UsageAnalytics]', ...messages);
        },
        warn: (...messages: unknown[]) => {
          console.warn('[UsageAnalytics]', ...messages);
        },
      },
      storageNamespace: 'expo-preview',
    });
  });

  useEffect(() => {
    if (!reporter) {
      return;
    }

    reporter.updateContext(context);
    reporter.setDeliveryEnabled(enabled).catch((error: unknown) => {
      setStatus(`Usage analytics delivery update failed: ${formatError(error)}`);
    });
  }, [context, enabled, reporter]);

  useEffect(() => () => {
    reporter?.setDeliveryEnabled(false).catch((error: unknown) => {
      console.warn('[UsageAnalytics] Cleanup failed:', formatError(error));
    });
  }, [reporter]);

  const recordRender = useCallback(
    (output: RenderOutput) => {
      if (!enabled) {
        setStatus('Usage analytics disabled.');
        return;
      }
      if (!reporter) {
        setStatus('Set EXPO_PUBLIC_MARKUP_STATS_INGEST_URL and EXPO_PUBLIC_MARKUP_STATS_API_KEY to send render stats.');
        return;
      }

      reporter
        .recordRenderComplete({
          stats: output.stats,
          context,
        })
        .then(() => {
          setStatus('Usage analytics event queued.');
        })
        .catch((error: unknown) => {
          setStatus(`Usage analytics failed: ${formatError(error)}`);
        });
    },
    [context, enabled, reporter],
  );
  const toggle = useCallback(() => {
    setEnabled((current) => !current);
  }, []);

  return useMemo(
    () => ({
      enabled,
      isConfigured: config !== null,
      recordRender,
      status,
      toggle,
    }),
    [config, enabled, recordRender, status, toggle],
  );
}

export function UsageAnalyticsControl({
  enabled,
  isConfigured,
  onToggle,
  status,
}: {
  enabled: boolean;
  isConfigured: boolean;
  onToggle: () => void;
  status: string | null;
}) {
  return (
    <View style={styles.usageAnalytics}>
      <View style={styles.usageAnalyticsHeader}>
        <View style={styles.usageAnalyticsText}>
          <Text style={styles.optionLabel}>Usage Analytics</Text>
          <Text style={styles.fontMeta}>
            Sends render statistics from `onRendered`; Markdown text is not included.
          </Text>
          <Text style={isConfigured ? styles.fontStatus : styles.fontError}>
            {isConfigured
              ? 'Endpoint and client API key configured. Enable to send statistics.'
              : 'Set EXPO_PUBLIC_MARKUP_STATS_INGEST_URL and EXPO_PUBLIC_MARKUP_STATS_API_KEY to enable delivery.'}
          </Text>
        </View>
        <SmallButton
          disabled={!isConfigured}
          label={enabled && isConfigured ? 'Enabled' : 'Disabled'}
          onPress={onToggle}
        />
      </View>
      {status ? <Text style={styles.fontMeta}>{status}</Text> : null}
    </View>
  );
}
