#!/usr/bin/env sh
set -eu

# Keep local tag refs up to date when running in CI.
git fetch --tags --force >/dev/null 2>&1 || true

EXACT_TAG=$(git describe --tags --exact-match 2>/dev/null || true)
LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || true)
SHORT_SHA=$(git rev-parse --short=8 HEAD)

if [ -n "$EXACT_TAG" ]; then
  printf '%s\n' "$EXACT_TAG"
  exit 0
fi

if [ -z "$LAST_TAG" ]; then
  LAST_TAG='v0.0.0'
fi

printf '%s_%s\n' "$LAST_TAG" "$SHORT_SHA"