"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { PianoSession, SessionsResponse } from "@/app/lib/sessions";

type LiveSessionEvent =
  | { type: "session_started"; session_id: string; title: string; started_at: string }
  | { type: "midi_event"; session_id: string; timestamp_ms: number; bytes: number[] }
  | { type: "session_finished"; session_id: string; ended_at: string };

type LiveState = {
  connected: boolean;
  recording: boolean;
  sessionId: string | null;
  title: string | null;
  eventCount: number;
};

export function SessionBrowser({
  initialData,
  initialSelectedId,
}: {
  initialData: SessionsResponse;
  initialSelectedId?: string;
}) {
  const [data, setData] = useState(initialData);
  const [selectedId, setSelectedId] = useState(
    initialSelectedId ?? initialData.sessions[0]?.id ?? "",
  );
  const [query, setQuery] = useState("");
  const [isDeleting, setIsDeleting] = useState(false);
  const [pendingFavorite, setPendingFavorite] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const [live, setLive] = useState<LiveState>({
    connected: false,
    recording: false,
    sessionId: null,
    title: null,
    eventCount: 0,
  });
  const audioRef = useRef<HTMLAudioElement | null>(null);

  const refreshSessions = useCallback(() => {
    return fetch("/api/sessions")
      .then((response) => response.json())
      .then((nextData: SessionsResponse) => {
        if (nextData.backendAvailable) setData(nextData);
      })
      .catch(() => undefined);
  }, []);

  useEffect(() => {
    void refreshSessions();
  }, [refreshSessions]);

  useEffect(() => {
    let socket: WebSocket | null = null;
    let reconnectTimer: number | null = null;
    let refreshTimer: number | null = null;
    let stopped = false;

    const connect = () => {
      socket = new WebSocket(data.liveWebSocketUrl);
      socket.onopen = () => setLive((current) => ({ ...current, connected: true }));
      socket.onmessage = (message) => {
        try {
          const event = JSON.parse(message.data) as LiveSessionEvent;
          if (event.type === "session_started") {
            setLive({
              connected: true,
              recording: true,
              sessionId: event.session_id,
              title: event.title,
              eventCount: 0,
            });
          } else if (event.type === "midi_event") {
            setLive((current) => ({
              connected: true,
              recording: true,
              sessionId: event.session_id,
              title: current.sessionId === event.session_id ? current.title : event.session_id,
              eventCount: current.sessionId === event.session_id ? current.eventCount + 1 : 1,
            }));
          } else if (event.type === "session_finished") {
            setLive((current) => ({ ...current, recording: false }));
            refreshTimer = window.setTimeout(() => void refreshSessions(), 1200);
          }
        } catch {
          // Ignore malformed events without dropping the live connection.
        }
      };
      socket.onclose = () => {
        setLive((current) => ({ ...current, connected: false }));
        if (!stopped) reconnectTimer = window.setTimeout(connect, 2000);
      };
      socket.onerror = () => socket?.close();
    };

    connect();
    return () => {
      stopped = true;
      if (reconnectTimer !== null) window.clearTimeout(reconnectTimer);
      if (refreshTimer !== null) window.clearTimeout(refreshTimer);
      socket?.close();
    };
  }, [data.liveWebSocketUrl, refreshSessions]);

  const sessions = useMemo(() => {
    const needle = query.trim().toLowerCase();
    if (!needle) return data.sessions;
    return data.sessions.filter((session) =>
      `${session.title} ${session.key}`.toLowerCase().includes(needle),
    );
  }, [data.sessions, query]);

  const favoriteCount = data.sessions.filter((session) => session.favorite).length;
  const selected = data.sessions.find((session) => session.id === selectedId);

  function selectSession(session: PianoSession) {
    audioRef.current?.pause();
    setSelectedId(session.id);
    setActionError(null);
    window.history.replaceState(null, "", `/?take=${encodeURIComponent(session.id)}`);
  }

  async function toggleFavorite(session: PianoSession) {
    if (pendingFavorite) return;
    const favorite = !session.favorite;
    setPendingFavorite(session.id);
    setActionError(null);
    setData((current) => ({
      ...current,
      sessions: current.sessions.map((item) =>
        item.id === session.id ? { ...item, favorite } : item,
      ),
    }));

    try {
      const response = await fetch("/api/sessions", {
        method: "PATCH",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ id: session.id, favorite }),
      });
      if (!response.ok) throw new Error((await response.text()) || "Favorite update failed");
    } catch (error) {
      setData((current) => ({
        ...current,
        sessions: current.sessions.map((item) =>
          item.id === session.id ? { ...item, favorite: !favorite } : item,
        ),
      }));
      setActionError(error instanceof Error ? error.message : "Could not update favorite");
    } finally {
      setPendingFavorite(null);
    }
  }

  async function removeSelected() {
    if (!selected || isDeleting) return;
    if (!window.confirm(`Remove “${selected.title}”? This cannot be undone.`)) return;

    setIsDeleting(true);
    setActionError(null);
    audioRef.current?.pause();
    try {
      const response = await fetch("/api/sessions", {
        method: "DELETE",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ id: selected.id }),
      });
      if (!response.ok) throw new Error((await response.text()) || `Delete failed (${response.status})`);

      const remaining = data.sessions.filter((session) => session.id !== selected.id);
      setData((current) => ({ ...current, sessions: remaining }));
      setSelectedId(remaining[0]?.id ?? "");
    } catch (error) {
      setActionError(error instanceof Error ? error.message : "Could not remove take");
    } finally {
      setIsDeleting(false);
    }
  }

  return (
    <main className="min-h-screen overflow-x-hidden bg-[#f7f7f2] text-[#20211d]">
      <section className="mx-auto grid min-h-screen w-full max-w-7xl grid-cols-1 lg:grid-cols-[320px_minmax(0,1fr)]">
        <section className="min-w-0 p-3 lg:col-start-2 lg:row-start-1 lg:p-6">
          {selected ? (
            <div key={selected.id} className="min-w-0 overflow-hidden border border-[#c9c9bf] bg-[#fffffb]">
              <header className="flex items-start justify-between gap-3 border-b border-[#d8d8cf] p-4 lg:p-5">
                <div className="min-w-0">
                  <p className="text-[10px] font-semibold uppercase tracking-[0.22em] text-[#77786f]">
                    Selected take
                  </p>
                  <h2 className="mt-1 truncate text-2xl font-semibold leading-tight lg:text-4xl">
                    {selected.title}
                  </h2>
                  <p className="mt-1 text-xs text-[#66675f]">{formatDate(selected.startedAt)}</p>
                </div>
                <StarButton
                  favorite={selected.favorite}
                  disabled={pendingFavorite === selected.id}
                  onClick={() => void toggleFavorite(selected)}
                />
              </header>

              <div className="grid min-w-0 grid-cols-1 lg:grid-cols-[minmax(0,1fr)_280px]">
                <div className="order-2 min-w-0 overflow-hidden border-t border-[#d8d8cf] bg-[#e8e8e1] lg:order-1 lg:border-r lg:border-t-0">
                  <object
                    key={`mobile-${selected.pdfUrl}`}
                    data={selected.pdfUrl}
                    type="application/pdf"
                    aria-label={`${selected.title} sheet music`}
                    className="pointer-events-none block h-[70svh] min-h-[480px] w-full bg-white lg:hidden"
                  >
                    <a href={selected.pdfUrl} target="_blank" rel="noreferrer">
                      Open sheet music
                    </a>
                  </object>
                  <iframe
                    key={`desktop-${selected.pdfUrl}`}
                    title={`${selected.title} sheet music`}
                    src={`${selected.pdfUrl}#view=FitH&zoom=page-width`}
                    className="pointer-events-none hidden h-[calc(100vh-48px)] min-h-[520px] w-full max-w-full border-0 bg-white lg:block"
                  />
                </div>

                <div className="order-1 min-w-0 space-y-2 p-3 lg:order-2 lg:p-4">
                  <div className="min-w-0 border border-[#c9c9bf] bg-[#f7f7f2] p-2">
                    <p className="mb-2 text-[10px] font-semibold uppercase tracking-[0.16em] text-[#77786f]">
                      Audio
                    </p>
                    <audio
                      key={selected.audioUrl}
                      ref={audioRef}
                      controls
                      preload="none"
                      src={selected.audioUrl}
                      className="block h-10 w-full min-w-0 max-w-full"
                    />
                  </div>

                  <div className="grid grid-cols-2 gap-2 lg:grid-cols-1">
                    <a
                      href={selected.pdfUrl}
                      target="_blank"
                      rel="noreferrer"
                      className="flex h-10 items-center justify-center border border-[#c9c9bf] text-xs font-semibold uppercase tracking-[0.14em] transition hover:border-[#20211d]"
                    >
                      Open PDF
                    </a>
                    <button
                      type="button"
                      onClick={() => void removeSelected()}
                      disabled={isDeleting}
                      className="h-10 border border-red-300 text-xs font-semibold uppercase tracking-[0.14em] text-red-800 transition hover:border-red-700 disabled:cursor-wait disabled:opacity-50"
                    >
                      {isDeleting ? "Removing…" : "Remove"}
                    </button>
                  </div>
                  {actionError ? <p className="text-xs text-red-700">{actionError}</p> : null}
                </div>
              </div>
            </div>
          ) : (
            <div className="flex min-h-80 items-center justify-center border border-[#c9c9bf] bg-[#fffffb]">
              <p className="text-sm uppercase tracking-[0.2em] text-[#77786f]">No sessions found</p>
            </div>
          )}
        </section>

        <aside className="relative z-10 border-t border-[#d8d8cf] bg-[#fbfbf6] p-3 lg:col-start-1 lg:row-start-1 lg:min-h-screen lg:border-r lg:border-t-0 lg:p-4">
          <div className="flex items-center justify-between gap-3">
            <div>
              <p className="text-[10px] font-semibold uppercase tracking-[0.24em] text-[#77786f]">
                Fugue
              </p>
              <h1 className="mt-1 text-2xl font-semibold leading-none">Takes</h1>
            </div>
            <p className={`text-[10px] font-semibold uppercase tracking-[0.12em] ${live.recording ? "text-red-700" : "text-[#77786f]"}`}>
              <span className={`mr-1.5 inline-block size-2 rounded-full ${live.recording ? "animate-pulse bg-red-600" : live.connected ? "bg-emerald-600" : "bg-[#aaa]"}`} />
              {live.recording ? "Recording" : live.connected ? "Connected" : "Offline"}
            </p>
          </div>

          {live.recording ? (
            <div className="mt-3 border border-red-300 bg-red-50 px-3 py-2 text-xs text-red-900">
              <p className="truncate font-mono">{live.title ?? live.sessionId}</p>
              <p className="mt-0.5 text-red-700">{live.eventCount} events</p>
            </div>
          ) : null}

          <div className="mt-3 grid grid-cols-2 border-y border-[#d8d8cf] text-center text-xs">
            <Stat label="Takes" value={data.sessions.length} />
            <Stat label="Favorites" value={favoriteCount} />
          </div>

          <label className="mt-3 block text-[10px] font-semibold uppercase tracking-[0.18em] text-[#77786f]">
            Search
            <input
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              className="mt-1.5 h-9 w-full border border-[#c9c9bf] bg-white px-3 text-sm text-[#20211d] outline-none focus:border-[#20211d]"
              placeholder="Find a take"
            />
          </label>

          {!data.backendAvailable ? (
            <p className="mt-3 border border-[#d7c7b3] bg-[#fff8ee] px-3 py-2 text-xs text-[#786045]">
              Backend unavailable
            </p>
          ) : null}

          <div className="mt-3 max-h-52 space-y-1.5 overflow-y-auto overscroll-contain lg:hidden">
            {sessions.map((session) => (
              <button
                key={session.id}
                type="button"
                onClick={() => selectSession(session)}
                className={`session-row touch-manipulation ${selected?.id === session.id ? "session-row-active" : ""}`}
              >
                <span className="min-w-0 flex-1">
                  <span className="block truncate text-sm font-semibold">{session.title}</span>
                  <span className="block text-[10px] uppercase tracking-[0.12em] text-[#77786f]">
                    {formatDate(session.startedAt)}
                  </span>
                </span>
                {session.favorite ? <StarIcon filled /> : null}
              </button>
            ))}
          </div>

          <div className="mt-3 hidden space-y-1.5 lg:block">
            {sessions.map((session) => (
              <a
                key={session.id}
                href={`/?take=${encodeURIComponent(session.id)}`}
                onClick={(event) => {
                  event.preventDefault();
                  selectSession(session);
                }}
                className={`session-row touch-manipulation ${selected?.id === session.id ? "session-row-active" : ""}`}
              >
                <span className="min-w-0 flex-1">
                  <span className="block truncate text-sm font-semibold">{session.title}</span>
                  <span className="block text-[10px] uppercase tracking-[0.12em] text-[#77786f]">
                    {formatDate(session.startedAt)}
                  </span>
                </span>
                {session.favorite ? <StarIcon filled /> : null}
              </a>
            ))}
          </div>
        </aside>
      </section>
    </main>
  );
}

function StarButton({
  favorite,
  disabled,
  onClick,
}: {
  favorite: boolean;
  disabled: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      disabled={disabled}
      aria-label={favorite ? "Remove from favorites" : "Add to favorites"}
      aria-pressed={favorite}
      className="grid size-11 shrink-0 touch-manipulation place-items-center border border-[#c9c9bf] bg-white transition hover:border-[#20211d] disabled:opacity-50"
    >
      <StarIcon filled={favorite} />
    </button>
  );
}

function StarIcon({ filled }: { filled: boolean }) {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true" className="size-5">
      <path
        d="m12 2.8 2.75 5.57 6.15.9-4.45 4.33 1.05 6.12L12 16.83l-5.5 2.89 1.05-6.12L3.1 9.27l6.15-.9L12 2.8Z"
        fill={filled ? "currentColor" : "none"}
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinejoin="round"
      />
    </svg>
  );
}

function Stat({ label, value }: { label: string; value: string | number }) {
  return (
    <div className="border-r border-[#d8d8cf] px-3 py-2 last:border-r-0">
      <span className="block text-[9px] uppercase tracking-[0.14em] text-[#77786f]">{label}</span>
      <span className="mt-0.5 block truncate text-sm font-semibold">{value}</span>
    </div>
  );
}

function formatDate(value: string | null) {
  if (!value) return "Undated";
  return new Intl.DateTimeFormat("en", {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(value));
}
