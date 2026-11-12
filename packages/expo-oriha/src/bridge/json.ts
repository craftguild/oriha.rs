export function stringifyBridgePayload(value: unknown): string {
  const json = JSON.stringify(value);
  if (json === undefined) {
    throw new Error("Cannot serialize undefined native bridge payload");
  }
  return json;
}

export function stringifyOptionalBridgePayload(
  value: unknown | null | undefined,
): string | null {
  return value == null ? null : stringifyBridgePayload(value);
}

export function parseBridgePayload<T>(json: string): T {
  return JSON.parse(json) as T;
}
