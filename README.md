# Fugue

Fugue is a small piano session archive. It records MIDI from a digital piano,
uploads takes to a Rust backend, stores the source `.mid` files in S3-compatible
object storage, and exposes a Next.js web app for browsing sessions, opening
generated sheet music, and listening back.

The project is intentionally local-first: it works well with a laptop, a piano,
Postgres, and MinIO.

## Apps

- `backend` - Axum API for sessions, uploads, auth, S3/MinIO storage, and
  MuseScore/FluidSynth rendering.
- `recorder` - Rust MIDI recorder that listens to a configured MIDI input and
  uploads finished takes.
- `frontend` - Next.js web app for browsing recorded sessions, viewing PDFs,
  and playing generated audio.

## Requirements

- Rust
- Node.js 22.13 or newer
- pnpm
- Postgres
- MinIO or another S3-compatible object store
- MuseScore CLI available as `mscore`
- FluidSynth
- A SoundFont file for audio rendering

## Configuration

Copy the example env files and fill in your local values:

```sh
cp backend/.env.example backend/.env
cp recorder/.env.example recorder/.env
cp frontend/.env.example frontend/.env.local
```

Keep real `.env` files private. They are ignored by Git.

## Backend

```sh
cd backend
cargo run
```

The API defaults to `http://localhost:3000`.

Useful routes:

- `GET /sessions` - list sessions
- `POST /sessions/add` - upload a MIDI session
- `DELETE /sessions/:id` - delete a session and its MIDI object
- `GET /sessions/pdf/:key` - render sheet music from stored MIDI
- `GET /sessions/audio/:key` - render FLAC audio from stored MIDI
- `GET /sessions/live/subscribe` - subscribe to live recorder events

Example upload:

```sh
curl -X POST "http://localhost:3000/sessions/add" \
  -H "x-api-key: $FUGUE_API_KEY" \
  -F 'metadata={"title":"example-take","user_id":"00000000-0000-0000-0000-000000000000","start_time":"2026-01-01T12:00:00Z","end_time":"2026-01-01T12:03:00Z"}' \
  -F "midi_file=@../recorder/sessions/example.mid;type=audio/midi"
```

## Recorder

```sh
cd recorder
cargo run
```

Set `MIDI_PORT_NAME` in `recorder/.env` to a substring of the MIDI input name
printed by the recorder, for example `Roland Digital Piano`.

## Frontend

```sh
cd frontend
pnpm install
pnpm dev
```

The frontend runs on `http://localhost:3001` and proxies backend requests so the
API key stays server-side. If the backend is unavailable, it can still show
placeholder sessions from `recorder/sessions`.

## Public Repo Notes

- Do not commit real `.env` files, database dumps, SoundFonts, or private MIDI
  takes unless you intentionally want to publish them.
- Rotate any credentials that were ever committed before making the repository
  public.
- Prefer placeholder values in examples and docs.
