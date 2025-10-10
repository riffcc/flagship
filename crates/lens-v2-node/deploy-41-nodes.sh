#!/bin/bash
# Deploy 41 lens-v2-nodes in a cluster
# All nodes sync via Node 0's relay

set -e

# Configuration
CLUSTER_ID="${LENS_CLUSTER_ID:-lens-cluster-production}"
CLUSTER_SIZE=41
BASE_HTTP_PORT=5002
BASE_DATA_DIR="/opt/lens-cluster"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}===================================${NC}"
echo -e "${BLUE}  Deploying 41-Node Lens Cluster  ${NC}"
echo -e "${BLUE}===================================${NC}"
echo ""
echo -e "${GREEN}Cluster ID:${NC} $CLUSTER_ID"
echo -e "${GREEN}Nodes:${NC} $CLUSTER_SIZE"
echo -e "${GREEN}Relay:${NC} Node 0 (http://localhost:$BASE_HTTP_PORT)"
echo ""

# Create base data directory
mkdir -p "$BASE_DATA_DIR"

# Build lens-node binary
echo -e "${YELLOW}Building lens-node binary...${NC}"
cargo build --release -p lens-v2-node
echo -e "${GREEN}✓ Build complete${NC}"
echo ""

# Function to start a node
start_node() {
    local node_id=$1
    local http_port=$((BASE_HTTP_PORT + node_id))
    local data_dir="$BASE_DATA_DIR/node-$node_id"

    # Create data directory
    mkdir -p "$data_dir"

    # Set environment variables
    export LENS_CLUSTER_ID="$CLUSTER_ID"
    export LENS_NODE_ID="$node_id"
    export LENS_CLUSTER_SIZE="$CLUSTER_SIZE"
    export PORT="$http_port"
    export DB_PATH="$data_dir/rocksdb"

    # Node 0 uses its own relay, others connect to Node 0
    if [ "$node_id" -eq 0 ]; then
        export LENS_RELAY_URL="ws://localhost:$BASE_HTTP_PORT/api/v1/relay/ws"
    else
        export LENS_RELAY_URL="ws://localhost:$BASE_HTTP_PORT/api/v1/relay/ws"
    fi

    # Start node in background
    echo -e "${BLUE}Starting Node $node_id on port $http_port...${NC}"
    nohup ../../target/release/lens-node \
        > "$data_dir/node.log" 2>&1 &

    echo $! > "$data_dir/node.pid"
    echo -e "${GREEN}✓ Node $node_id started (PID: $(cat $data_dir/node.pid))${NC}"
}

# Start all nodes
echo -e "${YELLOW}Starting nodes...${NC}"
for i in $(seq 0 $((CLUSTER_SIZE - 1))); do
    start_node $i
    # Small delay to avoid port conflicts
    sleep 0.5
done

echo ""
echo -e "${GREEN}===================================${NC}"
echo -e "${GREEN}  All 41 nodes started!           ${NC}"
echo -e "${GREEN}===================================${NC}"
echo ""
echo -e "${BLUE}Node URLs:${NC}"
echo -e "  Node 0 (Relay): http://localhost:$BASE_HTTP_PORT"
echo -e "  Node 1:         http://localhost:$((BASE_HTTP_PORT + 1))"
echo -e "  Node 40:        http://localhost:$((BASE_HTTP_PORT + 40))"
echo ""
echo -e "${BLUE}Sync Status:${NC}"
echo -e "  curl http://localhost:$BASE_HTTP_PORT/api/v1/ready | jq"
echo ""
echo -e "${BLUE}Stop all nodes:${NC}"
echo -e "  ./stop-41-nodes.sh"
echo ""
echo -e "${BLUE}View logs:${NC}"
echo -e "  tail -f $BASE_DATA_DIR/node-0/node.log"
echo ""
