#!/bin/bash
# Test 3 lens nodes syncing locally

set -e

# Build first
echo "Building lens-node..."
cargo build --release -p lens-v2-node

# Create data dirs
mkdir -p /tmp/lens-test/{node0,node1,node2}

# Kill any existing nodes
pkill -f "lens-node" || true
sleep 1

echo "Starting 3 nodes..."

# Node 0 - acts as relay
RUST_LOG=info \
PORT=5002 \
DB_PATH=/tmp/lens-test/node0/rocksdb \
LENS_NODE_ID=0 \
LENS_CLUSTER_ID=test-cluster \
../../target/release/lens-node > /tmp/lens-test/node0/log.txt 2>&1 &
echo "Node 0 PID: $!"
sleep 2

# Node 1 - connects to node 0
RUST_LOG=info \
PORT=5003 \
DB_PATH=/tmp/lens-test/node1/rocksdb \
LENS_NODE_ID=1 \
LENS_CLUSTER_ID=test-cluster \
LENS_RELAY_URL=ws://localhost:5002/api/v1/relay/ws \
../../target/release/lens-node > /tmp/lens-test/node1/log.txt 2>&1 &
echo "Node 1 PID: $!"
sleep 2

# Node 2 - connects to node 0
RUST_LOG=info \
PORT=5004 \
DB_PATH=/tmp/lens-test/node2/rocksdb \
LENS_NODE_ID=2 \
LENS_CLUSTER_ID=test-cluster \
LENS_RELAY_URL=ws://localhost:5002/api/v1/relay/ws \
../../target/release/lens-node > /tmp/lens-test/node2/log.txt 2>&1 &
echo "Node 2 PID: $!"

sleep 3

echo ""
echo "✓ 3 nodes started!"
echo ""
echo "Node URLs:"
echo "  Node 0: http://localhost:5002"
echo "  Node 1: http://localhost:5003"
echo "  Node 2: http://localhost:5004"
echo ""
echo "Check sync status:"
echo "  curl http://localhost:5002/api/v1/ready | jq"
echo "  curl http://localhost:5003/api/v1/ready | jq"
echo "  curl http://localhost:5004/api/v1/ready | jq"
echo ""
echo "Create a release on Node 0:"
echo "  curl -X POST http://localhost:5002/api/v1/admin/authorize -H 'Content-Type: application/json' -d '{\"publicKey\": \"test_admin\"}'"
echo "  curl -X POST http://localhost:5002/api/v1/releases -H 'Content-Type: application/json' -d '{\"name\": \"Test Release\", \"categoryId\": \"cat-1\", \"categorySlug\": \"test\", \"contentCID\": \"QmTest123\"}'"
echo ""
echo "Check if it synced to Node 1 and Node 2:"
echo "  curl http://localhost:5003/api/v1/releases | jq"
echo "  curl http://localhost:5004/api/v1/releases | jq"
echo ""
echo "View logs:"
echo "  tail -f /tmp/lens-test/node0/log.txt"
echo "  tail -f /tmp/lens-test/node1/log.txt"
echo "  tail -f /tmp/lens-test/node2/log.txt"
echo ""
echo "Stop nodes:"
echo "  pkill -f lens-node"
echo ""
