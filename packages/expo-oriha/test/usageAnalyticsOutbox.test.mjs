import assert from "node:assert/strict";
import test from "node:test";

import {
  UsageAnalyticsOutboxCore,
  executeUsageAnalyticsOutboxCommands,
} from "../src/usage-analytics/usageAnalyticsOutboxCore.ts";

const outboxConfig = {
  ingestUrl: "https://stats.example.test/api/ingest",
  apiKey: "public-write-token",
  requestTimeoutMs: 3000,
  maxQueuedEvents: 50,
  retentionMs: 86_400_000,
};

test("usage analytics command executor bridges storage and HTTP commands", async () => {
  const store = createMemoryOutboxStore();
  const sentRequests = [];
  const transport = {
    async send(context) {
      sentRequests.push(context);
      return { statusCode: 202 };
    },
  };

  const results = await executeUsageAnalyticsOutboxCommands(
    [
      {
        id: "write-envelope:event-1",
        kind: "writeEnvelope",
        envelopeId: "event-1",
        envelopeJson: "{\"ok\":true}",
      },
      {
        id: "load-envelopes",
        kind: "loadEnvelopes",
      },
      {
        id: "send:event-1",
        kind: "sendHttp",
        httpContext: {
          url: outboxConfig.ingestUrl,
          method: "POST",
          timeoutMs: outboxConfig.requestTimeoutMs,
          headers: [{ name: "X-ORIHA-Stats-Key", value: outboxConfig.apiKey }],
          body: "{\"eventId\":\"event-1\"}",
        },
      },
      {
        id: "delete-envelope:event-1",
        kind: "deleteEnvelope",
        envelopeId: "event-1",
      },
    ],
    { store, transport },
  );

  assert.deepEqual(
    results.map((result) => result.succeeded),
    [true, true, true, true],
  );
  assert.deepEqual(results[1].storedEnvelopes, [
    { id: "event-1", envelopeJson: "{\"ok\":true}" },
  ]);
  assert.equal(results[2].statusCode, 202);
  assert.equal(sentRequests.length, 1);
  assert.equal(await store.pendingCount(), 0);
});

test("usage analytics outbox purges queued envelopes when delivery is disabled", async () => {
  const store = createMemoryOutboxStore({
    "event-1": "{\"id\":\"event-1\"}",
  });
  const outbox = new UsageAnalyticsOutboxCore({
    config: outboxConfig,
    store,
    transport: { send: async () => ({ statusCode: 202 }) },
    engine: createPurgeOnlyEngine(),
    isDeliveryEnabled: true,
  });

  await outbox.setDeliveryEnabled(false);
  await outbox.enqueue(createRenderPayload("event-2"));

  assert.equal(outbox.isDeliveryEnabled, false);
  assert.equal(outbox.generation, 1);
  assert.equal(await store.pendingCount(), 0);
});

test("disabling the active outbox stops its scheduled HTTP retries", async (t) => {
  t.mock.timers.enable({ apis: ["setTimeout"] });
  let deliveries = 0;
  const outbox = new UsageAnalyticsOutboxCore({
    config: outboxConfig,
    store: createMemoryOutboxStore(),
    transport: {
      async send() {
        deliveries += 1;
        return { statusCode: 503 };
      },
    },
    engine: {
      async startEnqueue() {
        throw new Error("disabled delivery must not enqueue");
      },
      async startDeliveryStateChange() {
        return completeStep();
      },
      async startDrain() {
        return {
          stepToken: "retry",
          isComplete: false,
          commands: [{
            id: "retry-request",
            kind: "sendHttp",
            httpContext: {
              url: outboxConfig.ingestUrl,
              method: "POST",
              timeoutMs: outboxConfig.requestTimeoutMs,
              headers: [],
              body: "{}",
            },
          }],
        };
      },
      async resume() {
        return { ...completeStep(), nextDrainDelayMs: 10 };
      },
    },
  });

  await outbox.setDeliveryEnabled(true);
  assert.equal(deliveries, 1);
  t.mock.timers.tick(10);
  await outbox.setDeliveryEnabled(true);
  assert.equal(deliveries, 2, "the enabled outbox actually retries");

  await outbox.setDeliveryEnabled(false);
  t.mock.timers.tick(100);
  await outbox.drain();
  await outbox.enqueue(createRenderPayload("disabled-event"));
  assert.equal(deliveries, 2, "no new HTTP delivery starts after opt-out");
});

function createMemoryOutboxStore(seed = {}) {
  const envelopes = new Map(Object.entries(seed));

  return {
    envelopes,
    async loadEnvelopes() {
      return [...envelopes.entries()].map(([id, envelopeJson]) => ({
        id,
        envelopeJson,
      }));
    },
    async writeEnvelope(envelope) {
      envelopes.set(envelope.id, envelope.envelopeJson);
    },
    async deleteEnvelope(id) {
      envelopes.delete(id);
    },
    async purgeEnvelopes() {
      envelopes.clear();
    },
    async pendingCount() {
      return envelopes.size;
    },
  };
}

function createRenderPayload(eventId) {
  return {
    platform: "macOS",
    appVersion: "1.0",
    buildVersion: "1",
    locale: "ja",
    subscriptionPlan: "free",
    stats: {
      schemaVersion: 1,
      totalBlocks: 0,
      blocksByKind: [],
      headingsByLevel: [],
      codeBlocksByLanguageKind: [],
      calloutsByKind: [],
      lockedFeaturesByKind: [],
      unsupportedByKind: [],
      issuesByKind: [],
    },
    eventId,
  };
}

function createPurgeOnlyEngine() {
  return {
    async startEnqueue() {
      throw new Error("enqueue should be gated while delivery is disabled");
    },
    async startDrain() {
      throw new Error("drain should be gated while delivery is disabled");
    },
    async startDeliveryStateChange(input) {
      if (input.isDeliveryEnabled) {
        return completeStep();
      }
      return {
        stepToken: "purge",
        commands: [{ id: "purge-envelopes", kind: "purgeEnvelopes" }],
        isComplete: false,
      };
    },
    async resume(input) {
      assert.equal(input.stepToken, "purge");
      assert.deepEqual(
        input.results.map((result) => result.succeeded),
        [true],
      );
      return completeStep();
    },
  };
}

function completeStep() {
  return {
    commands: [],
    isComplete: true,
  };
}
