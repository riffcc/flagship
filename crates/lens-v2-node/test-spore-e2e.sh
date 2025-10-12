#!/bin/bash
# End-to-end SPORE synchronization test with Ed25519 signatures
set -e

cd /opt/castle/workspace/flagship/crates/lens-v2-node

echo "========================================="
echo "SPORE End-to-End Test"
echo "========================================="
echo ""

# Kill existing nodes
echo "Cleaning up..."
pkill -9 -f lens-node || true
sleep 2

# Build
echo "Building..."
cargo build --release -p lens-v2-node
cargo build --release --example test_spore_sync

# Clean data
rm -rf /tmp/lens-test
mkdir -p /tmp/lens-test/{node0,node1,node2}

# Start Node 0 (relay)
echo "Starting Node 0 (relay)..."
RUST_LOG=info \
PORT=6002 \
DB_PATH=/tmp/lens-test/node0/rocksdb \
LENS_NODE_ID=0 \
LENS_CLUSTER_ID=test-cluster \
../../target/release/lens-node > /tmp/lens-test/node0/log.txt 2>&1 &
sleep 3

# Start Node 1
echo "Starting Node 1..."
RUST_LOG=info \
PORT=6003 \
DB_PATH=/tmp/lens-test/node1/rocksdb \
LENS_NODE_ID=1 \
LENS_CLUSTER_ID=test-cluster \
LENS_RELAY_URL=ws://localhost:6002/api/v1/relay/ws \
../../target/release/lens-node > /tmp/lens-test/node1/log.txt 2>&1 &
sleep 3

# Start Node 2
echo "Starting Node 2..."
RUST_LOG=info \
PORT=6004 \
DB_PATH=/tmp/lens-test/node2/rocksdb \
LENS_NODE_ID=2 \
LENS_CLUSTER_ID=test-cluster \
LENS_RELAY_URL=ws://localhost:6002/api/v1/relay/ws \
../../target/release/lens-node > /tmp/lens-test/node2/log.txt 2>&1 &
sleep 3

echo "All nodes started!"
echo ""

# Run test
echo "Running SPORE synchronization test..."
echo ""
../../target/release/examples/test_spore_sync

# Show SPORE logs
echo ""
echo "========================================="
echo "SPORE Activity Logs"
echo "========================================="
echo ""
echo "Node 0:"
grep -i "spore\|missing\|block" /tmp/lens-test/node0/log.txt | tail -30 || echo "No logs"

echo ""
echo "Node 1:"
grep -i "spore\|missing\|block" /tmp/lens-test/node1/log.txt | tail -30 || echo "No logs"

echo ""
echo "Node 2:"
grep -i "spore\|missing\|block" /tmp/lens-test/node2/log.txt | tail -30 || echo "No logs"

echo ""
echo "========================================="
echo "Test Complete!"
echo "========================================="
echo ""
echo "Clean up with: pkill -f lens-node"
echo ""
