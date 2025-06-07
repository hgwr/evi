#!/usr/bin/env bash
set -euo pipefail

REPO_DIR="$(cd "$(dirname "$0")/.." && pwd)"

# Build Docker image
docker build -f "$REPO_DIR/docker/e2e.Dockerfile" -t evi-e2e "$REPO_DIR"

# Run tests inside container
docker run --rm -it -v "$REPO_DIR":/evi evi-e2e
