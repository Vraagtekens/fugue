import { SessionBrowser } from "@/app/components/SessionBrowser";
import { getSessions } from "@/app/lib/sessions";

export default async function Home({
  searchParams,
}: {
  searchParams: Promise<{ take?: string | string[] }>;
}) {
  const data = await getSessions();
  const requestedTake = (await searchParams).take;
  const initialSelectedId =
    typeof requestedTake === "string" &&
    data.sessions.some((session) => session.id === requestedTake)
      ? requestedTake
      : data.sessions[0]?.id;

  return (
    <SessionBrowser
      key={initialSelectedId}
      initialData={data}
      initialSelectedId={initialSelectedId}
    />
  );
}
