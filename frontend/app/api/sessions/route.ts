import { deleteSession, getSessions, type PianoSession } from "@/app/lib/sessions";

export async function GET() {
  return Response.json(await getSessions());
}

export async function DELETE(request: Request) {
  const session = (await request.json()) as Partial<Pick<PianoSession, "id" | "source" | "key">>;
  if (!session.id || !session.key || (session.source !== "backend" && session.source !== "local")) {
    return new Response("Invalid session", { status: 400 });
  }

  return deleteSession(session as Pick<PianoSession, "id" | "source" | "key">);
}
