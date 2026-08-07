"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { SessionsResponse } from "@/app/lib/sessions";

type MidiNote = {
  start: number;
  duration: number;
  note: number;
  velocity: number;
};

type LiveSessionEvent =
  | { type: "session_started"; session_id: string; title: string; started_at: string }
  | { type: "midi_event"; session_id: string; timestamp_ms: number; bytes: number[] }
  | { type: "session_finished"; session_id: string; ended_at: string };

type LiveState = {
  connected: boolean;
  recording: boolean;
  sessionId: string | null;
  title: string | null;
  noteCount: number;
};

export function SessionBrowser({ initialData }: { initialData: SessionsResponse }) {
  const [data, setData] = useState(initialData);
  const [selectedId, setSelectedId] = useState(initialData.sessions[0]?.id ?? "");
  const [query, setQuery] = useState("");
  const [isPlaying, setIsPlaying] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);
  const [deleteError, setDeleteError] = useState<string | null>(null);
  const [live, setLive] = useState<LiveState>({
    connected: false,
    recording: false,
    sessionId: null,
    title: null,
    noteCount: 0,
  });
  const audioRef = useRef<HTMLAudioElement | null>(null);
  const synthRef = useRef<{ context: AudioContext; stop: () => void } | null>(null);

  const refreshSessions = useCallback(() => {
    return fetch("/api/sessions")
      .then((response) => response.json())
      .then((nextData: SessionsResponse) => {
        setData(nextData);
        setSelectedId((current) => current || nextData.sessions[0]?.id || "");
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
              noteCount: 0,
            });
          } else if (event.type === "midi_event") {
            setLive((current) => ({
              connected: true,
              recording: true,
              sessionId: event.session_id,
              title: current.sessionId === event.session_id ? current.title : event.session_id,
              noteCount: current.sessionId === event.session_id ? current.noteCount + 1 : 1,
            }));
          } else if (event.type === "session_finished") {
            setLive((current) => ({ ...current, recording: false }));
            refreshTimer = window.setTimeout(() => void refreshSessions(), 1200);
          }
        } catch {
          // Ignore malformed live events and keep the connection open.
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
    return data.sessions.filter((session) => {
      return `${session.title} ${session.key}`.toLowerCase().includes(needle);
    });
  }, [data.sessions, query]);

  const selected = data.sessions.find((session) => session.id === selectedId) ?? sessions[0];
  const backendCount = data.sessions.filter((session) => session.source === "backend").length;
  const localCount = data.sessions.filter((session) => session.source === "local").length;

  async function playSelected() {
    if (!selected?.midiUrl || isPlaying) return;

    setIsPlaying(true);
    stopPlayback(false);

    await playMidiPreview(selected.midiUrl);
  }

  function stopPlayback(reset = true) {
    audioRef.current?.pause();
    if (audioRef.current) audioRef.current.currentTime = 0;
    synthRef.current?.stop();
    synthRef.current = null;
    if (reset) setIsPlaying(false);
  }

  async function removeSelected() {
    if (!selected || isDeleting) return;
    if (!window.confirm(`Remove “${selected.title}”? This cannot be undone.`)) return;

    setIsDeleting(true);
    setDeleteError(null);
    stopPlayback();
    try {
      const response = await fetch("/api/sessions", {
        method: "DELETE",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ id: selected.id, source: selected.source, key: selected.key }),
      });
      if (!response.ok) throw new Error((await response.text()) || `Delete failed (${response.status})`);

      const remaining = data.sessions.filter((session) => session.id !== selected.id);
      setData((current) => ({ ...current, sessions: remaining }));
      setSelectedId(remaining[0]?.id ?? "");
    } catch (error) {
      setDeleteError(error instanceof Error ? error.message : "Could not remove take");
    } finally {
      setIsDeleting(false);
    }
  }

  async function playMidiPreview(url: string) {
    try {
      const buffer = await fetch(url).then((response) => response.arrayBuffer());
      const notes = parseMidi(buffer).slice(0, 360);
      const AudioContextClass = window.AudioContext || window.webkitAudioContext;
      const context = new AudioContextClass();
      const gain = context.createGain();
      const scheduled: OscillatorNode[] = [];
      gain.gain.value = 0.12;
      gain.connect(context.destination);

      notes.forEach((note) => {
        const start = context.currentTime + 0.08 + note.start;
        const end = start + Math.min(note.duration, 6);
        const osc = context.createOscillator();
        const noteGain = context.createGain();
        osc.type = "triangle";
        osc.frequency.value = 440 * 2 ** ((note.note - 69) / 12);
        noteGain.gain.setValueAtTime(0, start);
        noteGain.gain.linearRampToValueAtTime(Math.max(0.025, note.velocity * 0.16), start + 0.01);
        noteGain.gain.exponentialRampToValueAtTime(0.001, end);
        osc.connect(noteGain);
        noteGain.connect(gain);
        osc.start(start);
        osc.stop(end + 0.02);
        scheduled.push(osc);
      });

      const duration = Math.min(Math.max(...notes.map((note) => note.start + note.duration), 1), 30);
      const timeout = window.setTimeout(() => {
        context.close();
        synthRef.current = null;
        setIsPlaying(false);
      }, (duration + 0.25) * 1000);

      synthRef.current = {
        context,
        stop: () => {
          window.clearTimeout(timeout);
          scheduled.forEach((osc) => {
            try {
              osc.stop();
            } catch {
              // Already stopped.
            }
          });
          context.close();
        },
      };
    } catch {
      setIsPlaying(false);
    }
  }

  return (
    <main className="min-h-screen bg-[#f7f7f2] text-[#20211d]">
      <section className="mx-auto grid min-h-screen w-full max-w-7xl grid-cols-1 lg:grid-cols-[360px_1fr]">
        <aside className="border-b border-[#d8d8cf] bg-[#fbfbf6] px-5 py-5 lg:min-h-screen lg:border-b-0 lg:border-r">
          <div className="mb-8 flex items-start justify-between gap-4">
            <div>
              <p className="text-[10px] font-semibold uppercase tracking-[0.28em] text-[#77786f]">
                Fugue Archive
              </p>
              <h1 className="mt-2 max-w-[12rem] text-4xl font-semibold leading-none tracking-normal">
                Piano sessions
              </h1>
            </div>
            <div className="text-right">
              <div className="label-seal" aria-hidden="true">
                MIDI
              </div>
              <p className={`mt-2 text-[10px] font-semibold uppercase tracking-[0.14em] ${live.recording ? "text-red-700" : "text-[#77786f]"}`}>
                <span className={`mr-1.5 inline-block size-2 rounded-full ${live.recording ? "animate-pulse bg-red-600" : live.connected ? "bg-emerald-600" : "bg-[#aaa]"}`} />
                {live.recording ? "Recording" : live.connected ? "Recorder ready" : "Live offline"}
              </p>
            </div>
          </div>

          {live.recording ? (
            <div className="mb-5 border border-red-300 bg-red-50 px-3 py-3 text-xs text-red-900">
              <p className="font-semibold uppercase tracking-[0.16em]">Live take</p>
              <p className="mt-1 truncate font-mono">{live.title ?? live.sessionId}</p>
              <p className="mt-1 text-red-700">{live.noteCount} MIDI events received</p>
            </div>
          ) : null}

          <div className="mb-5 grid grid-cols-3 border-y border-[#d8d8cf] text-center text-xs">
            <Stat label="Total" value={data.sessions.length} />
            <Stat label="Cloud" value={backendCount} />
            <Stat label="Local" value={localCount} />
          </div>

          <label className="block text-[10px] font-semibold uppercase tracking-[0.22em] text-[#77786f]">
            Search label
            <input
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              className="mt-2 h-11 w-full border border-[#c9c9bf] bg-white px-3 text-sm font-medium text-[#20211d] outline-none transition focus:border-[#20211d]"
              placeholder="fp30x, 2026, midi..."
            />
          </label>

          {!data.backendAvailable ? (
            <p className="mt-4 border border-[#d7c7b3] bg-[#fff8ee] px-3 py-2 text-xs leading-5 text-[#786045]">
              Backend offline or unconfigured. Showing local recorder files.
            </p>
          ) : null}

          <div className="mt-5 space-y-2">
            {sessions.map((session, index) => (
              <button
                key={session.id}
                onClick={() => {
                  stopPlayback();
                  setSelectedId(session.id);
                }}
                className={`session-row ${selected?.id === session.id ? "session-row-active" : ""}`}
              >
                <span className="font-mono text-[11px] text-[#7f8076]">
                  {String(index + 1).padStart(2, "0")}
                </span>
                <span className="min-w-0 flex-1">
                  <span className="block truncate text-sm font-semibold">{session.title}</span>
                  <span className="block text-[11px] uppercase tracking-[0.16em] text-[#77786f]">
                    {formatDate(session.startedAt)}
                  </span>
                </span>
                <span className="rounded-full border border-[#c9c9bf] px-2 py-1 text-[10px] uppercase tracking-[0.12em]">
                  {session.source}
                </span>
              </button>
            ))}
          </div>
        </aside>

        <section className="px-5 py-5 lg:px-8">
          {selected ? (
            <div className="grid min-h-[calc(100vh-40px)] grid-rows-[auto_1fr_auto] border border-[#c9c9bf] bg-[#fffffb]">
              <header className="grid gap-4 border-b border-[#d8d8cf] p-5 md:grid-cols-[1fr_auto]">
                <div>
                  <p className="text-[10px] font-semibold uppercase tracking-[0.28em] text-[#77786f]">
                    Selected composition
                  </p>
                  <h2 className="mt-2 text-3xl font-semibold leading-tight md:text-5xl">
                    {selected.title}
                  </h2>
                  <p className="mt-3 max-w-2xl text-sm leading-6 text-[#66675f]">
                    Sheet output is generated by the backend from the stored MIDI object.
                    Local recorder files use the browser synth for a fast placeholder listen.
                  </p>
                </div>
                <div className="grid min-w-48 grid-cols-2 border border-[#d8d8cf] text-center text-xs">
                  <Stat label="Source" value={selected.source} />
                  <Stat label="Format" value="MID" />
                </div>
              </header>

              <div className="grid grid-cols-1 lg:grid-cols-[1fr_300px]">
                <div className="min-h-[520px] border-b border-[#d8d8cf] bg-[#f0f0e8] lg:border-b-0 lg:border-r">
                  <iframe
                    key={selected.pdfUrl}
                    title={`${selected.title} sheet music`}
                    src={selected.pdfUrl}
                    className="h-full min-h-[520px] w-full bg-white"
                  />
                </div>
                <div className="flex flex-col justify-between gap-6 p-5">
                  <div className="space-y-5">
                    <ProductBlock label="Catalog key" value={selected.key} />
                    <ProductBlock label="Recorded" value={formatDate(selected.startedAt)} />
                    <ProductBlock label="Playback" value={selected.midiUrl ? "Browser synth" : "Backend MP3"} />
                  </div>

                  <div className="space-y-3">
                    {selected.source === "backend" ? (
                      <div className="border border-[#c9c9bf] bg-[#f7f7f2] p-2">
                        <p className="mb-2 text-[10px] font-semibold uppercase tracking-[0.18em] text-[#77786f]">MP3 player</p>
                        <audio
                          key={selected.audioUrl}
                          ref={audioRef}
                          controls
                          preload="none"
                          src={selected.audioUrl}
                          className="h-10 w-full"
                        />
                      </div>
                    ) : (
                      <button
                        onClick={() => (isPlaying ? stopPlayback() : playSelected())}
                        className="h-12 w-full border border-[#20211d] bg-[#20211d] text-sm font-semibold uppercase tracking-[0.18em] text-white transition hover:bg-[#34352f]"
                      >
                        {isPlaying ? "Stop synth" : "Listen with synth"}
                      </button>
                    )}
                    <a
                      href={selected.pdfUrl}
                      className="flex h-11 items-center justify-center border border-[#c9c9bf] text-sm font-semibold uppercase tracking-[0.16em] transition hover:border-[#20211d]"
                    >
                      PDF
                    </a>
                    <button
                      onClick={removeSelected}
                      disabled={isDeleting}
                      className="h-11 w-full border border-red-300 text-sm font-semibold uppercase tracking-[0.16em] text-red-800 transition hover:border-red-700 disabled:cursor-wait disabled:opacity-50"
                    >
                      {isDeleting ? "Removing…" : "Remove take"}
                    </button>
                    {deleteError ? <p className="text-xs leading-5 text-red-700">{deleteError}</p> : null}
                  </div>
                </div>
              </div>

              <footer className="grid border-t border-[#d8d8cf] text-[10px] uppercase tracking-[0.2em] text-[#77786f] md:grid-cols-3">
                <span className="border-b border-[#d8d8cf] px-4 py-3 md:border-b-0 md:border-r">
                  Store cool and dry
                </span>
                <span className="border-b border-[#d8d8cf] px-4 py-3 md:border-b-0 md:border-r">
                  One take per packet
                </span>
                <span className="px-4 py-3">For listening and notation</span>
              </footer>
            </div>
          ) : (
            <div className="flex min-h-[70vh] items-center justify-center border border-[#c9c9bf] bg-[#fffffb]">
              <p className="text-sm uppercase tracking-[0.2em] text-[#77786f]">
                No sessions found
              </p>
            </div>
          )}
        </section>
      </section>
    </main>
  );
}

function Stat({ label, value }: { label: string; value: string | number }) {
  return (
    <div className="border-r border-[#d8d8cf] px-3 py-3 last:border-r-0">
      <span className="block text-[10px] uppercase tracking-[0.16em] text-[#77786f]">{label}</span>
      <span className="mt-1 block truncate text-sm font-semibold">{value}</span>
    </div>
  );
}

function ProductBlock({ label, value }: { label: string; value: string }) {
  return (
    <div className="border-t border-[#d8d8cf] pt-3">
      <p className="text-[10px] font-semibold uppercase tracking-[0.2em] text-[#77786f]">{label}</p>
      <p className="mt-2 break-words font-mono text-sm leading-6">{value}</p>
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

function parseMidi(buffer: ArrayBuffer): MidiNote[] {
  const view = new DataView(buffer);
  const bytes = new Uint8Array(buffer);
  if (readString(bytes, 0, 4) !== "MThd") return [];

  const division = view.getUint16(12);
  let offset = 14;
  let tempo = 500000;
  const notes: MidiNote[] = [];

  while (offset < bytes.length - 8) {
    if (readString(bytes, offset, 4) !== "MTrk") break;
    const length = view.getUint32(offset + 4);
    offset += 8;
    const end = offset + length;
    let tick = 0;
    let runningStatus = 0;
    const active = new Map<string, { tick: number; velocity: number }>();

    while (offset < end) {
      const delta = readVar(bytes, offset);
      tick += delta.value;
      offset = delta.offset;

      let status = bytes[offset++];
      if (status < 0x80) {
        offset--;
        status = runningStatus;
      } else {
        runningStatus = status;
      }

      if (status === 0xff) {
        const type = bytes[offset++];
        const size = readVar(bytes, offset);
        offset = size.offset;
        if (type === 0x51 && size.value === 3) {
          tempo = (bytes[offset] << 16) | (bytes[offset + 1] << 8) | bytes[offset + 2];
        }
        offset += size.value;
        continue;
      }

      if (status === 0xf0 || status === 0xf7) {
        const size = readVar(bytes, offset);
        offset = size.offset + size.value;
        continue;
      }

      const command = status & 0xf0;
      const channel = status & 0x0f;
      const note = bytes[offset++];
      const velocity = command === 0xc0 || command === 0xd0 ? 0 : bytes[offset++];

      if (command === 0x90 && velocity > 0) {
        active.set(`${channel}:${note}`, { tick, velocity: velocity / 127 });
      } else if (command === 0x80 || command === 0x90) {
        const key = `${channel}:${note}`;
        const started = active.get(key);
        if (started) {
          notes.push({
            start: ticksToSeconds(started.tick, division, tempo),
            duration: Math.max(0.06, ticksToSeconds(tick - started.tick, division, tempo)),
            note,
            velocity: started.velocity,
          });
          active.delete(key);
        }
      }
    }
    offset = end;
  }

  return notes.sort((a, b) => a.start - b.start);
}

function readString(bytes: Uint8Array, offset: number, length: number) {
  return String.fromCharCode(...bytes.slice(offset, offset + length));
}

function readVar(bytes: Uint8Array, offset: number) {
  let value = 0;
  let current = 0;
  do {
    current = bytes[offset++];
    value = (value << 7) | (current & 0x7f);
  } while (current & 0x80);
  return { value, offset };
}

function ticksToSeconds(ticks: number, division: number, tempo: number) {
  return (ticks * tempo) / division / 1_000_000;
}

declare global {
  interface Window {
    webkitAudioContext?: typeof AudioContext;
  }
}
