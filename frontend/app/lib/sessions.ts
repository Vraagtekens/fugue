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
  audioUrl: string;
  pdfUrl: string;
};

export type SessionsResponse = {
  backendAvailable: boolean;
  backendError: string | null;
  liveWebSocketUrl: string;
  sessions: PianoSession[];
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
    liveWebSocketUrl: getLiveWebSocketUrl(),
    sessions: [...backendResult.sessions, ...localSessions].sort((a, b) => {
      const aTime = Date.parse(a.startedAt ?? a.createdAt ?? "");
      const bTime = Date.parse(b.startedAt ?? b.createdAt ?? "");
      return (Number.isNaN(bTime) ? 0 : bTime) - (Number.isNaN(aTime) ? 0 : aTime);
    }),
  };
}

export async function deleteSession(session: Pick<PianoSession, "id" | "source" | "key">) {
  if (session.source === "backend") {
    if (!backendApiKey) {
      return new Response("Missing FUGUE_API_KEY", { status: 503 });
    }

    const id = session.id.replace(/^backend:/, "");
    if (!/^\d+$/.test(id)) {
      return new Response("Invalid backend session ID", { status: 400 });
    }

    const response = await fetch(`${backendBaseUrl}/sessions/${id}`, {
      method: "DELETE",
      headers: { "x-api-key": backendApiKey },
      cache: "no-store",
    });

    if (!response.ok) {
      return new Response(await response.text(), { status: response.status });
    }
    return new Response(null, { status: 204 });
  }

  const sessionsDir = path.resolve(process.cwd(), "..", "recorder", "sessions");
  const midiPath = path.resolve(sessionsDir, session.key);
  if (!midiPath.startsWith(`${sessionsDir}${path.sep}`) || !/\.(midi|mid)$/i.test(midiPath)) {
    return new Response("Invalid local MIDI path", { status: 400 });
  }

  try {
    await fs.unlink(midiPath);
    return new Response(null, { status: 204 });
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") {
      return new Response("Take not found", { status: 404 });
    }
    throw error;
  }
}

export async function fetchBackendFile(kind: "pdf" | "mp3" | "audio", key: string) {
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
      audioUrl: `/api/session-file/audio/${encodeURIComponent(`${sanitizeS3KeyPart(title)}.mid`)}`,
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
    audioUrl: `/api/session-file/audio/${encodeURIComponent(key)}`,
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

function fallbackContentType(kind: "pdf" | "mp3" | "audio") {
  if (kind === "pdf") return "application/pdf";
  return kind === "audio" ? "audio/flac" : "audio/mpeg";
}

function getLiveWebSocketUrl() {
  const url = new URL(backendBaseUrl);
  url.protocol = url.protocol === "https:" ? "wss:" : "ws:";
  url.pathname = `${url.pathname.replace(/\/$/, "")}/sessions/live/subscribe`;
  url.search = "";
  return url.toString();
}
