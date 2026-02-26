#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"
DEKA_DOCS_OUT="${DEKA_DOCS_OUT:-${REPO_ROOT}/../deka-website/content/docs}"

publish_docs() {
  if [[ "${DEKA_TEST_SKIP_DOCS:-0}" == "1" ]]; then
    echo "[docs] skipped (DEKA_TEST_SKIP_DOCS=1)"
    return
  fi
  if [[ ! -f "${REPO_ROOT}/scripts/publish-docs.js" ]]; then
    return
  fi
  if ! command -v node >/dev/null 2>&1; then
    echo "[docs] node is required to publish docs during tests" >&2
    exit 1
  fi
  echo "[docs] publishing docs -> ${DEKA_DOCS_OUT}"
  node "${REPO_ROOT}/scripts/publish-docs.js" --manual "${REPO_ROOT}/docs/phpx" --scan "${REPO_ROOT}" --sections phpx --version latest --out "${DEKA_DOCS_OUT}" --force
}

if ! command -v docker >/dev/null 2>&1; then
  echo "docker is required for DB e2e tests"
  exit 1
fi

if ! command -v bun >/dev/null 2>&1; then
  echo "bun is required for PHPX fixture runner"
  exit 1
fi

POSTGRES_CONTAINER="${POSTGRES_CONTAINER:-deka-phpx-db-e2e-postgres}"
DB_HOST="${DB_HOST:-127.0.0.1}"
DB_PORT="${DB_PORT:-55432}"
DB_NAME="${DB_NAME:-linkhash_registry}"
DB_USER="${DB_USER:-postgres}"
DB_PASSWORD="${DB_PASSWORD:-postgres}"
PHPX_BIN="${PHPX_BIN:-target/release/cli}"
PHPX_BIN_ARGS="${PHPX_BIN_ARGS:-}"

cleanup() {
  docker rm -f "$POSTGRES_CONTAINER" >/dev/null 2>&1 || true
}
trap cleanup EXIT

docker rm -f "$POSTGRES_CONTAINER" >/dev/null 2>&1 || true

echo "[db-e2e] starting postgres container: $POSTGRES_CONTAINER"
HOST_PORT="$DB_PORT"
STARTED=0
for _ in $(seq 1 5); do
  set +e
  run_out="$(docker run -d \
    --name "$POSTGRES_CONTAINER" \
    -e POSTGRES_PASSWORD="$DB_PASSWORD" \
    -e POSTGRES_DB="$DB_NAME" \
    -p "$HOST_PORT:5432" \
    postgres:16-alpine 2>&1)"
  run_code=$?
  set -e
  if [ $run_code -eq 0 ]; then
    STARTED=1
    DB_PORT="$HOST_PORT"
    break
  fi
  if echo "$run_out" | grep -q "port is already allocated"; then
    HOST_PORT=$((HOST_PORT + 1))
    docker rm -f "$POSTGRES_CONTAINER" >/dev/null 2>&1 || true
    continue
  fi
  echo "$run_out"
  exit 1
done

if [ $STARTED -ne 1 ]; then
  echo "[db-e2e] failed to start postgres container after port retries"
  exit 1
fi

echo "[db-e2e] waiting for postgres readiness"
for _ in $(seq 1 60); do
  if docker exec "$POSTGRES_CONTAINER" pg_isready -U "$DB_USER" -d "$DB_NAME" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

if ! docker exec "$POSTGRES_CONTAINER" pg_isready -U "$DB_USER" -d "$DB_NAME" >/dev/null 2>&1; then
  echo "[db-e2e] postgres did not become ready"
  exit 1
fi

echo "[db-e2e] seeding minimal postgres schema"
docker exec "$POSTGRES_CONTAINER" psql -U "$DB_USER" -d "$DB_NAME" -c \
  "CREATE TABLE IF NOT EXISTS packages(id SERIAL PRIMARY KEY, name TEXT NOT NULL);" >/dev/null

if [ ! -x "$PHPX_BIN" ]; then
  echo "[db-e2e] release cli missing at $PHPX_BIN; building"
  cargo build --release -p cli >/dev/null
fi

echo "[db-e2e] running PHPX db fixture suite (postgres + sqlite)"
PHPX_DB_SMOKE=1 \
DB_HOST="$DB_HOST" \
DB_PORT="$DB_PORT" \
DB_NAME="$DB_NAME" \
DB_USER="$DB_USER" \
DB_PASSWORD="$DB_PASSWORD" \
PHPX_BIN="$PHPX_BIN" \
PHPX_BIN_ARGS="$PHPX_BIN_ARGS" \
bun tests/phpx/testrunner.js tests/phpx/db \
  --skip=tests/phpx/db/mysql_smoke.phpx,tests/phpx/db/async_mysql_smoke.phpx,tests/phpx/db/wire_mysql_smoke.phpx,tests/phpx/db/wire_mysql_param_type_error.phpx,tests/phpx/db/contract_mysql.phpx

publish_docs

echo "[db-e2e] complete"
