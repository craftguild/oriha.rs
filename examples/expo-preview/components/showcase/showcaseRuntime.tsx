// Explicit telemetry configuration, platform identity and application build metadata.
import Constants from 'expo-constants';
import { Platform } from 'react-native';
import { type UsageAnalyticsReporterContext } from '@craftguild/expo-oriha';

export function formatError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export function analyticsPlatform(): UsageAnalyticsReporterContext['platform'] {
  switch (Platform.OS) {
    case 'ios':
      return 'iOS';
    case 'android':
      return 'Android';
    case 'web':
      return 'Web';
    default:
      return Platform.OS;
  }
}

export function buildVersion(): string {
  const iosBuildNumber = Constants.expoConfig?.ios?.buildNumber;
  if (iosBuildNumber) {
    return String(iosBuildNumber);
  }

  const androidVersionCode = Constants.expoConfig?.android?.versionCode;
  if (androidVersionCode !== undefined) {
    return String(androidVersionCode);
  }

  const runtimeVersion = Constants.expoConfig?.runtimeVersion;
  return typeof runtimeVersion === 'string' ? runtimeVersion : 'dev';
}
