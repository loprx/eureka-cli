#!/usr/bin/env bash
# Build Eureka Server test JARs (requires Maven + JDK 17)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "=== Building Eureka Server JARs ==="

build_jar() {
    local dir=$1
    local name=$2
    local jar_path="$dir/target/${name}.jar"

    if [ -f "$jar_path" ]; then
        echo "  [skip] $name already built"
        return 0
    fi

    echo "  [build] $name ..."
    (cd "$dir" && mvn package -DskipTests -q)
    echo "  [done] $name"
}

build_jar "$SCRIPT_DIR/eureka-1.10" "eureka-server-1.10"
build_jar "$SCRIPT_DIR/eureka-2.0" "eureka-server-2.0"

echo ""
echo "=== JARs ready ==="
ls -lh "$SCRIPT_DIR/eureka-1.10/target/eureka-server-1.10.jar" 2>/dev/null || echo "  WARN: eureka-1.10 jar missing"
ls -lh "$SCRIPT_DIR/eureka-2.0/target/eureka-server-2.0.jar" 2>/dev/null || echo "  WARN: eureka-2.0 jar missing"

echo ""
echo "Run with:"
echo "  java -jar eureka-server/eureka-1.10/target/eureka-server-1.10.jar  # port 8761"
echo "  java -jar eureka-server/eureka-2.0/target/eureka-server-2.0.jar    # port 8762 (change in application.yml)"
