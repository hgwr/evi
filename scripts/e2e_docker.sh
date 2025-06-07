#!/usr/bin/env bash
set -euo pipefail

REPO_DIR="$(cd "$(dirname "$0")/.." && pwd)"

# Build Docker image
docker build -f "$REPO_DIR/docker/e2e.Dockerfile" -t evi-e2e "$REPO_DIR"

# Run tests inside container
PYTEST_ARGS="${*:-e2e --verbose}"
docker run --rm -v "$REPO_DIR":/evi -e "PYTEST_ARGS=${PYTEST_ARGS}" evi-e2e
