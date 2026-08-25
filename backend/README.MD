# Fugue Backend

Axum API for storing MIDI sessions, rendering sheet music/audio, and serving the
web app with session data.

## Setup

```sh
cp .env.example .env
cargo run
```

For entity generation with SeaORM, point `DATABASE_URL` at your local database:

```sh
sea-orm-cli generate entity \
  --database-url "$DATABASE_URL" \
  --output-dir src/entities \
  --with-serde both
```
