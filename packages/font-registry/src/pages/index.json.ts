import { readFile, stat } from "node:fs/promises";
import path from "node:path";

const GENERATED_ROOT = path.resolve(process.cwd(), "generated");
const REGISTRY_PATH = path.join(GENERATED_ROOT, "index.json");

export async function GET(): Promise<Response> {
  try {
    const [payload, fileStat] = await Promise.all([
      readFile(REGISTRY_PATH),
      stat(REGISTRY_PATH),
    ]);

    return new Response(payload, {
      headers: {
        "access-control-allow-origin": "*",
        "cache-control": "public, max-age=300, stale-while-revalidate=86400",
        "content-length": String(fileStat.size),
        "content-type": "application/json; charset=utf-8",
      },
    });
  } catch (error) {
    if (isNotFoundError(error)) {
      return new Response("Font registry index is not generated.\n", {
        status: 404,
        headers: {
          "content-type": "text/plain; charset=utf-8",
        },
      });
    }
    throw error;
  }
}

function isNotFoundError(error: unknown): boolean {
  return (
    error instanceof Error &&
    "code" in error &&
    (error as NodeJS.ErrnoException).code === "ENOENT"
  );
}
