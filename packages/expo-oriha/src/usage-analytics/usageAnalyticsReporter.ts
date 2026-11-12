import * as Crypto from "expo-crypto";

import type {
  RenderStats,
  UsageAnalyticsOutboxConfig,
  UsageAnalyticsRenderPayloadInput,
} from "../types";
import {
  UsageAnalyticsOutbox,
  createUsageAnalyticsOutbox,
  type CreateUsageAnalyticsOutboxOptions,
} from "./usageAnalyticsOutbox";

export type UsageAnalyticsReporterContext = Omit<
  UsageAnalyticsRenderPayloadInput,
  "stats" | "eventId"
>;

export type UsageAnalyticsReporterOptions = CreateUsageAnalyticsOutboxOptions & {
  outbox?: UsageAnalyticsOutbox;
};

export type UsageAnalyticsRenderCompleteInput = {
  stats: RenderStats;
  eventId?: string;
  context?: Partial<UsageAnalyticsReporterContext>;
};

export function createUsageAnalyticsReporter(
  outboxConfig: UsageAnalyticsOutboxConfig,
  context: UsageAnalyticsReporterContext,
  options?: UsageAnalyticsReporterOptions,
): UsageAnalyticsReporter {
  return new UsageAnalyticsReporter(outboxConfig, context, options);
}

export class UsageAnalyticsReporter {
  readonly outbox: UsageAnalyticsOutbox;

  private context: UsageAnalyticsReporterContext;

  constructor(
    outboxConfig: UsageAnalyticsOutboxConfig,
    context: UsageAnalyticsReporterContext,
    options?: UsageAnalyticsReporterOptions,
  ) {
    this.context = context;
    this.outbox =
      options?.outbox ?? createUsageAnalyticsOutbox(outboxConfig, options);
  }

  updateContext(context: Partial<UsageAnalyticsReporterContext>): void {
    this.context = {
      ...this.context,
      ...context,
    };
  }

  setDeliveryEnabled(isEnabled: boolean): Promise<void> {
    return this.outbox.setDeliveryEnabled(isEnabled);
  }

  drain(): Promise<void> {
    return this.outbox.drain();
  }

  purge(): Promise<void> {
    return this.outbox.purge();
  }

  async recordRenderComplete(
    input: UsageAnalyticsRenderCompleteInput,
  ): Promise<void> {
    if (!this.outbox.isDeliveryEnabled) {
      return;
    }

    await this.outbox.enqueue({
      ...this.context,
      ...input.context,
      stats: input.stats,
      eventId: input.eventId ?? makeUsageAnalyticsEventId(),
    });
  }
}

export function makeUsageAnalyticsEventId(): string {
  const expoCrypto = Crypto as {
    randomUUID?: () => string;
    getRandomBytes?: (byteCount: number) => Uint8Array;
  };

  if (typeof expoCrypto.randomUUID === "function") {
    return expoCrypto.randomUUID();
  }

  const globalCrypto = globalThis.crypto as
    | {
        randomUUID?: () => string;
        getRandomValues?: (bytes: Uint8Array) => Uint8Array;
      }
    | undefined;

  if (typeof globalCrypto?.randomUUID === "function") {
    return globalCrypto.randomUUID();
  }

  const bytes =
    typeof expoCrypto.getRandomBytes === "function"
      ? expoCrypto.getRandomBytes(16)
      : globalCrypto?.getRandomValues?.(new Uint8Array(16));
  if (!bytes) {
    throw new Error("A UUID v4 random source is not available");
  }

  bytes[6] = (bytes[6] & 0x0f) | 0x40;
  bytes[8] = (bytes[8] & 0x3f) | 0x80;
  return bytesToUuid(bytes);
}

function bytesToUuid(bytes: Uint8Array): string {
  const hex = [...bytes].map((byte) => byte.toString(16).padStart(2, "0"));
  return [
    hex.slice(0, 4).join(""),
    hex.slice(4, 6).join(""),
    hex.slice(6, 8).join(""),
    hex.slice(8, 10).join(""),
    hex.slice(10, 16).join(""),
  ].join("-");
}
