import fs from "node:fs/promises";
import path from "node:path";

export async function GET(
  _request: Request,
  context: RouteContext<"/api/local-midi/[...path]">,
) {
  const params = await context.params;
  const safeParts = params.path.filter((part) => part !== ".." && !part.includes("/"));
  const sessionsDir = path.join(process.cwd(), "..", "recorder", "sessions");
  const midiPath = path.join(sessionsDir, ...safeParts);

  if (!midiPath.startsWith(sessionsDir) || !/\.(midi|mid)$/i.test(midiPath)) {
    return new Response("Invalid MIDI path", { status: 400 });
  }

  try {
    const file = await fs.readFile(midiPath);
    return new Response(file, {
      headers: {
        "content-type": "audio/midi",
        "cache-control": "public, max-age=60",
      },
    });
  } catch {
    return new Response("MIDI file not found", { status: 404 });
  }
}
