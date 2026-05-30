#!/usr/bin/env bash
# Build static musl binary using cached Docker image
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

IMAGE_NAME="eureka-cli-builder"
IMAGE_TAG="rust-1.95-musl"

echo "=== Building Docker image (with layer cache) ==="
docker build \
  --target builder \
  --cache-from "$IMAGE_NAME:$IMAGE_TAG" \
  -t "$IMAGE_NAME:$IMAGE_TAG" \
  -f "$PROJECT_ROOT/Dockerfile.musl" \
  "$PROJECT_ROOT"

echo ""
echo "=== Extracting binary from image ==="
CONTAINER_ID=$(docker create "$IMAGE_NAME:$IMAGE_TAG")
docker cp "$CONTAINER_ID:/build/target/x86_64-unknown-linux-musl/release/eureka-cli" \
  "$PROJECT_ROOT/target/x86_64-unknown-linux-musl/release/eureka-cli" 2>/dev/null || {
    mkdir -p "$PROJECT_ROOT/target/x86_64-unknown-linux-musl/release"
    docker cp "$CONTAINER_ID:/build/target/x86_64-unknown-linux-musl/release/eureka-cli" \
      "$PROJECT_ROOT/target/x86_64-unknown-linux-musl/release/eureka-cli"
}
docker rm "$CONTAINER_ID" >/dev/null

echo ""
echo "=== Binary info ==="
ls -lh "$PROJECT_ROOT/target/x86_64-unknown-linux-musl/release/eureka-cli"
file "$PROJECT_ROOT/target/x86_64-unknown-linux-musl/release/eureka-cli"

echo ""
echo "✓ Static musl binary: $PROJECT_ROOT/target/x86_64-unknown-linux-musl/release/eureka-cli"
