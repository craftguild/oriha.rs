// Preview lifecycle diagnostics and error message formatting.

export function debugLogIfEnabled(
  enabled: boolean,
  event: string,
  payload?: Record<string, unknown>,
): void {
  if (enabled) {
    debugLog(event, payload);
  }
}

export function debugLog(event: string, payload?: Record<string, unknown>): void {
  console.debug("[OrihaPreview]", {
    event,
    timestamp: new Date().toISOString(),
    ...payload,
  });
}

export function formatDebugError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export function formatError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
