import "server-only";

export type PianoSession = {
  id: string;
  title: string;
  key: string;
  startedAt: string | null;
  endedAt: string | null;
  createdAt: string | null;
  favorite: boolean;
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
  favorite: boolean;
};

const backendBaseUrl = process.env.FUGUE_API_BASE_URL ?? "http://localhost:3000";
const backendApiKey = process.env.FUGUE_API_KEY ?? process.env.API_KEY;
const fileCache = new Map<string, Promise<{ buffer: ArrayBuffer; contentType: string }>>();

export async function getSessions() {
  const backendResult = await getBackendSessions();

  return {
    backendAvailable: backendResult.ok,
    backendError: backendResult.error,
    liveWebSocketUrl: getLiveWebSocketUrl(),
    sessions: backendResult.sessions.sort((a, b) => {
      const aTime = Date.parse(a.startedAt ?? a.createdAt ?? "");
      const bTime = Date.parse(b.startedAt ?? b.createdAt ?? "");
      return (Number.isNaN(bTime) ? 0 : bTime) - (Number.isNaN(aTime) ? 0 : aTime);
    }),
  };
}

export async function deleteSession(sessionId: string) {
  return mutateBackendSession(sessionId, "DELETE");
}

export async function updateFavorite(sessionId: string, favorite: boolean) {
  return mutateBackendSession(sessionId, "PATCH", favorite);
}

async function mutateBackendSession(sessionId: string, method: "DELETE" | "PATCH", favorite?: boolean) {
  if (!backendApiKey) {
    return new Response("Missing FUGUE_API_KEY", { status: 503 });
  }

  const id = sessionId.replace(/^backend:/, "");
  if (!/^\d+$/.test(id)) {
    return new Response("Invalid backend session ID", { status: 400 });
  }

  const response = await fetch(
    `${backendBaseUrl}/sessions/${id}${method === "PATCH" ? "/favorite" : ""}`,
    {
      method,
      headers: {
        "x-api-key": backendApiKey,
        ...(method === "PATCH" ? { "content-type": "application/json" } : {}),
      },
      body: method === "PATCH" ? JSON.stringify({ favorite }) : undefined,
      cache: "no-store",
    },
  );
  if (!response.ok) {
    return new Response(await response.text(), { status: response.status });
  }
  return method === "DELETE"
    ? new Response(null, { status: 204 })
    : Response.json(await response.json());
}

export async function fetchBackendFile(
  kind: "pdf" | "mp3" | "audio",
  key: string,
  rangeHeader?: string | null,
  headOnly = false,
) {
  if (!backendApiKey) {
    return new Response("Missing FUGUE_API_KEY", { status: 503 });
  }

  if (kind === "audio" || kind === "pdf") {
    const cacheKey = `${kind}:${key}`;
    let file = fileCache.get(cacheKey);
    if (!file) {
      file = fetch(`${backendBaseUrl}/sessions/${kind}/${encodeURIComponent(key)}`, {
        headers: { "x-api-key": backendApiKey },
        cache: "no-store",
      }).then(async (response) => {
        if (!response.ok) throw new Error(await response.text());
        return {
          buffer: await response.arrayBuffer(),
          contentType: response.headers.get("content-type") ?? fallbackContentType(kind),
        };
      });
      fileCache.set(cacheKey, file);
      file.catch(() => fileCache.delete(cacheKey));
    }

    try {
      const { buffer, contentType } = await file;
      return rangedFileResponse(buffer, contentType, rangeHeader, headOnly);
    } catch (error) {
      return new Response(error instanceof Error ? error.message : "File generation failed", {
        status: 502,
      });
    }
  }

  const response = await fetch(`${backendBaseUrl}/sessions/${kind}/${encodeURIComponent(key)}`, {
    headers: { "x-api-key": backendApiKey },
    cache: "no-store",
  });

  return new Response(response.body, {
    status: response.status,
    headers: {
      "content-type": response.headers.get("content-type") ?? fallbackContentType(kind),
      "cache-control": "no-store",
    },
  });
}

function rangedFileResponse(
  buffer: ArrayBuffer,
  contentType: string,
  rangeHeader?: string | null,
  headOnly = false,
) {
  const size = buffer.byteLength;
  const commonHeaders = {
    "accept-ranges": "bytes",
    "cache-control": "private, max-age=3600",
    "content-type": contentType,
  };
  if (!rangeHeader) {
    return new Response(headOnly ? null : buffer.slice(0), {
      status: 200,
      headers: { ...commonHeaders, "content-length": String(size) },
    });
  }

  const range = parseByteRange(rangeHeader, size);
  if (!range) {
    return new Response(null, {
      status: 416,
      headers: { ...commonHeaders, "content-range": `bytes */${size}` },
    });
  }
  const [start, end] = range;
  return new Response(headOnly ? null : buffer.slice(start, end + 1), {
    status: 206,
    headers: {
      ...commonHeaders,
      "content-length": String(end - start + 1),
      "content-range": `bytes ${start}-${end}/${size}`,
    },
  });
}

function parseByteRange(header: string, size: number): [number, number] | null {
  const match = /^bytes=(\d*)-(\d*)$/.exec(header.trim());
  if (!match || (!match[1] && !match[2]) || size === 0) return null;

  if (!match[1]) {
    const suffixLength = Number(match[2]);
    if (!Number.isSafeInteger(suffixLength) || suffixLength <= 0) return null;
    return [Math.max(0, size - suffixLength), size - 1];
  }

  const start = Number(match[1]);
  const requestedEnd = match[2] ? Number(match[2]) : size - 1;
  if (!Number.isSafeInteger(start) || !Number.isSafeInteger(requestedEnd) || start >= size) {
    return null;
  }
  const end = Math.min(requestedEnd, size - 1);
  return end < start ? null : [start, end];
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

function toPianoSession(session: BackendSession): PianoSession {
  const key = `${sanitizeS3KeyPart(session.title)}.mid`;

  return {
    id: `backend:${session.id}`,
    title: session.title,
    key,
    startedAt: session.start_time,
    endedAt: session.end_time,
    createdAt: session.created_at,
    favorite: session.favorite ?? false,
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

function fallbackContentType(kind: "pdf" | "mp3" | "audio") {
  if (kind === "pdf") return "application/pdf";
  return kind === "audio" ? "audio/wav" : "audio/mpeg";
}

function getLiveWebSocketUrl() {
  const url = new URL(backendBaseUrl);
  url.protocol = url.protocol === "https:" ? "wss:" : "ws:";
  url.pathname = `${url.pathname.replace(/\/$/, "")}/sessions/live/subscribe`;
  url.search = "";
  return url.toString();
}
