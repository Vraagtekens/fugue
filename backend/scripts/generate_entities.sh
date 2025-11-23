#!/usr/bin/env bash
set -euo pipefail

# Load environment variables (optional)
if [ -f .env ]; then
  export $(grep -v '^#' .env | xargs)
fi

echo "${DATABASE_URL}"

sea-orm-cli generate entity \
  --database-url "${DATABASE_URL}" \
  --output-dir src/entities \
  --with-serde both \
  # --frontend-format
  # --model-extra-attributes "ts(export)" \
