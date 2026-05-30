#!/usr/bin/env bash
# Compatibility matrix: N client hosts × M Eureka servers
# Each round runs ALL CLI commands (v0.1 lifecycle + v0.2 ops queries).

set -uo pipefail

CLIENTS="${CLIENTS:-host1.example.com host2.example.com host3.example.com}"

# label|url — override via env for your environment, e.g.:
#   SERVERS_OVERRIDE='prod|http://10.0.0.1:8761/eureka,test|http://10.0.0.2:8761/eureka' ./test-matrix.sh
# Falls back to the placeholder set below if SERVERS_OVERRIDE is unset.
if [ -n "${SERVERS_OVERRIDE:-}" ]; then
    IFS=',' read -r -a SERVERS <<< "$SERVERS_OVERRIDE"
else
    SERVERS=(
        "eureka-a|http://192.168.1.100:8761/eureka"
        "eureka-b|http://192.168.1.101:8761/eureka"
        "test-1.10|http://192.168.1.102:8761/eureka"
        "test-2.0|http://192.168.1.102:8762/eureka"
    )
fi

SSH_USER="${SSH_USER:-root}"

declare -A RESULTS
declare -A FAILS
TOTAL_ROUNDS=0
PASSED_ROUNDS=0

for client in $CLIENTS; do
    for entry in "${SERVERS[@]}"; do
        SERVER_LABEL="${entry%%|*}"
        SERVER_URL="${entry##*|}"
        TEST_KEY="$client -> $SERVER_LABEL"
        TOTAL_ROUNDS=$((TOTAL_ROUNDS + 1))

        echo ""
        echo "============================================================"
        echo "[$TOTAL_ROUNDS] $TEST_KEY"
        echo "  Server: $SERVER_URL"
        echo "============================================================"

        OUT=$(ssh -o ConnectTimeout=10 -o BatchMode=yes "$SSH_USER@$client" \
            bash -s "$SERVER_URL" "$client" "$SERVER_LABEL" <<'REMOTE_SCRIPT' 2>&1
SERVER_URL="$1"
CLIENT="$2"
SLABEL="$3"

PASS=0
FAIL=0
FAILED_CMDS=()

run_check() {
    local label="$1"; shift
    local pattern="$1"; shift
    local out rc
    out=$("$@" 2>&1); rc=$?
    if [ $rc -eq 0 ] && grep -qE -- "$pattern" <<<"$out"; then
        echo "  ✓ $label"
        PASS=$((PASS + 1))
    else
        echo "  ✗ $label  (rc=$rc)"
        FAIL=$((FAIL + 1))
        FAILED_CMDS+=("$label")
    fi
}

run_ok() {
    local label="$1"; shift
    local out rc
    out=$("$@" 2>&1); rc=$?
    if [ $rc -eq 0 ]; then
        echo "  ✓ $label"
        PASS=$((PASS + 1))
    else
        echo "  ✗ $label  (rc=$rc)"
        FAIL=$((FAIL + 1))
        FAILED_CMDS+=("$label")
    fi
}

run_404() {
    local label="$1"; shift
    local out rc
    out=$("$@" 2>&1); rc=$?
    if [ $rc -ne 0 ] && grep -qE '(404|Not Found|not found)' <<<"$out"; then
        echo "  ✓ $label"
        PASS=$((PASS + 1))
    else
        echo "  ✗ $label  (rc=$rc, expected 404)"
        FAIL=$((FAIL + 1))
        FAILED_CMDS+=("$label")
    fi
}

TS=$(date +%s)
APP="MATRIX${CLIENT//./}"
ID="m-${CLIENT//./}-${SLABEL//./}-${TS}"

run_check "version"            "eureka-cli"      eureka-cli version
run_ok    "apps list"          eureka-cli --server "$SERVER_URL" apps list
run_check "apps list json"     '"applications"'  eureka-cli --server "$SERVER_URL" --output json apps list

# --- v0.2 read-side (no preconditions, runs against whatever's registered) ---
# Global flags must work both BEFORE and AFTER the subcommand (kubectl-style).
# Without `global = true` on the clap derive struct, "instances ls -l ..."
# fails with "unexpected argument '-l' found". Test both placements.
run_ok    "v02:apps wide"           eureka-cli --server "$SERVER_URL" -o wide apps list
run_ok    "v02:apps wide (post)"    eureka-cli --server "$SERVER_URL" apps list -o wide
run_ok    "v02:instances wide"      eureka-cli --server "$SERVER_URL" -o wide instances list
run_ok    "v02:instances wide (post)" eureka-cli --server "$SERVER_URL" instances list -o wide
run_ok    "v02:selector status=UP"  eureka-cli --server "$SERVER_URL" -l 'status=UP' instances list
run_ok    "v02:selector (post)"     eureka-cli --server "$SERVER_URL" instances list -l 'status=UP'
run_ok    "v02:selector !=UP"       eureka-cli --server "$SERVER_URL" -l 'status!=UP' instances list
run_ok    "v02:sort-by status"      eureka-cli --server "$SERVER_URL" --sort-by status instances list
run_ok    "v02:sort-by (post)"      eureka-cli --server "$SERVER_URL" instances list --sort-by status
run_ok    "v02:apps unhealthy"      eureka-cli --server "$SERVER_URL" apps unhealthy
run_ok    "v02:instances unhealthy" eureka-cli --server "$SERVER_URL" instances unhealthy
run_ok    "v02:jsonpath array"      eureka-cli --server "$SERVER_URL" -o 'jsonpath=$.instances[*].instanceId' instances list
run_ok    "v02:jsonpath (post)"     eureka-cli --server "$SERVER_URL" instances list -o 'jsonpath=$.instances[*].instanceId'
run_check "v02:config ls header"    "Name|name"   eureka-cli config list
run_ok    "v02:completion bash"     eureka-cli completion bash
run_ok    "v02:completion zsh"      eureka-cli completion zsh

run_check "register"           "registered successfully" \
    eureka-cli --server "$SERVER_URL" register \
        --app "$APP" --instance-id "$ID" \
        --hostname "$CLIENT" --ip 127.0.0.1 --port 9999 \
        --vip-address matrix \
        --metadata "client=$CLIENT" --metadata "server=$SLABEL"

echo -n "  waiting for visibility..."
for i in $(seq 1 30); do
    if eureka-cli --server "$SERVER_URL" --output json apps get "$APP" 2>/dev/null | grep -qF "\"$ID\""; then
        echo " ${i}s"
        break
    fi
    sleep 1
done

run_check "apps get"           "\"$ID\""   eureka-cli --server "$SERVER_URL" --output json apps get "$APP"

# --- v0.2 write-then-read: verify selector/describe/jsonpath actually find the instance ---
run_check "v02:apps describe"     "Name:"      eureka-cli --server "$SERVER_URL" apps describe "$APP"
run_check "v02:instance describe" "Identity:"  eureka-cli --server "$SERVER_URL" instances describe -a "$APP" "$ID"

# instances list / unhealthy go through Eureka's /apps cache (~30s). Wait for visibility there.
echo -n "  waiting for /apps cache..."
for i in $(seq 1 45); do
    if eureka-cli --server "$SERVER_URL" --output json instances list 2>/dev/null | grep -qF "\"$ID\""; then
        echo " ${i}s"
        break
    fi
    sleep 1
done

run_check "v02:selector finds new ID" "$ID" \
    eureka-cli --server "$SERVER_URL" -l "app=${APP^^}" -o "jsonpath=\$.instances[*].instanceId" instances list
run_check "v02:metadata selector"     "$ID" \
    eureka-cli --server "$SERVER_URL" -l "metadata.client=$CLIENT" -o "jsonpath=\$.instances[*].instanceId" instances list

run_check "heartbeat"          "Heartbeat sent"  eureka-cli --server "$SERVER_URL" heartbeat "$APP" "$ID"
run_check "status set"         "updated"   eureka-cli --server "$SERVER_URL" status set "$APP" "$ID" OUT_OF_SERVICE
sleep 1
run_check "metadata set"       "updated"   eureka-cli --server "$SERVER_URL" metadata set "$APP" "$ID" testkey testvalue
sleep 1
run_ok    "status remove"      eureka-cli --server "$SERVER_URL" status remove "$APP" "$ID"
run_check "deregister"         "deregistered"  eureka-cli --server "$SERVER_URL" deregister "$APP" "$ID"
sleep 2
run_404   "instance gone"      eureka-cli --server "$SERVER_URL" instances get -a "$APP" "$ID"

echo ""
echo "RESULT_LINE: PASS=$PASS FAIL=$FAIL"
if [ $FAIL -gt 0 ]; then
    echo "FAILED_CMDS_LINE: ${FAILED_CMDS[*]}"
fi
REMOTE_SCRIPT
)

        echo "$OUT"
        RESULT_LINE=$(echo "$OUT" | grep -E "^RESULT_LINE:" | tail -1)

        if [ -z "$RESULT_LINE" ]; then
            RESULTS["$TEST_KEY"]="ERROR"
            FAILS["$TEST_KEY"]="ssh-or-shell-failure"
        else
            P=$(echo "$RESULT_LINE" | grep -oE 'PASS=[0-9]+' | cut -d= -f2)
            F=$(echo "$RESULT_LINE" | grep -oE 'FAIL=[0-9]+' | cut -d= -f2)
            T=$((P + F))
            RESULTS["$TEST_KEY"]="$P/$T"
            if [ "$F" = "0" ]; then
                PASSED_ROUNDS=$((PASSED_ROUNDS + 1))
            else
                FAILS["$TEST_KEY"]=$(echo "$OUT" | grep -E "^FAILED_CMDS_LINE:" | sed 's/^FAILED_CMDS_LINE: //')
            fi
        fi
    done
done

echo ""
echo "============================================================"
echo "COMPATIBILITY MATRIX (Client x Eureka Server)"
echo "============================================================"
printf "%-20s" "Client"
for entry in "${SERVERS[@]}"; do
    printf " | %-12s" "${entry%%|*}"
done
echo ""
echo "--------------------------------------------------------------"
for client in $CLIENTS; do
    printf "%-20s" "$client"
    for entry in "${SERVERS[@]}"; do
        SLABEL="${entry%%|*}"
        result="${RESULTS["$client -> $SLABEL"]:-?}"
        printf " | %-12s" "$result"
    done
    echo ""
done

echo ""
echo "============================================================"
echo "ROUNDS: $PASSED_ROUNDS/$TOTAL_ROUNDS fully passed"
echo "============================================================"

[ $PASSED_ROUNDS -eq $TOTAL_ROUNDS ] && exit 0 || exit 1
