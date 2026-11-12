import {
  usageAnalyticsOutboxResume,
  usageAnalyticsOutboxStartDeliveryStateChange,
  usageAnalyticsOutboxStartDrain,
  usageAnalyticsOutboxStartEnqueue,
} from "./usageAnalytics";
import {
  UsageAnalyticsOutboxCore,
  executeUsageAnalyticsOutboxCommands,
  type UsageAnalyticsOutboxEngine,
  type UsageAnalyticsOutboxLogger,
} from "./usageAnalyticsOutboxCore";
import {
  createExpoUsageAnalyticsOutboxStore,
  type UsageAnalyticsOutboxFileStoreConfig,
  type UsageAnalyticsOutboxStore,
} from "./usageAnalyticsOutboxStore";
import {
  createFetchUsageAnalyticsTransport,
  type UsageAnalyticsHttpTransport,
} from "./usageAnalyticsOutboxTransport";
import type { UsageAnalyticsOutboxConfig } from "../types";

export type CreateUsageAnalyticsOutboxOptions =
  UsageAnalyticsOutboxFileStoreConfig & {
    store?: UsageAnalyticsOutboxStore;
    transport?: UsageAnalyticsHttpTransport;
    engine?: UsageAnalyticsOutboxEngine;
    isDeliveryEnabled?: boolean;
    nowMs?: () => number;
    logger?: UsageAnalyticsOutboxLogger;
  };

export const nativeUsageAnalyticsOutboxEngine: UsageAnalyticsOutboxEngine = {
  startEnqueue: usageAnalyticsOutboxStartEnqueue,
  startDrain: usageAnalyticsOutboxStartDrain,
  startDeliveryStateChange: usageAnalyticsOutboxStartDeliveryStateChange,
  resume: usageAnalyticsOutboxResume,
};

export function createUsageAnalyticsOutbox(
  config: UsageAnalyticsOutboxConfig,
  options?: CreateUsageAnalyticsOutboxOptions,
): UsageAnalyticsOutbox {
  return new UsageAnalyticsOutbox(config, options);
}

export class UsageAnalyticsOutbox extends UsageAnalyticsOutboxCore {
  constructor(
    config: UsageAnalyticsOutboxConfig,
    options?: CreateUsageAnalyticsOutboxOptions,
  ) {
    super({
      config,
      store:
        options?.store ??
        createExpoUsageAnalyticsOutboxStore({
          rootDirectory: options?.rootDirectory,
          storageNamespace: options?.storageNamespace,
        }),
      transport: options?.transport ?? createFetchUsageAnalyticsTransport(),
      engine: options?.engine ?? nativeUsageAnalyticsOutboxEngine,
      isDeliveryEnabled: options?.isDeliveryEnabled,
      nowMs: options?.nowMs,
      logger: options?.logger,
    });
  }
}

export {
  UsageAnalyticsOutboxCore,
  executeUsageAnalyticsOutboxCommands,
  type UsageAnalyticsOutboxEngine,
  type UsageAnalyticsOutboxLogger,
};
