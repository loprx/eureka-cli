#!/usr/bin/env bash
# Compatibility test: run the same lifecycle against multiple Eureka versions
set -u

CLI="${CLI:-./target/release/eureka-cli}"

SERVERS=(
    "eureka-1.10|http://localhost:8761/eureka|Spring Cloud 2021 / Boot 2.7"
    "eureka-2.0|http://localhost:8762/eureka|Spring Cloud 2023 / Boot 3.3"
)

PASS=0
FAIL=0
FAILED_TESTS=()

expect_ok() {
    local label="$1"; shift
    local cmd="$1"; shift
    local out rc
    out=$($cmd 2>&1)
    rc=$?
    local ok=1
    if [[ $rc -ne 0 ]]; then ok=0; fi
    for pat in "$@"; do
        if ! grep -qF -- "$pat" <<<"$out"; then ok=0; fi
    done
    if [[ $ok -eq 1 ]]; then
        echo "  ✓ $label"
        PASS=$((PASS+1))
    else
        echo "  ✗ $label (rc=$rc)"
        FAIL=$((FAIL+1))
        FAILED_TESTS+=("[$NAME] $label")
    fi
}

expect_404() {
    local label="$1"; shift
    local cmd="$1"
    local out rc
    out=$($cmd 2>&1)
    rc=$?
    if [[ $rc -ne 0 ]] && grep -qE '(404|Not Found|not found)' <<<"$out"; then
        echo "  ✓ $label"
        PASS=$((PASS+1))
    else
        echo "  ✗ $label (rc=$rc, expected 404)"
        FAIL=$((FAIL+1))
        FAILED_TESTS+=("[$NAME] $label")
    fi
}

for entry in "${SERVERS[@]}"; do
    NAME="${entry%%|*}"
    rest="${entry#*|}"
    URL="${rest%%|*}"
    DESC="${rest##*|}"
    APP="COMPAT$(echo "$NAME" | tr -d '.-' | tr '[:lower:]' '[:upper:]')"
    INSTANCE_ID="compat-${NAME//./-}-$$"

    echo ""
    echo "============================================================"
    echo "Testing: $NAME ($DESC)"
    echo "         $URL"
    echo "============================================================"

    expect_ok "apps list" \
        "$CLI --server $URL apps list" "Application"

    expect_ok "register" \
        "$CLI --server $URL register --app $APP --instance-id $INSTANCE_ID --hostname compat-host --ip 127.0.0.1 --port 9999 --vip-address compat" \
        "registered successfully"

    sleep 3

    expect_ok "apps get" \
        "$CLI --server $URL apps get $APP" "$INSTANCE_ID"

    expect_ok "heartbeat" \
        "$CLI --server $URL heartbeat $APP $INSTANCE_ID" "Heartbeat sent"

    expect_ok "status set" \
        "$CLI --server $URL status set $APP $INSTANCE_ID OUT_OF_SERVICE" "updated"

    sleep 2

    expect_ok "metadata set" \
        "$CLI --server $URL metadata set $APP $INSTANCE_ID deployed-from compat-test" "updated"

    expect_ok "deregister" \
        "$CLI --server $URL deregister $APP $INSTANCE_ID" "deregistered"

    sleep 3

    expect_404 "instance gone after deregister" \
        "$CLI --server $URL instances get -a $APP $INSTANCE_ID"
done

echo ""
echo "============================================================"
echo "RESULTS: $PASS passed, $FAIL failed"
echo "============================================================"
if [[ $FAIL -gt 0 ]]; then
    echo "Failed tests:"
    for t in "${FAILED_TESTS[@]}"; do
        echo "  - $t"
    done
    exit 1
fi
exit 0
