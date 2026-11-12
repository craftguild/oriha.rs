import { defineMiddleware } from "astro:middleware";

const LOGGED_PREFIXES = ["/fonts/"];
const LOGGED_PATHS = new Set(["/", "/index.json"]);

export const onRequest = defineMiddleware(async ({ request, url }, next) => {
  const startedAt = Date.now();

  try {
    const response = await next();
    logAccess(request, url, response.status, Date.now() - startedAt, response);
    return response;
  } catch (error) {
    logAccess(request, url, 500, Date.now() - startedAt, undefined, error);
    throw error;
  }
});

function logAccess(
  request: Request,
  url: URL,
  status: number,
  durationMs: number,
  response?: Response,
  error?: unknown,
): void {
  if (!shouldLog(url.pathname)) {
    return;
  }

  const logEntry: Record<string, unknown> = {
    event: "font_registry_access",
    timestamp: new Date().toISOString(),
    method: request.method,
    path: url.pathname,
    status,
    durationMs,
    userAgent: request.headers.get("user-agent"),
    referer: request.headers.get("referer"),
    vercelId: request.headers.get("x-vercel-id"),
    country: request.headers.get("x-vercel-ip-country"),
    region: request.headers.get("x-vercel-ip-country-region"),
    city: request.headers.get("x-vercel-ip-city"),
    contentLength: response?.headers.get("content-length"),
    cacheControl: response?.headers.get("cache-control"),
  };

  if (error instanceof Error) {
    logEntry.error = error.message;
  }

  console.log(JSON.stringify(logEntry));
}

function shouldLog(pathname: string): boolean {
  return LOGGED_PATHS.has(pathname) || LOGGED_PREFIXES.some((prefix) => pathname.startsWith(prefix));
}
