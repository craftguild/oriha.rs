export function stableSerialize(value: unknown): string {
  return JSON.stringify(stableSerializableValue(value));
}

function stableSerializableValue(value: unknown): unknown {
  if (Array.isArray(value)) {
    return value.map(stableSerializableValue);
  }

  if (value !== null && typeof value === "object") {
    const sortedValue: Record<string, unknown> = {};

    for (const [key, childValue] of Object.entries(value).sort(
      ([left], [right]) => left.localeCompare(right),
    )) {
      if (childValue !== undefined) {
        sortedValue[key] = stableSerializableValue(childValue);
      }
    }

    return sortedValue;
  }

  return value;
}
