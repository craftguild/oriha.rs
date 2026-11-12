import type {
  UsageAnalyticsOutboxCommand,
  UsageAnalyticsOutboxCommandResult,
  UsageAnalyticsOutboxConfig,
  UsageAnalyticsOutboxDeliveryStateInput,
  UsageAnalyticsOutboxEnqueueStartInput,
  UsageAnalyticsOutboxResumeInput,
  UsageAnalyticsOutboxStartInput,
  UsageAnalyticsOutboxStep,
  UsageAnalyticsRenderPayloadInput,
  UsageAnalyticsStoredEnvelope,
} from "../types";
import type { UsageAnalyticsOutboxStore } from "./usageAnalyticsOutboxStore";
import type { UsageAnalyticsHttpTransport } from "./usageAnalyticsOutboxTransport";

export type UsageAnalyticsOutboxEngine = {
  startEnqueue(
    input: UsageAnalyticsOutboxEnqueueStartInput,
    nowMs: number,
  ): Promise<UsageAnalyticsOutboxStep>;
  startDrain(
    input: UsageAnalyticsOutboxStartInput,
    nowMs: number,
  ): Promise<UsageAnalyticsOutboxStep>;
  startDeliveryStateChange(
    input: UsageAnalyticsOutboxDeliveryStateInput,
  ): Promise<UsageAnalyticsOutboxStep>;
  resume(
    input: UsageAnalyticsOutboxResumeInput,
    nowMs: number,
  ): Promise<UsageAnalyticsOutboxStep>;
};

export type UsageAnalyticsOutboxLogger = {
  debug?: (...messages: unknown[]) => void;
  warn?: (...messages: unknown[]) => void;
};

export type UsageAnalyticsOutboxCoreOptions = {
  config: UsageAnalyticsOutboxConfig;
  store: UsageAnalyticsOutboxStore;
  transport: UsageAnalyticsHttpTransport;
  engine: UsageAnalyticsOutboxEngine;
  isDeliveryEnabled?: boolean;
  nowMs?: () => number;
  logger?: UsageAnalyticsOutboxLogger;
};

export class UsageAnalyticsOutboxCore {
  private deliveryEnabled: boolean;
  private deliveryGeneration = 0;
  private scheduledDrain: ReturnType<typeof setTimeout> | undefined;
  private serialOperation: Promise<void> = Promise.resolve();

  private readonly config: UsageAnalyticsOutboxConfig;
  private readonly store: UsageAnalyticsOutboxStore;
  private readonly transport: UsageAnalyticsHttpTransport;
  private readonly engine: UsageAnalyticsOutboxEngine;
  private readonly nowMs: () => number;
  private readonly logger?: UsageAnalyticsOutboxLogger;

  constructor(options: UsageAnalyticsOutboxCoreOptions) {
    this.config = options.config;
    this.store = options.store;
    this.transport = options.transport;
    this.engine = options.engine;
    this.nowMs = options.nowMs ?? Date.now;
    this.logger = options.logger;
    this.deliveryEnabled = options.isDeliveryEnabled ?? false;
  }

  get isDeliveryEnabled(): boolean {
    return this.deliveryEnabled;
  }

  get generation(): number {
    return this.deliveryGeneration;
  }

  enqueue(renderPayload: UsageAnalyticsRenderPayloadInput): Promise<void> {
    return this.enqueueOperation(async () => {
      if (!this.deliveryEnabled) {
        return;
      }

      const generation = this.deliveryGeneration;
      this.logger?.debug?.(
        "Usage analytics outbox enqueue",
        `generation=${generation}`,
      );
      const step = await this.engine.startEnqueue(
        {
          config: this.config,
          renderPayload,
          isDeliveryEnabled: true,
          deliveryGeneration: generation,
        },
        this.nowMs(),
      );
      await this.runStep(step, generation);
    });
  }

  drain(): Promise<void> {
    return this.enqueueOperation(async () => {
      const generation = this.deliveryGeneration;
      await this.startDrain(generation);
    });
  }

  setDeliveryEnabled(isEnabled: boolean): Promise<void> {
    if (this.deliveryEnabled === isEnabled) {
      return this.serialOperation;
    }

    this.deliveryEnabled = isEnabled;
    this.deliveryGeneration += 1;
    this.cancelScheduledDrain();
    const generation = this.deliveryGeneration;
    this.logger?.debug?.(
      "Usage analytics outbox delivery state changed",
      `enabled=${isEnabled}`,
      `generation=${generation}`,
    );

    return this.enqueueOperation(async () => {
      const step = await this.engine.startDeliveryStateChange({
        isDeliveryEnabled: isEnabled,
        deliveryGeneration: generation,
      });
      await this.runStep(step, generation, { allowDisabled: !isEnabled });
      if (isEnabled) {
        await this.startDrain(generation);
      }
    });
  }

  purge(): Promise<void> {
    this.deliveryEnabled = false;
    this.deliveryGeneration += 1;
    this.cancelScheduledDrain();
    const generation = this.deliveryGeneration;

    return this.enqueueOperation(async () => {
      const step = await this.engine.startDeliveryStateChange({
        isDeliveryEnabled: false,
        deliveryGeneration: generation,
      });
      await this.runStep(step, generation, { allowDisabled: true });
    });
  }

  pendingCount(): Promise<number> {
    return this.store.pendingCount?.() ?? Promise.resolve(0);
  }

  private async runStep(
    initialStep: UsageAnalyticsOutboxStep,
    generation: number,
    options?: { allowDisabled?: boolean },
  ): Promise<void> {
    let step = initialStep;

    while (!step.isComplete) {
      if (!step.stepToken || step.commands.length === 0) {
        return;
      }
      if (!this.canContinue(generation, options)) {
        return;
      }

      const results = await executeUsageAnalyticsOutboxCommands(step.commands, {
        store: this.store,
        transport: this.transport,
        logger: this.logger,
      });
      if (!this.canContinue(generation, options)) {
        return;
      }

      step = await this.engine.resume(
        {
          stepToken: step.stepToken,
          results,
          isDeliveryEnabled: this.deliveryEnabled,
          deliveryGeneration: this.deliveryGeneration,
        },
        this.nowMs(),
      );
    }

    if (this.canContinue(generation, options)) {
      this.scheduleDrain(step.nextDrainDelayMs, generation);
    }
  }

  private async startDrain(generation: number): Promise<void> {
    if (!this.canContinue(generation)) {
      return;
    }

    const step = await this.engine.startDrain(
      {
        config: this.config,
        isDeliveryEnabled: true,
        deliveryGeneration: generation,
      },
      this.nowMs(),
    );
    await this.runStep(step, generation);
  }

  private canContinue(
    generation: number,
    options?: { allowDisabled?: boolean },
  ): boolean {
    return (
      generation === this.deliveryGeneration &&
      (options?.allowDisabled === true || this.deliveryEnabled)
    );
  }

  private scheduleDrain(
    delayMs: number | undefined,
    generation: number,
  ): void {
    if (delayMs === undefined || delayMs === null || !this.deliveryEnabled) {
      return;
    }

    this.cancelScheduledDrain();
    this.scheduledDrain = setTimeout(() => {
      this.scheduledDrain = undefined;
      if (generation === this.deliveryGeneration && this.deliveryEnabled) {
        void this.drain();
      }
    }, Math.max(0, delayMs));
  }

  private cancelScheduledDrain(): void {
    if (this.scheduledDrain !== undefined) {
      clearTimeout(this.scheduledDrain);
      this.scheduledDrain = undefined;
    }
  }

  private enqueueOperation(operation: () => Promise<void>): Promise<void> {
    const next = this.serialOperation.then(operation, operation);
    this.serialOperation = next.catch((error: unknown) => {
      this.logger?.warn?.("Usage analytics outbox operation failed", error);
    });
    return next;
  }
}

export async function executeUsageAnalyticsOutboxCommands(
  commands: UsageAnalyticsOutboxCommand[],
  dependencies: {
    store: UsageAnalyticsOutboxStore;
    transport: UsageAnalyticsHttpTransport;
    logger?: UsageAnalyticsOutboxLogger;
  },
): Promise<UsageAnalyticsOutboxCommandResult[]> {
  const results: UsageAnalyticsOutboxCommandResult[] = [];

  for (const command of commands) {
    results.push(await executeUsageAnalyticsOutboxCommand(command, dependencies));
  }

  return results;
}

async function executeUsageAnalyticsOutboxCommand(
  command: UsageAnalyticsOutboxCommand,
  dependencies: {
    store: UsageAnalyticsOutboxStore;
    transport: UsageAnalyticsHttpTransport;
    logger?: UsageAnalyticsOutboxLogger;
  },
): Promise<UsageAnalyticsOutboxCommandResult> {
  try {
    switch (command.kind) {
      case "loadEnvelopes":
        return commandResult(command, {
          storedEnvelopes: await dependencies.store.loadEnvelopes(),
        });
      case "writeEnvelope":
        if (!command.envelopeId || !command.envelopeJson) {
          return commandResult(command, { succeeded: false });
        }
        await dependencies.store.writeEnvelope({
          id: command.envelopeId,
          envelopeJson: command.envelopeJson,
        });
        dependencies.logger?.debug?.(
          "Usage analytics outbox enqueue",
          `envelopeId=${command.envelopeId}`,
        );
        return commandResult(command);
      case "deleteEnvelope":
        if (!command.envelopeId) {
          return commandResult(command, { succeeded: false });
        }
        await dependencies.store.deleteEnvelope(command.envelopeId);
        return commandResult(command);
      case "purgeEnvelopes":
        await dependencies.store.purgeEnvelopes();
        dependencies.logger?.debug?.("Usage analytics outbox purged");
        return commandResult(command);
      case "sendHttp":
        return sendHttp(command, dependencies.transport, dependencies.logger);
      default:
        return commandResult(command, { succeeded: false });
    }
  } catch (error) {
    dependencies.logger?.debug?.("Usage analytics outbox command failed", error);
    return commandResult(command, { succeeded: false });
  }
}

async function sendHttp(
  command: UsageAnalyticsOutboxCommand,
  transport: UsageAnalyticsHttpTransport,
  logger?: UsageAnalyticsOutboxLogger,
): Promise<UsageAnalyticsOutboxCommandResult> {
  const context = command.httpContext;
  if (!context) {
    return commandResult(command, { succeeded: false });
  }

  logger?.debug?.("Usage analytics outbox send", `requestId=${command.id}`);
  const response = await transport.send(context);
  return commandResult(command, { statusCode: response.statusCode });
}

function commandResult(
  command: UsageAnalyticsOutboxCommand,
  options?: {
    succeeded?: boolean;
    storedEnvelopes?: UsageAnalyticsStoredEnvelope[];
    statusCode?: number;
  },
): UsageAnalyticsOutboxCommandResult {
  return {
    commandId: command.id,
    succeeded: options?.succeeded ?? true,
    storedEnvelopes: options?.storedEnvelopes ?? [],
    statusCode: options?.statusCode,
  };
}
