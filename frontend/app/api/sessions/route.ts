import { deleteSession, getSessions, updateFavorite } from "@/app/lib/sessions";

export async function GET() {
  return Response.json(await getSessions());
}

export async function DELETE(request: Request) {
  const session = (await request.json()) as { id?: string };
  if (!session.id) {
    return new Response("Invalid session", { status: 400 });
  }

  return deleteSession(session.id);
}

export async function PATCH(request: Request) {
  const session = (await request.json()) as { id?: string; favorite?: boolean };
  if (!session.id || typeof session.favorite !== "boolean") {
    return new Response("Invalid favorite update", { status: 400 });
  }

  return updateFavorite(session.id, session.favorite);
}
