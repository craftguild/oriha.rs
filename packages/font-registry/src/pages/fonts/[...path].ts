import { readFile, stat } from "node:fs/promises";
import path from "node:path";

const GENERATED_ROOT = path.resolve(process.cwd(), "generated");
const FONT_ROOT = path.join(GENERATED_ROOT, "fonts");

export async function GET({ params }: { params: { path?: string } }): Promise<Response> {
  const requestedPath = params.path ?? "";
  const filePath = resolveFontPath(requestedPath);

  if (!filePath) {
    return new Response("Invalid font path.\n", {
      status: 400,
      headers: {
        "content-type": "text/plain; charset=utf-8",
      },
    });
  }

  try {
    const [payload, fileStat] = await Promise.all([
      readFile(filePath),
      stat(filePath),
    ]);

    return new Response(payload, {
      headers: {
        "access-control-allow-origin": "*",
        "cache-control": "public, max-age=31536000, immutable",
        "content-length": String(fileStat.size),
        "content-type": contentTypeForFont(filePath),
      },
    });
  } catch (error) {
    if (isNotFoundError(error)) {
      return new Response("Font file not found.\n", {
        status: 404,
        headers: {
          "content-type": "text/plain; charset=utf-8",
        },
      });
    }
    throw error;
  }
}

function resolveFontPath(requestedPath: string): string | null {
  const normalized = path.normalize(requestedPath);
  if (
    normalized === "." ||
    normalized.startsWith("..") ||
    path.isAbsolute(normalized) ||
    normalized.includes(`..${path.sep}`)
  ) {
    return null;
  }

  const filePath = path.join(FONT_ROOT, normalized);
  const relative = path.relative(FONT_ROOT, filePath);
  if (relative.startsWith("..") || path.isAbsolute(relative)) {
    return null;
  }
  return filePath;
}

function contentTypeForFont(filePath: string): string {
  const extension = path.extname(filePath).toLowerCase();
  if (extension === ".woff2") {
    return "font/woff2";
  }
  if (extension === ".woff") {
    return "font/woff";
  }
  if (extension === ".ttf") {
    return "font/ttf";
  }
  if (extension === ".otf") {
    return "font/otf";
  }
  return "application/octet-stream";
}

function isNotFoundError(error: unknown): boolean {
  return (
    error instanceof Error &&
    "code" in error &&
    (error as NodeJS.ErrnoException).code === "ENOENT"
  );
}
