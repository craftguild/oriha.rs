import type { SendHttpContext } from "../types";

export type UsageAnalyticsHttpResponse = {
  statusCode?: number;
};

export type UsageAnalyticsHttpTransport = {
  send(context: SendHttpContext): Promise<UsageAnalyticsHttpResponse>;
};

export type UsageAnalyticsFetch = (
  input: string,
  init?: {
    method?: string;
    headers?: Record<string, string>;
    body?: string;
    signal?: AbortSignal;
  },
) => Promise<{ status: number }>;

export type FetchUsageAnalyticsTransportOptions = {
  fetch?: UsageAnalyticsFetch;
};

export function createFetchUsageAnalyticsTransport(
  options?: FetchUsageAnalyticsTransportOptions,
): UsageAnalyticsHttpTransport {
  const fetchImplementation = options?.fetch ?? globalThis.fetch;

  return {
    async send(context) {
      if (!fetchImplementation) {
        return {};
      }

      let timeoutId: ReturnType<typeof setTimeout> | undefined;
      const controller =
        context.timeoutMs > 0 && typeof AbortController !== "undefined"
          ? new AbortController()
          : undefined;

      if (controller) {
        timeoutId = setTimeout(() => controller.abort(), context.timeoutMs);
      }

      try {
        const response = await fetchImplementation(context.url, {
          method: context.method,
          headers: headersFromContext(context),
          body: context.body,
          signal: controller?.signal,
        });
        return { statusCode: response.status };
      } catch {
        return {};
      } finally {
        if (timeoutId !== undefined) {
          clearTimeout(timeoutId);
        }
      }
    },
  };
}

function headersFromContext(context: SendHttpContext): Record<string, string> {
  return Object.fromEntries(
    context.headers.map((header) => [header.name, header.value]),
  );
}
