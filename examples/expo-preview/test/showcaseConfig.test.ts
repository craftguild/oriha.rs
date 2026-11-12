import assert from 'node:assert/strict';
import { test } from 'node:test';
import { resolveFontRegistryConfig, resolveUsageAnalyticsConfig } from '../components/showcase/showcaseConfig.ts';

test('sample networking requires explicit configuration and has no fallback', (t) => {
  const names = ['EXPO_PUBLIC_MARKUP_STATS_INGEST_URL', 'EXPO_PUBLIC_MARKUP_STATS_API_KEY', 'EXPO_PUBLIC_FONT_REGISTRY_URL'];
  const previous = names.map((name) => process.env[name]);
  t.after(() => names.forEach((name, i) => {
    if (previous[i] === undefined) delete process.env[name];
    else process.env[name] = previous[i];
  }));
  names.forEach((name) => delete process.env[name]);
  assert.equal(resolveUsageAnalyticsConfig(), null);
  assert.equal(resolveFontRegistryConfig(), undefined);

  process.env.EXPO_PUBLIC_MARKUP_STATS_API_KEY = 'client-test-key';
  assert.equal(resolveUsageAnalyticsConfig(), null);
  delete process.env.EXPO_PUBLIC_MARKUP_STATS_API_KEY;
  process.env.EXPO_PUBLIC_MARKUP_STATS_INGEST_URL = 'https://statistics.example.invalid/ingest';
  assert.equal(resolveUsageAnalyticsConfig(), null);
  process.env.EXPO_PUBLIC_MARKUP_STATS_API_KEY = '  client-test-key  ';
  assert.equal(resolveUsageAnalyticsConfig()?.ingestUrl, 'https://statistics.example.invalid/ingest');
  assert.equal(resolveUsageAnalyticsConfig()?.apiKey, 'client-test-key');
  process.env.EXPO_PUBLIC_MARKUP_STATS_INGEST_URL = '  ';
  assert.equal(resolveUsageAnalyticsConfig(), null);

  process.env.EXPO_PUBLIC_FONT_REGISTRY_URL = '  https://fonts.example.invalid/index.json  ';
  assert.deepEqual(resolveFontRegistryConfig(), {
    baseUrl: 'https://fonts.example.invalid/index.json', storageNamespace: 'expo-preview',
  });
  process.env.EXPO_PUBLIC_FONT_REGISTRY_URL = ' ';
  assert.equal(resolveFontRegistryConfig(), undefined);
});
