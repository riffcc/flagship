#!/bin/bash
# Build lens-node Docker image
# Resolves symlinks to palace crates

set -e

echo "Preparing build context with resolved symlinks..."

# Create temp directory
BUILD_DIR=$(mktemp -d)
trap "rm -rf $BUILD_DIR" EXIT

# Copy flagship
cp -rL . "$BUILD_DIR/"

# Resolve palace symlinks
for link in crates/consensus-*; do
    if [ -L "$link" ]; then
        target=$(readlink -f "$link")
        rm "$BUILD_DIR/$link"
        cp -r "$target" "$BUILD_DIR/$link"
        echo "  Resolved: $link -> $target"
    fi
done

# Build from temp dir
echo "Building Docker image..."
docker build -f Dockerfile.lens-node -t lens-node:latest "$BUILD_DIR"

echo "✓ Docker image built: lens-node:latest"
