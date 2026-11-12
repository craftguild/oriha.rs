// Sample-only network settings. Expo inlines these static property references.
import type { OrihaFontRegistryConfig, UsageAnalyticsOutboxConfig } from '@craftguild/expo-oriha';

declare const process: {
  env: {
    EXPO_PUBLIC_MARKUP_STATS_INGEST_URL?: string;
    EXPO_PUBLIC_MARKUP_STATS_API_KEY?: string;
    EXPO_PUBLIC_FONT_REGISTRY_URL?: string;
  };
};

export function resolveUsageAnalyticsConfig(): UsageAnalyticsOutboxConfig | null {
  const ingestUrl = process.env.EXPO_PUBLIC_MARKUP_STATS_INGEST_URL?.trim();
  const apiKey = process.env.EXPO_PUBLIC_MARKUP_STATS_API_KEY?.trim();
  if (!ingestUrl || !apiKey) {
    return null;
  }
  return {
    ingestUrl,
    apiKey,
    requestTimeoutMs: 5000,
    maxQueuedEvents: 200,
    retentionMs: 7 * 24 * 60 * 60 * 1000,
  };
}

export function resolveFontRegistryConfig(): OrihaFontRegistryConfig | undefined {
  const baseUrl = process.env.EXPO_PUBLIC_FONT_REGISTRY_URL?.trim();
  return baseUrl ? { baseUrl, storageNamespace: 'expo-preview' } : undefined;
}
