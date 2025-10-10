#!/bin/bash
# Test P2P sync with actual data transfer

set -e

echo "🔥 COMPREHENSIVE P2P SYNC TEST"
echo "================================"
echo ""

# Check if nodes are running
NODE_COUNT=$(ps aux | grep lens-node | grep -v grep | wc -l)
if [ "$NODE_COUNT" -lt 3 ]; then
    echo "❌ Not enough nodes running. Run ./test-3-nodes.sh first!"
    exit 1
fi

echo "✅ Found $NODE_COUNT running nodes"
echo ""

# Authorize admin on Node 0
echo "📝 Authorizing admin on Node 0..."
curl -s -X POST http://localhost:5002/api/v1/admin/authorize \
    -H 'Content-Type: application/json' \
    -d '{"publicKey": "sync_test_admin"}' | jq -r '.message'

sleep 1

# Create a release on Node 0
echo ""
echo "📦 Creating release on Node 0..."
RELEASE_RESPONSE=$(curl -s -X POST http://localhost:5002/api/v1/releases \
    -H 'Content-Type: application/json' \
    -H 'X-Public-Key: sync_test_admin' \
    -d '{
        "name": "P2P Sync Test Release",
        "categoryId": "cat-test",
        "categorySlug": "testing",
        "contentCID": "QmTestSync12345"
    }')

echo "$RELEASE_RESPONSE" | jq -r '.id // .error'
RELEASE_ID=$(echo "$RELEASE_RESPONSE" | jq -r '.id // empty')

if [ -z "$RELEASE_ID" ]; then
    echo "❌ Failed to create release"
    exit 1
fi

echo "✅ Created release: $RELEASE_ID"
echo ""

# Check Node 0 has the release
echo "🔍 Checking Node 0..."
NODE0_RELEASES=$(curl -s http://localhost:5002/api/v1/releases | jq '. | length')
echo "  Node 0 has $NODE0_RELEASES release(s)"

# Wait for sync (2 cycles = 60 seconds)
echo ""
echo "⏳ Waiting 65 seconds for P2P sync (2 full cycles)..."
for i in {65..1}; do
    echo -ne "\r  $i seconds remaining...   "
    sleep 1
done
echo ""
echo ""

# Check if synced to Node 1
echo "🔍 Checking Node 1..."
NODE1_RELEASES=$(curl -s http://localhost:5003/api/v1/releases | jq '. | length')
echo "  Node 1 has $NODE1_RELEASES release(s)"

if [ "$NODE1_RELEASES" -gt 0 ]; then
    echo "  ✅ SYNCED! Node 1 has the release!"
    curl -s http://localhost:5003/api/v1/releases | jq -r '.[0] | "  📦 \(.name) (CID: \(.contentCID))"'
else
    echo "  ⚠️  Node 1 does not have the release yet"
fi

# Check if synced to Node 2
echo ""
echo "🔍 Checking Node 2..."
NODE2_RELEASES=$(curl -s http://localhost:5004/api/v1/releases | jq '. | length')
echo "  Node 2 has $NODE2_RELEASES release(s)"

if [ "$NODE2_RELEASES" -gt 0 ]; then
    echo "  ✅ SYNCED! Node 2 has the release!"
    curl -s http://localhost:5004/api/v1/releases | jq -r '.[0] | "  📦 \(.name) (CID: \(.contentCID))"'
else
    echo "  ⚠️  Node 2 does not have the release yet"
fi

# Show sync status
echo ""
echo "📊 Sync Status:"
echo "  Node 0: $(curl -s http://localhost:5002/api/v1/ready | jq -r '"peer_count: \(.peer_count), synced: \(.is_synced)"')"
echo "  Node 1: $(curl -s http://localhost:5003/api/v1/ready | jq -r '"peer_count: \(.peer_count), synced: \(.is_synced)"')"
echo "  Node 2: $(curl -s http://localhost:5004/api/v1/ready | jq -r '"peer_count: \(.peer_count), synced: \(.is_synced)"')"

echo ""
if [ "$NODE1_RELEASES" -gt 0 ] && [ "$NODE2_RELEASES" -gt 0 ]; then
    echo "🎉🎉🎉 SUCCESS! ALL NODES SYNCED! 🎉🎉🎉"
    echo ""
    echo "P2P synchronization is working perfectly!"
else
    echo "⚠️  Sync still in progress or needs debugging"
    echo ""
    echo "Check logs for details:"
    echo "  tail -f /tmp/lens-test/node0/log.txt | grep -E '(WantList|referral|Received|Block)'"
fi

