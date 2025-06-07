#!/usr/bin/env bash
set -euo pipefail

REPO_DIR="$(cd "$(dirname "$0")/.." && pwd)"

# Build Docker image
PYTEST_ARGS="${*:-e2e --verbose}"

# Run tests inside container
docker run --rm -v "$REPO_DIR":/evi -e "PYTEST_ARGS=${PYTEST_ARGS}" evi-e2e
