#!/usr/bin/env bash
# Run functional tests on all deployed hosts
set -euo pipefail

HOSTS="${HOSTS:-host1.example.com host2.example.com host3.example.com}"
SSH_USER="${SSH_USER:-root}"
EUREKA_SERVER="${EUREKA_SERVER:-http://eureka.example.com:8761/eureka}"

PASS=0
FAIL=0
FAILED_TESTS=()

for host in $HOSTS; do
    echo ""
    echo "============================================================"
    echo "Testing: $SSH_USER@$host"
    echo "============================================================"

    if ssh "$SSH_USER@$host" bash <<EOF 2>&1
set -e
SERVER="$EUREKA_SERVER"
TS=\$(date +%s)
ID="test-\$HOSTNAME-\$TS"
APP="TEST-\$HOSTNAME"

echo "=== 1. apps list ==="
eureka-cli --server "\$SERVER" apps list | head -5

echo ""
echo "=== 2. register ==="
eureka-cli --server "\$SERVER" register \\
  --app "\$APP" --instance-id "\$ID" \\
  --hostname testhost --ip 127.0.0.1 --port 9999 \\
  --vip-address test

sleep 2

echo ""
echo "=== 3. verify registration ==="
eureka-cli --server "\$SERVER" apps get "\$APP" | grep "\$ID"

echo ""
echo "=== 4. heartbeat ==="
eureka-cli --server "\$SERVER" heartbeat "\$APP" "\$ID"

echo ""
echo "=== 5. status update ==="
eureka-cli --server "\$SERVER" status set "\$APP" "\$ID" OUT_OF_SERVICE

sleep 1

echo ""
echo "=== 6. metadata update ==="
eureka-cli --server "\$SERVER" metadata set "\$APP" "\$ID" host "\$HOSTNAME"

sleep 1

echo ""
echo "=== 7. deregister ==="
eureka-cli --server "\$SERVER" deregister "\$APP" "\$ID"

echo ""
echo "All tests passed on \$HOSTNAME"
EOF
    then
        echo "  $host: ALL TESTS PASSED"
        PASS=$((PASS + 1))
    else
        echo "  $host: TESTS FAILED"
        FAIL=$((FAIL + 1))
        FAILED_TESTS+=("$host")
    fi
done
