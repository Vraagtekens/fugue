import fs from "node:fs/promises";
import path from "node:path";

export type SessionSource = "backend" | "local";

export type PianoSession = {
  id: string;
  source: SessionSource;
  title: string;
  key: string;
  startedAt: string | null;
  endedAt: string | null;
  createdAt: string | null;
  midiUrl: string | null;
  mp3Url: string;
  pdfUrl: string;
};

type BackendSession = {
  id: number;
  title: string;
  start_time: string;
  end_time: string | null;
  created_at: string | null;
  updated_at: string | null;
};

const backendBaseUrl = process.env.FUGUE_API_BASE_URL ?? "http://localhost:3000";
const backendApiKey = process.env.FUGUE_API_KEY ?? process.env.API_KEY;

export async function getSessions() {
  const [backendResult, localSessions] = await Promise.all([
    getBackendSessions(),
    getLocalSessions(),
  ]);

  return {
    backendAvailable: backendResult.ok,
    backendError: backendResult.error,
    sessions: [...backendResult.sessions, ...localSessions].sort((a, b) => {
      const aTime = Date.parse(a.startedAt ?? a.createdAt ?? "");
      const bTime = Date.parse(b.startedAt ?? b.createdAt ?? "");
      return (Number.isNaN(bTime) ? 0 : bTime) - (Number.isNaN(aTime) ? 0 : aTime);
    }),
  };
}

export async function fetchBackendFile(kind: "pdf" | "mp3", key: string) {
  if (!backendApiKey) {
    return new Response("Missing FUGUE_API_KEY", { status: 503 });
  }

  const response = await fetch(
    `${backendBaseUrl}/sessions/${kind}/${encodeURIComponent(key)}`,
    {
      headers: { "x-api-key": backendApiKey },
      cache: "no-store",
    },
  );

  return new Response(response.body, {
    status: response.status,
    headers: {
      "content-type": response.headers.get("content-type") ?? fallbackContentType(kind),
      "cache-control": "no-store",
    },
  });
}

async function getBackendSessions() {
  if (!backendApiKey) {
    return {
      ok: false,
      error: "Set FUGUE_API_KEY in frontend/.env.local to connect the backend.",
      sessions: [] as PianoSession[],
    };
  }

  try {
    const response = await fetch(`${backendBaseUrl}/sessions`, {
      headers: { "x-api-key": backendApiKey },
      cache: "no-store",
    });

    if (!response.ok) {
      return {
        ok: false,
        error: `Backend returned ${response.status}`,
        sessions: [] as PianoSession[],
      };
    }

    const sessions = (await response.json()) as BackendSession[];
    return {
      ok: true,
      error: null,
      sessions: sessions.map(toPianoSession),
    };
  } catch (error) {
    return {
      ok: false,
      error: error instanceof Error ? error.message : "Backend unavailable",
      sessions: [] as PianoSession[],
    };
  }
}

async function getLocalSessions() {
  const sessionsDir = path.join(process.cwd(), "..", "recorder", "sessions");
  const files = await listMidiFiles(sessionsDir);

  return files.map((file) => {
    const relative = path.relative(sessionsDir, file);
    const name = path.basename(file);
    const date = relative.split(path.sep)[0] ?? null;
    const title = name.replace(/\.(midi|mid)$/i, "");

    return {
      id: `local:${relative}`,
      source: "local" as const,
      title,
      key: relative.split(path.sep).join("/"),
      startedAt: inferDate(title, date),
      endedAt: null,
      createdAt: inferDate(title, date),
      midiUrl: `/api/local-midi/${relative.split(path.sep).map(encodeURIComponent).join("/")}`,
      mp3Url: `/api/session-file/mp3/${encodeURIComponent(`${sanitizeS3KeyPart(title)}.mid`)}`,
      pdfUrl: `/api/session-file/pdf/${encodeURIComponent(`${sanitizeS3KeyPart(title)}.mid`)}`,
    };
  });
}

async function listMidiFiles(dir: string): Promise<string[]> {
  const entries = await fs.readdir(dir, { withFileTypes: true });
  const nested = await Promise.all(
    entries.map(async (entry) => {
      const fullPath = path.join(dir, entry.name);
      if (entry.isDirectory()) return listMidiFiles(fullPath);
      if (entry.isFile() && /\.(midi|mid)$/i.test(entry.name)) return [fullPath];
      return [];
    }),
  );

  return nested.flat();
}

function toPianoSession(session: BackendSession): PianoSession {
  const key = `${sanitizeS3KeyPart(session.title)}.mid`;

  return {
    id: `backend:${session.id}`,
    source: "backend",
    title: session.title,
    key,
    startedAt: session.start_time,
    endedAt: session.end_time,
    createdAt: session.created_at,
    midiUrl: null,
    mp3Url: `/api/session-file/mp3/${encodeURIComponent(key)}`,
    pdfUrl: `/api/session-file/pdf/${encodeURIComponent(key)}`,
  };
}

function sanitizeS3KeyPart(value: string) {
  const sanitized = value
    .split("")
    .map((char) => (/^[A-Za-z0-9_.-]$/.test(char) ? char : "-"))
    .join("")
    .replace(/^-+|-+$/g, "");

  return sanitized || "session";
}

function inferDate(title: string, fallbackDate: string | null) {
  const timestamp = title.match(/(\d{10})/)?.[1];
  if (timestamp) return new Date(Number(timestamp) * 1000).toISOString();
  if (fallbackDate && /^\d{4}-\d{2}-\d{2}$/.test(fallbackDate)) {
    return new Date(`${fallbackDate}T12:00:00.000Z`).toISOString();
  }
  return null;
}

function fallbackContentType(kind: "pdf" | "mp3") {
  return kind === "pdf" ? "application/pdf" : "audio/mpeg";
}
