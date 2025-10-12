#!/bin/bash
# Start the full lens-v2 + flagship cluster with HAProxy

set -e

cd "$(dirname "$0")"

echo "🏗️  Building and starting cluster..."
echo ""
echo "Architecture:"
echo "  • HAProxy Load Balancer (port 8080)"
echo "  • 3x Lens v2 Nodes (Rust backend)"
echo "  • 3x Flagship Nodes (Node.js frontend)"
echo ""

# Build and start all services
docker compose -f docker-compose-cluster.yml up --build -d

echo ""
echo "⏳ Waiting for services to become healthy..."
sleep 5

# Check service status
echo ""
echo "📊 Service Status:"
docker compose -f docker-compose-cluster.yml ps

echo ""
echo "✅ Cluster started!"
echo ""
echo "🌐 Access Points:"
echo "  Frontend:    http://localhost:8080"
echo "  API:         http://localhost:8080/api/v1"
echo "  HAProxy Stats: http://localhost:8404/stats"
echo ""
echo "🔍 Check sync status:"
echo "  curl http://localhost:8080/api/v1/ready | jq"
echo ""
echo "📝 View logs:"
echo "  docker compose -f docker-compose-cluster.yml logs -f lens-v2-node-0"
echo "  docker compose -f docker-compose-cluster.yml logs -f flagship-node-0"
echo "  docker compose -f docker-compose-cluster.yml logs -f haproxy"
echo ""
echo "🛑 Stop cluster:"
echo "  docker compose -f docker-compose-cluster.yml down"
echo ""
