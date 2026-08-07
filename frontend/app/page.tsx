import { SessionBrowser } from "@/app/components/SessionBrowser";
import { getSessions } from "@/app/lib/sessions";

export default async function Home() {
  const data = await getSessions();
  return <SessionBrowser initialData={data} />;
}
