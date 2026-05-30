#!/usr/bin/env bash
# Deploy eureka-cli to multiple remote hosts and verify
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

BINARY="${BINARY:-$PROJECT_ROOT/target/x86_64-unknown-linux-musl/release/eureka-cli}"
HOSTS="${HOSTS:-host1.example.com host2.example.com host3.example.com}"
REMOTE_PATH="${REMOTE_PATH:-/usr/local/bin/eureka-cli}"
SSH_USER="${SSH_USER:-root}"

if [ ! -f "$BINARY" ]; then
    echo "Error: Binary not found at $BINARY"
    echo "Run ./scripts/build-musl.sh first"
    exit 1
fi

echo "=== Binary info ==="
ls -lh "$BINARY"
file "$BINARY"
echo ""

SUCCESS=0
FAILED=0
FAILED_HOSTS=()

for host in $HOSTS; do
    echo "=== Deploying to $SSH_USER@$host ==="

    if scp "$BINARY" "$SSH_USER@$host:$REMOTE_PATH" 2>&1; then
        echo "  ✓ Copied to $host:$REMOTE_PATH"

        if ssh "$SSH_USER@$host" "chmod +x $REMOTE_PATH && $REMOTE_PATH --version" 2>&1; then
            echo "  ✓ Verified on $host"
            SUCCESS=$((SUCCESS + 1))
        else
            echo "  ✗ Verification failed on $host"
            FAILED=$((FAILED + 1))
            FAILED_HOSTS+=("$host")
        fi
    else
        echo "  ✗ Failed to copy to $host"
        FAILED=$((FAILED + 1))
        FAILED_HOSTS+=("$host")
    fi
    echo ""
done

echo "============================================================"
echo "DEPLOYMENT SUMMARY: $SUCCESS succeeded, $FAILED failed"
echo "============================================================"
