#!/usr/bin/env bash
set -euo pipefail

export DATABASE_URL="sqlite://../frontend/db/app.db"

echo "Using DATABASE_URL: $DATABASE_URL"

# Run migration
sea-orm-cli migrate
