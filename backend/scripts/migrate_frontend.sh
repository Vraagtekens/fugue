#!/usr/bin/env bash
set -euo pipefail

export DATABASE_URL="sqlite://../frontend/assets/sql/db.db"

echo "Using DATABASE_URL: $DATABASE_URL"

# Run migration
sea-orm-cli migrate
