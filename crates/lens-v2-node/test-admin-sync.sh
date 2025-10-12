#!/bin/bash
# Test admin authorization sync across 3 nodes
set -e

cd /opt/castle/workspace/flagship/crates/lens-v2-node

echo "========================================="
echo "Admin Authorization Sync Test"
echo "========================================="
echo ""

# Kill existing nodes
echo "Cleaning up..."
pkill -9 -f lens-node || true
sleep 2

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

# Test admin key
ADMIN_KEY="ed25119p/48853522c1cabcae3f588e4e42cbe5b7fcbf8497390913ef9c30c4b6d033a03b"

# Authorize admin on node 0
echo "Step 1: Authorizing admin on Node 0"
echo "  Admin key: $ADMIN_KEY"
curl -X POST http://localhost:6002/api/v1/admin/authorize \
  -H 'Content-Type: application/json' \
  -d "{\"publicKey\": \"$ADMIN_KEY\"}" \
  -s | jq .
echo ""

# Wait for sync
echo "Step 2: Waiting for admin authorization to sync (10 seconds)..."
sleep 10

# Check admin status on all nodes
echo "Step 3: Checking admin status on all nodes"
echo ""

for port in 6002 6003 6004; do
  echo "=== Node $port ==="
  curl -s "http://localhost:$port/api/v1/account/$ADMIN_KEY" | jq .
  echo ""
done

# Check logs for admin sync messages
echo "========================================="
echo "Admin Sync Activity Logs"
echo "========================================="
echo ""

echo "Node 0 (authorizer):"
grep -i "admin\|authorization\|block" /tmp/lens-test/node0/log.txt | tail -20 || echo "No logs"
echo ""

echo "Node 1 (should receive):"
grep -i "admin\|authorization\|block" /tmp/lens-test/node1/log.txt | tail -20 || echo "No logs"
echo ""

echo "Node 2 (should receive):"
grep -i "admin\|authorization\|block" /tmp/lens-test/node2/log.txt | tail -20 || echo "No logs"
echo ""

echo "========================================="
echo "Test Complete!"
echo "========================================="
echo ""
echo "Clean up with: pkill -f lens-node"
echo ""
