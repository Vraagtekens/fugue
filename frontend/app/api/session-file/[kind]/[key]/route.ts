import { fetchBackendFile } from "@/app/lib/sessions";

export async function GET(
  request: Request,
  context: RouteContext<"/api/session-file/[kind]/[key]">,
) {
  const { kind, key } = await context.params;

  if (kind !== "pdf" && kind !== "mp3" && kind !== "audio") {
    return new Response("Unsupported file kind", { status: 400 });
  }

  return fetchBackendFile(kind, key, request.headers.get("range"));
}

export async function HEAD(
  request: Request,
  context: RouteContext<"/api/session-file/[kind]/[key]">,
) {
  const { kind, key } = await context.params;
  if (kind !== "audio" && kind !== "pdf") return new Response(null, { status: 405 });
  return fetchBackendFile(kind, key, request.headers.get("range"), true);
}
