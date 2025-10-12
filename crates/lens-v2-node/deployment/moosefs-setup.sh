#!/bin/bash
# MooseFS Setup for Stateless Lens Node Deployment
# Run this on all lens-node servers

set -e

MOOSEFS_MASTER="${MOOSEFS_MASTER:-mfsmaster.palace.riff.cc}"
MOUNT_POINT="${MOUNT_POINT:-/mnt/mfs/lens-uploads}"

echo "=== MooseFS Setup for Lens Node ==="
echo "Master: $MOOSEFS_MASTER"
echo "Mount: $MOUNT_POINT"
echo ""

# 1. Create mount point
echo "[1/5] Creating mount point..."
mkdir -p "$MOUNT_POINT"

# 2. Mount MooseFS
echo "[2/5] Mounting MooseFS..."
if mount | grep -q "$MOUNT_POINT"; then
    echo "Already mounted, skipping..."
else
    mfsmount "$MOUNT_POINT" -H "$MOOSEFS_MASTER"
    echo "✓ Mounted"
fi

# 3. Create directory structure (only needs to run once, but safe to re-run)
echo "[3/5] Creating directory structure..."
mkdir -p "$MOUNT_POINT/staging"
mkdir -p "$MOUNT_POINT/approved"
echo "✓ Directories created"

# 4. Set permissions
echo "[4/5] Setting permissions..."
chown -R lens-node:lens-node "$MOUNT_POINT"
chmod 755 "$MOUNT_POINT"
chmod 755 "$MOUNT_POINT/staging"
chmod 755 "$MOUNT_POINT/approved"
echo "✓ Permissions set"

# 5. Add to fstab for auto-mount on boot
echo "[5/5] Adding to fstab..."
FSTAB_ENTRY="mfsmount $MOUNT_POINT fuse defaults,mfsmaster=$MOOSEFS_MASTER 0 0"
if grep -q "$MOUNT_POINT" /etc/fstab; then
    echo "Already in fstab, skipping..."
else
    echo "$FSTAB_ENTRY" >> /etc/fstab
    echo "✓ Added to fstab"
fi

# Verify
echo ""
echo "=== Verification ==="
df -h | grep mfs
ls -la "$MOUNT_POINT"

echo ""
echo "✅ MooseFS setup complete!"
echo ""
echo "Next steps:"
echo "1. Deploy lens-node binary to /opt/lens-node/"
echo "2. Copy deployment/lens-node.service to /etc/systemd/system/"
echo "3. systemctl daemon-reload"
echo "4. systemctl enable lens-node"
echo "5. systemctl start lens-node"
