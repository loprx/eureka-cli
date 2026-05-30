#!/bin/bash
set -e

echo "Building eureka-cli for all platforms..."

TARGETS=(
    "x86_64-unknown-linux-gnu"
    "aarch64-unknown-linux-gnu"
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
    "x86_64-pc-windows-msvc"
)

for target in "${TARGETS[@]}"; do
    echo "Building for $target..."
    cross build --release --target "$target"
    echo "✓ Built $target"
done

echo ""
echo "All builds completed successfully!"
echo "Binaries are in target/<target>/release/"
