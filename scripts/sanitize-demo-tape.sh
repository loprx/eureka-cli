#!/usr/bin/env bash
# Sanitize internal hostnames/IPs in the recorded GIF's source frames.
#
# VHS is deterministic: the same .tape always produces the same GIF for the
# same Eureka data. So this script doesn't run on the GIF — it runs on a
# copy of the .tape *if* you'd rather record against production data and
# then redact the values for publication.
#
# Workflow:
#   1. Record once against your real Eureka:
#      vhs assets/demo.tape
#   2. Decide what literals leak (look at assets/demo.gif)
#   3. Edit this file's REPLACEMENTS list
#   4. Run this script — it produces assets/demo-public.tape
#   5. Re-record:
#      vhs assets/demo-public.tape
#
# This way the public .tape used by README has no internal data.

set -euo pipefail

SRC="${1:-assets/demo.tape}"
DST="${2:-assets/demo-public.tape}"

# label|search|replace — extend as needed
REPLACEMENTS=(
    "lan-ip|10\.1\.72\.[0-9]+|10.0.0.42"
    "lan-port|9999|8761"
    "internal-prefix-1|SVC-|USER-"
    "internal-prefix-2|JOB-|ORDER-"
)

cp "$SRC" "$DST"
for r in "${REPLACEMENTS[@]}"; do
    label="${r%%|*}"
    rest="${r#*|}"
    search="${rest%%|*}"
    replace="${rest##*|}"
    sed -i.bak -E "s|${search}|${replace}|g" "$DST"
done
rm -f "${DST}.bak"

echo "Sanitized .tape written to $DST"
echo "Now run: NO_PROXY=10.0.0.0/8 vhs $DST"
