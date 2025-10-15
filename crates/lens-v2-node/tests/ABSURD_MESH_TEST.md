# ABSURD Mesh Test - 64×64×16 Hexagonal Toroidal Grid

## Overview

This test spawns **65,536 real lens-v2-node instances** in a 64×64×16 hexagonal toroidal mesh and establishes WebRTC connections between all neighbors.

**This is ABSURD. That's the point.**

## Specifications

- **Mesh Dimensions**: 64 wide × 64 deep × 16 layers = 65,536 total slots
- **Topology**: 2.5D hexagonal toroidal mesh
- **Neighbors per node**: 8 (6 hexagonal + 2 vertical with toroidal wraparound)
- **Total WebRTC connections**: ~262,144 (each node connects to 8 neighbors, bidirectional)
- **Protocol**: Two Generals Protocol (TGP) over WebRTC DataChannels
- **Port range**: 10,000 - 75,535
- **Visualization**: http://localhost:8080

## System Requirements

- **RAM**: ~32 GB minimum
- **VRAM**: 96 GB for browser visualization (use Chrome with hardware acceleration!)
- **CPU**: Multi-core (the more the better)
- **Ports**: 10,000-75,535 must be available
- **Time**: 30-60 minutes to establish all connections

## Running the Test

```bash
cd /opt/castle/workspace/flagship/crates/lens-v2-node
cargo test --test absurd_mesh_test -- --nocapture
```

## What Happens

1. **HTTP Server Startup** (port 8080)
   - Serves interactive 3D force-directed visualization
   - WebSocket endpoint for real-time updates
   - DHT write/read API endpoints

2. **Node Spawning** (~60-90 seconds)
   - Spawns nodes in batches of 100 (656 batches total)
   - 0.1ms delay between nodes in each batch
   - 100ms delay between batches (system breathing room)
   - Progress printed after each batch completes

3. **WebRTC Connection Establishment** (~30-60 minutes)
   - Establishes ~262,144 connections in parallel
   - 50ms delay between starting each connection
   - Progress printed every 1,000 connections
   - 30 second timeout per connection

4. **Gossip Propagation** (~5 seconds)
   - All nodes announce slot ownership
   - Gossip propagates through entire mesh

5. **Visualization Running** (10 minutes)
   - All nodes remain active for testing
   - Press Ctrl+C to stop early

## Visualization

Open http://localhost:8080 in Chrome (hardware acceleration recommended!)

The visualization shows:
- ✨ Interactive 3D force-directed graph
- 🔷 65,536 nodes positioned according to hexagonal toroidal topology
- 🔗 ~262,144 edges showing WebRTC connections
- 📊 Real-time stats and controls
- 🎯 Click nodes to see details

**WARNING**: Rendering 65K nodes may be slow even with hardware acceleration!

## API Endpoints

### WebSocket Real-time Updates
```
ws://localhost:8080/ws
```

### Mesh Topology Map
```
http://localhost:8080/mesh-test-map
```

Returns JSON with:
- Full mesh configuration (64×64×16)
- All 65,536 nodes with slot coordinates
- All ~262,144 edges
- Statistics (peer count, occupancy, etc.)

### Individual Node APIs
```
http://localhost:10000/api/v1/map  # node 0
http://localhost:10001/api/v1/map  # node 1
...
http://localhost:75535/api/v1/map  # node 65535
```

### DHT Operations (via visualization server)
```
POST http://localhost:8080/dht/write
{
  "node_id": "node-0",
  "key": "test-key",
  "value": "test-value"
}

POST http://localhost:8080/dht/read
{
  "node_id": "node-0",
  "key": "test-key"
}
```

## Expected Output

```
🚀🚀🚀 Starting ABSURD 64×64×16 hexagonal toroidal mesh test
   Mesh dimensions: 64 wide × 64 deep × 16 layers = 65,536 total slots
   Each node connects to its 8 neighbors in the 2.5D toroid
   Packets route via Two Generals Protocol (TGP) over WebRTC
   ⚠️  WARNING: This will use ~32GB RAM and massive CPU resources!

🌐 Starting visualization server on http://0.0.0.0:8080
   ✅ Visualization server ready at http://0.0.0.0:8080
   📊 Map endpoint: http://0.0.0.0:8080/mesh-test-map
   🔌 WebSocket endpoint: ws://0.0.0.0:8080/ws

📐 Planning ABSURD mesh topology...
   Mesh: 64×64×16 = 65536 total slots
   Spawning ALL 65,536 nodes in parallel...
   Expected spawn time: ~60-90 seconds

🔧 Spawning 65536 nodes in batches of 100...
   📦 Batch 1/656: Spawning nodes 0 - 99...
   ✅ Batch 1 complete (100 / 65536 nodes spawned)
   📦 Batch 2/656: Spawning nodes 100 - 199...
   ✅ Batch 2 complete (200 / 65536 nodes spawned)
   ...
   📦 Batch 656/656: Spawning nodes 65500 - 65535...
   ✅ Batch 656 complete (65536 / 65536 nodes spawned)

✅ All 65536 nodes spawned in 656 batches

🔗 Establishing WebRTC connections (watch the mesh form in real-time!)...
   🔗 Connection 0 / ~262144: (0,0,0) ↔ (1,0,0) via PlusA
   ...
   ✅ 1000 / 262144 connections established (0.4%)
   ...
✅ Connection establishment complete:
   ✓ 262144 connections succeeded

📢 Announcing slot ownership for all 65536 nodes...
✅ Gossip propagation complete

✅🎉 ABSURD 64×64×16 hexagonal toroidal mesh test COMPLETE!
   ✓ 65536 nodes spawned across entire mesh
   ✓ 262144 / 262144 WebRTC connections established (TGP over WebRTC)
   ✓ Each node connected to its 8 neighbors in the toroid
   ✓ Gossip propagated through entire mesh

🎨 3D MESH VISUALIZATION:
   🌐 Open in browser: http://localhost:8080
   ✨ Interactive 3D force-directed graph
   ⚠️  WARNING: Visualization may be slow with 65K nodes - use Chrome with hardware acceleration!

⏸️  Nodes will remain running for 600 seconds (10 minutes) for visualization...
   Press Ctrl+C to stop early
```

## Topology Details

### Hexagonal 2.5D Toroidal Mesh

The mesh uses a hexagonal grid with toroidal wraparound:

- **Hexagonal Plane (X, Y)**: Each node has 6 neighbors in the hex grid
- **Vertical Layers (Z)**: Each node has 2 neighbors above/below (wraparound)
- **Toroidal Wraparound**: Edges wrap around to create a seamless donut topology

### Neighbor Directions

Each node connects to 8 neighbors:
- `PlusA`, `MinusA` - Hexagonal neighbors (axis A)
- `PlusB`, `MinusB` - Hexagonal neighbors (axis B)
- `PlusC`, `MinusC` - Hexagonal neighbors (axis C)
- `Up`, `Down` - Vertical neighbors (with toroidal wraparound)

### Slot Coordinates

Every slot is identified by `(x, y, z)` coordinates:
- `x ∈ [0, 63]` - Width coordinate
- `y ∈ [0, 63]` - Depth coordinate
- `z ∈ [0, 15]` - Layer coordinate

## Use Cases

This test validates:
- ✅ Massive P2P network scalability (65K+ peers)
- ✅ WebRTC connection establishment at scale
- ✅ TGP reliability over WebRTC DataChannels
- ✅ DHT routing in hexagonal toroidal topology
- ✅ Gossip propagation across 65K nodes
- ✅ Visualization performance with large networks
- ✅ System resource management under extreme load

## Why "ABSURD"?

Because spawning 65,536 real nodes with 262,144 real WebRTC connections on a single machine is, well, **absurd**. But it proves the system can handle it!

This is the kind of test you run when someone asks "but can it scale?" and you want to answer with **overwhelming force**.

## Related Tests

- `massive_mesh_test` - 8×8×2 (128 nodes, port 8080)
- `configurable_mesh_test` - Configurable via env vars (default 32×32×8, port 8080)

## Notes

- All tests use port 8080 for visualization
- Only one can run at a time
- Kill with `Ctrl+C` or `pkill -f absurd_mesh_test`
- Check port availability: `lsof -i :8080`

---

**VICTORY AWAITS** 🎉
