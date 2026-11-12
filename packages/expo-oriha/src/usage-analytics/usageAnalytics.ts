import ExpoOrihaModule from "../bridge/ExpoOrihaModule";
import {
  parseBridgePayload,
  stringifyBridgePayload,
} from "../bridge/json";
import type {
  UsageAnalyticsOutboxDeliveryStateInput,
  UsageAnalyticsOutboxEnqueueStartInput,
  UsageAnalyticsOutboxResumeInput,
  UsageAnalyticsOutboxStartInput,
  UsageAnalyticsOutboxStep,
} from "../types";

export async function usageAnalyticsOutboxStartEnqueue(
  input: UsageAnalyticsOutboxEnqueueStartInput,
  nowMs: number,
): Promise<UsageAnalyticsOutboxStep> {
  return parseBridgePayload(
    await ExpoOrihaModule.usageAnalyticsOutboxStartEnqueue(
      stringifyBridgePayload(input),
      nowMs,
    ),
  );
}

export async function usageAnalyticsOutboxStartDrain(
  input: UsageAnalyticsOutboxStartInput,
  nowMs: number,
): Promise<UsageAnalyticsOutboxStep> {
  return parseBridgePayload(
    await ExpoOrihaModule.usageAnalyticsOutboxStartDrain(
      stringifyBridgePayload(input),
      nowMs,
    ),
  );
}

export async function usageAnalyticsOutboxStartDeliveryStateChange(
  input: UsageAnalyticsOutboxDeliveryStateInput,
): Promise<UsageAnalyticsOutboxStep> {
  return parseBridgePayload(
    await ExpoOrihaModule.usageAnalyticsOutboxStartDeliveryStateChange(
      stringifyBridgePayload(input),
    ),
  );
}

export async function usageAnalyticsOutboxResume(
  input: UsageAnalyticsOutboxResumeInput,
  nowMs: number,
): Promise<UsageAnalyticsOutboxStep> {
  return parseBridgePayload(
    await ExpoOrihaModule.usageAnalyticsOutboxResume(
      stringifyBridgePayload(input),
      nowMs,
    ),
  );
}
