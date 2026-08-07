import { getSessions } from "@/app/lib/sessions";

export async function GET() {
  return Response.json(await getSessions());
}
