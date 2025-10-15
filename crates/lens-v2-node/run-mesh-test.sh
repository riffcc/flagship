#!/usr/bin/env bash
set -euo pipefail

# Configurable Hexagonal Toroidal Mesh Test Runner
# Usage:
#   ./run-mesh-test.sh                    # Default: 32×32×8
#   ./run-mesh-test.sh 16 16 4            # Custom: 16×16×4
#   ./run-mesh-test.sh 64 64 8            # Large: 64×64×8

# Get mesh dimensions from arguments or use defaults
MESH_WIDTH=${1:-32}
MESH_HEIGHT=${2:-32}
MESH_DEPTH=${3:-8}

TOTAL_NODES=$((MESH_WIDTH * MESH_HEIGHT * MESH_DEPTH))

echo "🚀 Running Hexagonal Toroidal Mesh Test"
echo "   Dimensions: ${MESH_WIDTH}×${MESH_HEIGHT}×${MESH_DEPTH}"
echo "   Total Nodes: ${TOTAL_NODES}"
echo ""
echo "💡 Visualization will be available at:"
echo "   http://localhost:8080"
echo ""
echo "⏳ Starting test (this may take several minutes)..."
echo ""

# Run the test with environment variables
MESH_WIDTH=$MESH_WIDTH \
MESH_HEIGHT=$MESH_HEIGHT \
MESH_DEPTH=$MESH_DEPTH \
RUST_LOG=info \
cargo test --test configurable_mesh_test -- --nocapture

echo ""
echo "✅ Test complete!"
