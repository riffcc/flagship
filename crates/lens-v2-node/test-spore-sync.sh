#!/bin/bash
# Test SPORE-driven block synchronization
set -e

echo "========================================="
echo "SPORE Block Sync Test"
echo "========================================="
echo ""

# Build
echo "Building lens-node..."
cargo build --release -p lens-v2-node

# Clean up old test data
rm -rf /tmp/lens-test/*
mkdir -p /tmp/lens-test/{node0,node1,node2}

# Start nodes
echo ""
echo "Starting 3 nodes..."

# Node 0 - acts as relay
RUST_LOG=info \
PORT=6002 \
DB_PATH=/tmp/lens-test/node0/rocksdb \
LENS_NODE_ID=0 \
LENS_CLUSTER_ID=test-cluster \
../../target/release/lens-node > /tmp/lens-test/node0/log.txt 2>&1 &
NODE0_PID=$!
echo "Node 0 PID: $NODE0_PID"
sleep 3

# Node 1 - connects to node 0
RUST_LOG=info \
PORT=6003 \
DB_PATH=/tmp/lens-test/node1/rocksdb \
LENS_NODE_ID=1 \
LENS_CLUSTER_ID=test-cluster \
LENS_RELAY_URL=ws://localhost:6002/api/v1/relay/ws \
../../target/release/lens-node > /tmp/lens-test/node1/log.txt 2>&1 &
NODE1_PID=$!
echo "Node 1 PID: $NODE1_PID"
sleep 3

# Node 2 - connects to node 0
RUST_LOG=info \
PORT=6004 \
DB_PATH=/tmp/lens-test/node2/rocksdb \
LENS_NODE_ID=2 \
LENS_CLUSTER_ID=test-cluster \
LENS_RELAY_URL=ws://localhost:6002/api/v1/relay/ws \
../../target/release/lens-node > /tmp/lens-test/node2/log.txt 2>&1 &
NODE2_PID=$!
echo "Node 2 PID: $NODE2_PID"

# Wait for nodes to fully connect
echo ""
echo "Waiting for nodes to connect..."
sleep 5

# Check initial sync status
echo ""
echo "========================================="
echo "Initial Sync Status"
echo "========================================="
for port in 6002 6003 6004; do
    echo ""
    echo "Node at port $port:"
    curl -s http://localhost:$port/api/v1/ready | jq
done

# Authorize admin on node 0
echo ""
echo "========================================="
echo "Authorizing admin on Node 0"
echo "========================================="
curl -X POST http://localhost:6002/api/v1/admin/authorize \
  -H 'Content-Type: application/json' \
  -H 'X-Public-Key: ed25119p/48853522c1cabcae3f588e4e42cbe5b7fcbf8497390913ef9c30c4b6d033a03b' \
  -d '{"publicKey": "ed25119p/48853522c1cabcae3f588e4e42cbe5b7fcbf8497390913ef9c30c4b6d033a03b"}' | jq

sleep 2

# Create a release on node 0
echo ""
echo "========================================="
echo "Creating Release on Node 0"
echo "========================================="
RELEASE_RESPONSE=$(curl -X POST http://localhost:6002/api/v1/releases \
  -H 'Content-Type: application/json' \
  -H 'X-Public-Key: ed25119p/48853522c1cabcae3f588e4e42cbe5b7fcbf8497390913ef9c30c4b6d033a03b' \
  -d '{
    "name": "SPORE Test Release",
    "categoryId": "cat-test",
    "categorySlug": "test",
    "contentCID": "QmTestSPORE123456",
    "version": "1.0.0"
  }')

echo "$RELEASE_RESPONSE" | jq

RELEASE_ID=$(echo "$RELEASE_RESPONSE" | jq -r '.id')
echo ""
echo "Created release with ID: $RELEASE_ID"

# Wait for SPORE propagation
echo ""
echo "Waiting for SPORE propagation..."
sleep 5

# Check SPORE logs
echo ""
echo "========================================="
echo "SPORE Activity Logs"
echo "========================================="
echo ""
echo "Node 0 SPORE logs:"
grep -i "spore\|missing" /tmp/lens-test/node0/log.txt | tail -20 || echo "No SPORE logs found"

echo ""
echo "Node 1 SPORE logs:"
grep -i "spore\|missing" /tmp/lens-test/node1/log.txt | tail -20 || echo "No SPORE logs found"

echo ""
echo "Node 2 SPORE logs:"
grep -i "spore\|missing" /tmp/lens-test/node2/log.txt | tail -20 || echo "No SPORE logs found"

# Check if release synced to other nodes
echo ""
echo "========================================="
echo "Release Synchronization Check"
echo "========================================="

echo ""
echo "Node 1 releases:"
NODE1_RELEASES=$(curl -s http://localhost:6003/api/v1/releases | jq)
echo "$NODE1_RELEASES"

echo ""
echo "Node 2 releases:"
NODE2_RELEASES=$(curl -s http://localhost:6004/api/v1/releases | jq)
echo "$NODE2_RELEASES"

# Final sync status
echo ""
echo "========================================="
echo "Final Sync Status"
echo "========================================="
for port in 6002 6003 6004; do
    echo ""
    echo "Node at port $port:"
    curl -s http://localhost:$port/api/v1/ready | jq
done

# Verify sync
echo ""
echo "========================================="
echo "Test Results"
echo "========================================="

# Check if release exists on all nodes
NODE1_HAS_RELEASE=$(echo "$NODE1_RELEASES" | jq -r ".[] | select(.id == \"$RELEASE_ID\") | .id")
NODE2_HAS_RELEASE=$(echo "$NODE2_RELEASES" | jq -r ".[] | select(.id == \"$RELEASE_ID\") | .id")

if [ "$NODE1_HAS_RELEASE" == "$RELEASE_ID" ]; then
    echo "✅ Release synced to Node 1"
else
    echo "❌ Release NOT synced to Node 1"
fi

if [ "$NODE2_HAS_RELEASE" == "$RELEASE_ID" ]; then
    echo "✅ Release synced to Node 2"
else
    echo "❌ Release NOT synced to Node 2"
fi

echo ""
echo "Test complete!"
echo ""
echo "Clean up with: pkill -f lens-node"
echo ""
