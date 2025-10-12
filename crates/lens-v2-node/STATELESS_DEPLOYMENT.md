# Stateless Multi-Node Deployment with MooseFS

## Architecture Overview

The lens-node upload system is designed to be **completely stateless** when deployed with shared storage:

```
┌─────────────────────────────────────────────────────────────────┐
│                         MooseFS Mount                           │
│                    /mnt/mfs/lens-uploads/                       │
│                                                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │  staging/    │  │  approved/   │  │   admin_keys │         │
│  │  ├─ uuid1/   │  │  ├─ uuid3/   │  │   .json      │         │
│  │  ├─ uuid1... │  │  ├─ uuid3... │  │              │         │
│  │  ├─ uuid2/   │  │  ├─ uuid4/   │  │              │         │
│  │  ├─ uuid2... │  │  └─ uuid4... │  │              │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
└─────────────────────────────────────────────────────────────────┘
                             ▲
                             │ All nodes read/write same files
          ┌──────────────────┼──────────────────┐
          │                  │                  │
    ┌─────▼─────┐      ┌─────▼─────┐     ┌─────▼─────┐
    │ lens-node │      │ lens-node │     │ lens-node │
    │   node1   │      │   node2   │     │   node3   │
    │ :5002     │      │ :5002     │     │ :5002     │
    └─────┬─────┘      └─────┬─────┘     └─────┬─────┘
          │                  │                  │
          └──────────────────┼──────────────────┘
                             │
                    ┌────────▼────────┐
                    │  IPFS Cluster   │
                    │  (3 nodes)      │
                    │  Auto-replicates│
                    │  across sites   │
                    └─────────────────┘
```

## Key Features

1. **Upload to any node** - Files written to shared MooseFS
2. **List from any node** - Reads from shared MooseFS
3. **Approve from any node** - Reads from MooseFS, pins to IPFS Cluster
4. **Cross-site replication** - IPFS Cluster handles automatically
5. **No sticky sessions required** - Load balancer can round-robin freely

## Configuration

### 1. MooseFS Setup

Mount MooseFS on all lens-node servers:

```bash
# On each lens-node server
mkdir -p /mnt/mfs/lens-uploads
mfsmount /mnt/mfs/lens-uploads -H mfsmaster.palace.riff.cc
```

Create directory structure:

```bash
mkdir -p /mnt/mfs/lens-uploads/staging
mkdir -p /mnt/mfs/lens-uploads/approved
```

### 2. Environment Variables

Configure each lens-node instance to use MooseFS paths:

**Option A: Environment file** (`/etc/lens-node/env`)

```bash
# Upload directories (shared via MooseFS)
UPLOAD_STAGING_DIR=/mnt/mfs/lens-uploads/staging
UPLOAD_APPROVED_DIR=/mnt/mfs/lens-uploads/approved

# Database (local per-node, synced via RocksDB)
DB_PATH=/var/lib/lens-node/rocksdb

# Server config
PORT=5002
LENS_RELAY_URL=ws://relay.example.com/api/v1/relay/ws
```

**Option B: Systemd service** (`/etc/systemd/system/lens-node.service`)

```ini
[Unit]
Description=Lens Node v2 - Stateless Instance
After=network.target mfsmount.service
Requires=mfsmount.service

[Service]
Type=simple
User=lens-node
Group=lens-node
WorkingDirectory=/opt/lens-node
ExecStart=/opt/lens-node/lens-node

# MooseFS shared storage
Environment=UPLOAD_STAGING_DIR=/mnt/mfs/lens-uploads/staging
Environment=UPLOAD_APPROVED_DIR=/mnt/mfs/lens-uploads/approved

# Local RocksDB
Environment=DB_PATH=/var/lib/lens-node/rocksdb

# Server config
Environment=PORT=5002
Environment=RUST_LOG=info

# Restart policy
Restart=on-failure
RestartSec=5s

[Install]
WantedBy=multi-user.target
```

### 3. HAProxy Load Balancer

No sticky sessions required! Round-robin freely:

```haproxy
backend lens_nodes
    balance roundrobin
    option httpchk GET /api/v1/health

    server lens-node1 10.7.1.100:5002 check
    server lens-node2 10.7.1.101:5002 check
    server lens-node3 10.7.1.102:5002 check
```

## Workflow Examples

### Upload Flow (Stateless)

```bash
# User uploads to node1
curl -X POST http://lens.example.com/api/v1/upload \
  -H "X-Public-Key: uploader_abc123" \
  -F "file=@photo.jpg"
# → HAProxy routes to node1
# → node1 saves to /mnt/mfs/lens-uploads/staging/uuid/photo.jpg
# → node1 writes /mnt/mfs/lens-uploads/staging/uuid.meta
# → File instantly visible to all nodes

# Moderator checks pending uploads on node2 (different node!)
curl -X GET http://lens.example.com/api/v1/admin/uploads/pending \
  -H "X-Public-Key: moderator_def456"
# → HAProxy routes to node2
# → node2 reads from /mnt/mfs/lens-uploads/staging/*.meta
# → Sees upload from node1 ✓

# Moderator approves from node3 (yet another node!)
curl -X POST http://lens.example.com/api/v1/admin/uploads/UUID/approve \
  -H "X-Public-Key: moderator_def456"
# → HAProxy routes to node3
# → node3 reads /mnt/mfs/lens-uploads/staging/uuid/photo.jpg
# → node3 runs ipfs-cluster-ctl add
# → IPFS Cluster replicates to all cluster nodes
# → node3 updates /mnt/mfs/lens-uploads/staging/uuid.meta
# → All nodes see the update ✓
```

## Multi-Site Replication

IPFS Cluster automatically handles cross-site replication:

```
Site A (Palace)                    Site B (Remote)
┌──────────────────┐              ┌──────────────────┐
│ MooseFS          │              │ MooseFS          │
│ /mnt/mfs/...     │              │ /mnt/mfs/...     │
│                  │              │                  │
│ ┌──────────────┐ │              │ ┌──────────────┐ │
│ │ lens-node x3 │ │              │ │ lens-node x3 │ │
│ └──────┬───────┘ │              │ └──────┬───────┘ │
│        │         │              │        │         │
│ ┌──────▼───────┐ │              │ ┌──────▼───────┐ │
│ │IPFS Cluster  │ │◄────────────►│ │IPFS Cluster  │ │
│ │cluster01-03  │ │   Replicate  │ │cluster04-06  │ │
│ └──────────────┘ │              │ └──────────────┘ │
└──────────────────┘              └──────────────────┘

When node approves upload:
1. Pins to local IPFS Cluster
2. IPFS Cluster automatically replicates to Site B
3. Content available at both sites
4. Each site has its own staging area (local to site)
```

## Deployment Steps

### Single Site (3 nodes)

```bash
# 1. Deploy MooseFS
# (Already done at Palace)

# 2. Create mount points on all lens-node servers
for node in lens-node{1..3}.palace.riff.cc; do
  ssh root@$node "mkdir -p /mnt/mfs/lens-uploads"
  ssh root@$node "mfsmount /mnt/mfs/lens-uploads -H mfsmaster.palace.riff.cc"
done

# 3. Create shared directories
ssh root@lens-node1.palace.riff.cc "mkdir -p /mnt/mfs/lens-uploads/{staging,approved}"

# 4. Deploy lens-node binary to all servers
for node in lens-node{1..3}.palace.riff.cc; do
  scp target/release/lens-node root@$node:/opt/lens-node/
done

# 5. Create systemd service on all nodes
for node in lens-node{1..3}.palace.riff.cc; do
  scp deployment/lens-node.service root@$node:/etc/systemd/system/
  ssh root@$node "systemctl daemon-reload"
  ssh root@$node "systemctl enable lens-node"
  ssh root@$node "systemctl start lens-node"
done

# 6. Configure HAProxy
# Add backend configuration (see HAProxy section above)

# 7. Verify stateless operation
curl http://lens.example.com/api/v1/health
# Should hit different nodes on each request
```

## Testing Statelessness

### Test 1: Upload and List from Different Nodes

```bash
# Force upload to specific node
curl -X POST http://10.7.1.100:5002/api/v1/upload \
  -H "X-Public-Key: uploader_abc123" \
  -F "file=@test.jpg"
# Response: {"upload_id": "abc-123-def", ...}

# List from different node
curl -X GET http://10.7.1.101:5002/api/v1/admin/uploads/pending \
  -H "X-Public-Key: moderator_xyz"
# Should see "abc-123-def" in results ✓
```

### Test 2: Approve from Third Node

```bash
# Approve from yet another node
curl -X POST http://10.7.1.102:5002/api/v1/admin/uploads/abc-123-def/approve \
  -H "X-Public-Key: moderator_xyz"
# Should successfully pin to IPFS ✓

# Verify from first node
curl -X GET http://10.7.1.100:5002/api/v1/admin/uploads/abc-123-def \
  -H "X-Public-Key: moderator_xyz"
# Should show status: approved, ipfs_cid: QmXXX ✓
```

### Test 3: Node Failure Handling

```bash
# Stop one node
ssh root@lens-node2.palace.riff.cc "systemctl stop lens-node"

# Upload still works (HAProxy routes around it)
curl -X POST http://lens.example.com/api/v1/upload \
  -H "X-Public-Key: uploader_abc123" \
  -F "file=@test2.jpg"
# Success ✓

# Bring node back up
ssh root@lens-node2.palace.riff.cc "systemctl start lens-node"

# Node2 immediately sees all uploads (reads from MooseFS)
curl -X GET http://10.7.1.101:5002/api/v1/admin/uploads/pending \
  -H "X-Public-Key: moderator_xyz"
# Shows all uploads including ones made while it was down ✓
```

## Monitoring

### Health Checks

```bash
# Check all nodes
for ip in 10.7.1.{100..102}; do
  echo "=== Node $ip ==="
  curl -s http://$ip:5002/api/v1/health | jq
done
```

### MooseFS Status

```bash
# Check mount on all nodes
for node in lens-node{1..3}.palace.riff.cc; do
  echo "=== $node ==="
  ssh root@$node "df -h | grep mfs"
  ssh root@$node "ls -la /mnt/mfs/lens-uploads/"
done
```

### Upload Queue Depth

```bash
# Count pending uploads (any node)
curl -s http://lens.example.com/api/v1/admin/uploads/pending \
  -H "X-Public-Key: moderator_xyz" | jq '.total'
```

## Disaster Recovery

### Backup Strategy

Since all state is in MooseFS:

```bash
# Backup MooseFS directory
mfstools snapshot /mnt/mfs/lens-uploads /mnt/mfs/backups/lens-uploads-$(date +%Y%m%d)
```

### Recovery

```bash
# Restore from backup
cp -r /mnt/mfs/backups/lens-uploads-20251010/* /mnt/mfs/lens-uploads/

# All nodes immediately see restored data
# No per-node recovery needed ✓
```

## Advantages

1. **True Horizontal Scaling** - Add nodes without configuration changes
2. **Zero Downtime Deployments** - Rolling restart without data loss
3. **No Session Affinity** - Simpler load balancing
4. **Instant Failover** - Any node can handle any request
5. **Simplified Backup** - One filesystem, not N node states
6. **Easy Monitoring** - Same metrics from any node
7. **Cross-Site Ready** - IPFS Cluster handles geographic replication

## Performance Considerations

### MooseFS Performance

- **Read Performance**: Near-native filesystem speeds
- **Write Performance**: Slightly slower due to network replication
- **Metadata Operations**: Very fast (in-memory cache)

### Optimization Tips

1. **Use SSD for MooseFS metadata servers**
2. **10 Gbit+ network between lens-nodes and MooseFS**
3. **Tune MooseFS chunk size** for upload file sizes
4. **Enable MooseFS read cache** on lens-node servers

### Expected Throughput

With 10 Gbit network and 3 lens-node instances:

- **Uploads**: ~500 MB/s aggregate
- **Approvals**: ~1000/minute (IPFS Cluster bottleneck)
- **Concurrent Users**: Thousands (limited by CPU, not storage)

## Troubleshooting

### Uploads Not Appearing on Other Nodes

```bash
# Check MooseFS mount
mount | grep mfs
# Should show: mfsmount on /mnt/mfs/lens-uploads

# Check file permissions
ls -la /mnt/mfs/lens-uploads/staging/

# Verify all nodes use same path
for node in lens-node{1..3}.palace.riff.cc; do
  ssh root@$node "systemctl show lens-node | grep UPLOAD_STAGING_DIR"
done
```

### IPFS Cluster Pinning Failures

```bash
# Check ipfs-cluster-ctl is available
which ipfs-cluster-ctl

# Test manual pin from each node
for node in lens-node{1..3}.palace.riff.cc; do
  echo "=== $node ==="
  ssh root@$node "ipfs-cluster-ctl add /mnt/mfs/lens-uploads/staging/test.txt"
done
```

### Slow Performance

```bash
# Check MooseFS latency
mfstools info /mnt/mfs/lens-uploads/

# Check network bandwidth
iperf3 -c mfsmaster.palace.riff.cc

# Monitor lens-node resource usage
htop
```

## Summary

**The system is completely stateless because:**

1. ✅ All files stored on shared MooseFS
2. ✅ All metadata stored on shared MooseFS
3. ✅ IPFS Cluster state managed by cluster (not lens-node)
4. ✅ RocksDB database synced via P2P (for releases, not uploads)
5. ✅ No local state required on any lens-node

**Any node can handle:**
- ✅ File uploads
- ✅ Listing pending uploads
- ✅ Approving/rejecting uploads
- ✅ Viewing upload details

**Load balancer can:**
- ✅ Round-robin without sticky sessions
- ✅ Health-check based routing
- ✅ Zero-downtime rolling updates
- ✅ Instant failover

This architecture provides **true stateless horizontal scalability** with no single point of failure! 🚀
