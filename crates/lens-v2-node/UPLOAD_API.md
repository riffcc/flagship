# Lens Node v2 - Upload API Documentation

## Overview

The Lens Node v2 Upload API provides a permissioned file upload system with staging, approval workflow, and automatic IPFS Cluster pinning. The system supports role-based access control with four permission levels:

- **User** - Basic access, uploads go to staging area awaiting approval
- **Uploader** - Trusted uploaders whose files are auto-approved and pinned
- **Moderator** - Can upload and approve/reject others' uploads
- **Admin** - Full system access, can grant roles and manage all uploads

## Architecture

```
┌─────────────┐
│   Client    │
└──────┬──────┘
       │ POST /api/v1/upload
       │ (X-Public-Key header)
       ▼
┌─────────────────────────────┐
│     Lens Node Backend       │
│  (Role-based authorization) │
└──────────┬──────────────────┘
           │
           ├─────────► Check role (uploader/moderator/admin?)
           │
           ├─ YES ──► Auto-approve ──► Pin to IPFS Cluster
           │                            └──► Store in staging with metadata
           │
           └─ NO ───► Store in staging ──► Awaiting approval
                      with metadata         (Moderator/Admin action required)
```

## Authentication

All API requests require authentication via the `X-Public-Key` header:

```bash
curl -H "X-Public-Key: YOUR_PUBLIC_KEY_HERE" https://lens.example.com/api/v1/...
```

The public key is used to:
- Identify the user making the request
- Check their role and permissions
- Track who uploaded/approved files

## Roles and Permissions

| Role | Upload | Auto-Approve Own | Approve Others | Grant Roles |
|------|--------|------------------|----------------|-------------|
| User | ❌ | ❌ | ❌ | ❌ |
| Uploader | ✅ | ✅ | ❌ | ❌ |
| Moderator | ✅ | ✅ | ✅ | ❌ |
| Admin | ✅ | ✅ | ✅ | ✅ |

## Endpoints

### 1. Upload File

Upload a file to the staging area. Files from users with uploader/moderator/admin roles are automatically approved and pinned to IPFS Cluster.

**Endpoint:** `POST /api/v1/upload`

**Authentication:** Required (X-Public-Key header)

**Permission:** Uploader, Moderator, or Admin role required

**Request:**
- Content-Type: `multipart/form-data`
- Required fields:
  - `file`: The file to upload
  - `metadata` (optional): JSON metadata object

**Response:**
```json
{
  "success": true,
  "upload_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "approved",
  "ipfs_cid": "QmXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
  "message": "File uploaded and automatically approved"
}
```

**Example:**
```bash
curl -X POST https://lens.example.com/api/v1/upload \
  -H "X-Public-Key: uploader_abc123" \
  -F "file=@/path/to/image.jpg" \
  -F 'metadata={"title":"My Image","description":"A beautiful sunset"}'
```

**Status Codes:**
- `200 OK` - Upload successful
- `400 Bad Request` - No file provided or invalid request
- `401 Unauthorized` - Missing X-Public-Key header
- `403 Forbidden` - User doesn't have upload permission
- `500 Internal Server Error` - Server error (check logs)

### 2. List Pending Uploads

List all uploads awaiting approval. Only moderators and admins can access this endpoint.

**Endpoint:** `GET /api/v1/admin/uploads/pending`

**Authentication:** Required (X-Public-Key header)

**Permission:** Moderator or Admin role required

**Response:**
```json
{
  "total": 2,
  "uploads": [
    {
      "upload_id": "550e8400-e29b-41d4-a716-446655440000",
      "uploader_public_key": "user_xyz789",
      "timestamp": "2025-10-10T14:30:00Z",
      "filename": "document.pdf",
      "size_bytes": 1048576,
      "mime_type": "application/pdf",
      "status": "pending",
      "auto_approved": false,
      "approved_by": null,
      "approved_at": null,
      "ipfs_cid": null,
      "additional_metadata": {
        "title": "Important Document"
      }
    }
  ]
}
```

**Example:**
```bash
curl -X GET https://lens.example.com/api/v1/admin/uploads/pending \
  -H "X-Public-Key: moderator_def456"
```

**Status Codes:**
- `200 OK` - Success
- `401 Unauthorized` - Missing X-Public-Key header
- `403 Forbidden` - User is not a moderator or admin

### 3. Get Upload Details

Get detailed information about a specific upload.

**Endpoint:** `GET /api/v1/admin/uploads/:id`

**Authentication:** Required (X-Public-Key header)

**Permission:** Moderator or Admin role required

**Response:**
```json
{
  "upload_id": "550e8400-e29b-41d4-a716-446655440000",
  "uploader_public_key": "user_xyz789",
  "timestamp": "2025-10-10T14:30:00Z",
  "filename": "document.pdf",
  "size_bytes": 1048576,
  "mime_type": "application/pdf",
  "status": "pending",
  "auto_approved": false,
  "approved_by": null,
  "approved_at": null,
  "ipfs_cid": null,
  "additional_metadata": {
    "title": "Important Document"
  }
}
```

**Example:**
```bash
curl -X GET https://lens.example.com/api/v1/admin/uploads/550e8400-e29b-41d4-a716-446655440000 \
  -H "X-Public-Key: moderator_def456"
```

**Status Codes:**
- `200 OK` - Success
- `401 Unauthorized` - Missing X-Public-Key header
- `403 Forbidden` - User is not a moderator or admin
- `404 Not Found` - Upload ID doesn't exist

### 4. Approve Upload

Approve a pending upload and pin it to IPFS Cluster.

**Endpoint:** `POST /api/v1/admin/uploads/:id/approve`

**Authentication:** Required (X-Public-Key header)

**Permission:** Moderator or Admin role required

**Response:**
```json
{
  "success": true,
  "upload_id": "550e8400-e29b-41d4-a716-446655440000",
  "ipfs_cid": "QmXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
  "message": "Upload approved and pinned to IPFS"
}
```

**Example:**
```bash
curl -X POST https://lens.example.com/api/v1/admin/uploads/550e8400-e29b-41d4-a716-446655440000/approve \
  -H "X-Public-Key: moderator_def456"
```

**Status Codes:**
- `200 OK` - Approval successful
- `400 Bad Request` - Upload already approved
- `401 Unauthorized` - Missing X-Public-Key header
- `403 Forbidden` - User is not a moderator or admin
- `404 Not Found` - Upload ID doesn't exist
- `500 Internal Server Error` - Failed to pin to IPFS (check IPFS Cluster)

### 5. Reject Upload

Reject a pending upload and remove it from the staging area.

**Endpoint:** `POST /api/v1/admin/uploads/:id/reject`

**Authentication:** Required (X-Public-Key header)

**Permission:** Moderator or Admin role required

**Response:**
```json
{
  "success": true,
  "upload_id": "550e8400-e29b-41d4-a716-446655440000",
  "message": "Upload rejected and removed"
}
```

**Example:**
```bash
curl -X POST https://lens.example.com/api/v1/admin/uploads/550e8400-e29b-41d4-a716-446655440000/reject \
  -H "X-Public-Key: moderator_def456"
```

**Status Codes:**
- `200 OK` - Rejection successful
- `401 Unauthorized` - Missing X-Public-Key header
- `403 Forbidden` - User is not a moderator or admin
- `404 Not Found` - Upload ID doesn't exist

## Role Management Endpoints

### 1. Authorize Admin

Grant admin privileges to a public key. This endpoint is open during initial setup but should be protected in production.

**Endpoint:** `POST /api/v1/admin/authorize`

**Request:**
```json
{
  "publicKey": "new_admin_abc123"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Public key new_admin_abc123 authorized as admin"
}
```

**Example:**
```bash
curl -X POST https://lens.example.com/api/v1/admin/authorize \
  -H "Content-Type: application/json" \
  -d '{"publicKey": "new_admin_abc123"}'
```

### 2. Grant Uploader Role

Grant uploader role to a user (admin only).

**Endpoint:** `POST /api/v1/admin/roles/uploader`

**Authentication:** Required (X-Public-Key header)

**Permission:** Admin role required

**Request:**
```json
{
  "publicKey": "user_to_promote"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Uploader role granted to user_to_promote"
}
```

**Example:**
```bash
curl -X POST https://lens.example.com/api/v1/admin/roles/uploader \
  -H "X-Public-Key: admin_xyz789" \
  -H "Content-Type: application/json" \
  -d '{"publicKey": "user_to_promote"}'
```

### 3. Grant Moderator Role

Grant moderator role to a user (admin only).

**Endpoint:** `POST /api/v1/admin/roles/moderator`

**Authentication:** Required (X-Public-Key header)

**Permission:** Admin role required

**Request:**
```json
{
  "publicKey": "user_to_promote"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Moderator role granted to user_to_promote"
}
```

**Example:**
```bash
curl -X POST https://lens.example.com/api/v1/admin/roles/moderator \
  -H "X-Public-Key: admin_xyz789" \
  -H "Content-Type: application/json" \
  -d '{"publicKey": "user_to_promote"}'
```

### 4. Check Account Status

Check the roles and permissions of a specific public key.

**Endpoint:** `GET /api/v1/account/:public_key`

**Response:**
```json
{
  "isAdmin": true,
  "roles": ["admin"],
  "permissions": ["admin", "upload", "moderate"]
}
```

**Example:**
```bash
curl -X GET https://lens.example.com/api/v1/account/admin_xyz789
```

## File Storage Structure

Uploaded files are stored in a staging directory with associated metadata:

```
data/staging/
├── 550e8400-e29b-41d4-a716-446655440000/
│   └── image.jpg
├── 550e8400-e29b-41d4-a716-446655440000.meta
├── 661f9511-f3ac-52e5-b827-557766551111/
│   └── document.pdf
└── 661f9511-f3ac-52e5-b827-557766551111.meta
```

Each `.meta` file contains JSON metadata:

```json
{
  "upload_id": "550e8400-e29b-41d4-a716-446655440000",
  "uploader_public_key": "uploader_abc123",
  "timestamp": "2025-10-10T14:30:00Z",
  "filename": "image.jpg",
  "size_bytes": 524288,
  "mime_type": "image/jpeg",
  "status": "approved",
  "auto_approved": true,
  "approved_by": "uploader_abc123",
  "approved_at": "2025-10-10T14:30:00Z",
  "ipfs_cid": "QmXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
  "additional_metadata": {
    "title": "Beautiful Sunset",
    "tags": ["nature", "photography"]
  }
}
```

## Environment Variables

Configure the upload system with these environment variables:

```bash
# Upload staging directory (default: data/staging)
UPLOAD_STAGING_DIR=/path/to/staging

# Upload approved directory (default: data/approved)
UPLOAD_APPROVED_DIR=/path/to/approved

# Server port (default: 5002)
PORT=5002

# RocksDB path (default: .lens-node-data/rocksdb)
DB_PATH=/path/to/rocksdb

# P2P relay URL (default: ws://localhost:5002/api/v1/relay/ws)
LENS_RELAY_URL=ws://relay.example.com/api/v1/relay/ws
```

### Stateless Multi-Node Deployment

For production deployments with multiple lens-node instances, use **MooseFS for shared state**:

```bash
# All nodes use the SAME MooseFS paths
UPLOAD_STAGING_DIR=/mnt/mfs/lens-uploads/staging
UPLOAD_APPROVED_DIR=/mnt/mfs/lens-uploads/approved
```

This enables:
- ✅ Upload to any node
- ✅ Approve from any node
- ✅ List pending from any node
- ✅ No sticky sessions required
- ✅ True horizontal scaling

**See [STATELESS_DEPLOYMENT.md](./STATELESS_DEPLOYMENT.md) for complete multi-node setup guide.**

## Workflow Examples

### Example 1: Trusted Uploader Workflow

1. User has "uploader" role
2. Uploads file → immediately approved and pinned to IPFS
3. No moderator action required

```bash
# Upload file (auto-approved)
curl -X POST https://lens.example.com/api/v1/upload \
  -H "X-Public-Key: uploader_abc123" \
  -F "file=@photo.jpg"

# Response includes IPFS CID immediately:
# {
#   "success": true,
#   "upload_id": "550e8400-...",
#   "status": "approved",
#   "ipfs_cid": "QmXXXX...",
#   "message": "File uploaded and automatically approved"
# }
```

### Example 2: Regular User Workflow (Requires Approval)

1. User without uploader role uploads file
2. File goes to staging, awaits approval
3. Moderator/admin reviews and approves
4. File is pinned to IPFS

```bash
# User uploads (pending approval)
curl -X POST https://lens.example.com/api/v1/upload \
  -H "X-Public-Key: regular_user" \
  -F "file=@document.pdf"

# Response shows pending status:
# {
#   "success": true,
#   "upload_id": "550e8400-...",
#   "status": "pending",
#   "ipfs_cid": null,
#   "message": "File uploaded and awaiting approval"
# }

# Moderator lists pending uploads
curl -X GET https://lens.example.com/api/v1/admin/uploads/pending \
  -H "X-Public-Key: moderator_def456"

# Moderator reviews and approves
curl -X POST https://lens.example.com/api/v1/admin/uploads/550e8400-.../approve \
  -H "X-Public-Key: moderator_def456"

# Response includes IPFS CID:
# {
#   "success": true,
#   "upload_id": "550e8400-...",
#   "ipfs_cid": "QmXXXX...",
#   "message": "Upload approved and pinned to IPFS"
# }
```

### Example 3: Admin Setup and Role Management

```bash
# Initial admin authorization (first-time setup)
curl -X POST https://lens.example.com/api/v1/admin/authorize \
  -H "Content-Type: application/json" \
  -d '{"publicKey": "admin_xyz789"}'

# Admin grants uploader role to trusted user
curl -X POST https://lens.example.com/api/v1/admin/roles/uploader \
  -H "X-Public-Key: admin_xyz789" \
  -H "Content-Type: application/json" \
  -d '{"publicKey": "trusted_user_123"}'

# Admin grants moderator role
curl -X POST https://lens.example.com/api/v1/admin/roles/moderator \
  -H "X-Public-Key: admin_xyz789" \
  -H "Content-Type: application/json" \
  -d '{"publicKey": "moderator_456"}'

# Check user's permissions
curl -X GET https://lens.example.com/api/v1/account/trusted_user_123

# Response:
# {
#   "isAdmin": false,
#   "roles": ["uploader"],
#   "permissions": ["upload"]
# }
```

## IPFS Cluster Integration

Approved uploads are automatically pinned to IPFS Cluster with **descriptive names** for easy identification:

```bash
ipfs-cluster-ctl add --name "title | filename | uploader | timestamp" /path/to/file
```

**Pin Naming Format:**

Pins are named with the following structure:
```
<title> | <filename> | <uploader_short> | <timestamp>
```

**Example pin names:**
```
Brad Sucks Album | album-cover.jpg | a1b2c3d4 | 2025-10-10 14:30 UTC
My Photo Collection | sunset.jpg | x9y8z7w6 | 2025-10-10 15:45 UTC
document.pdf | document.pdf | m5n4k3l2 | 2025-10-10 16:20 UTC
```

**Benefits:**
- ✅ Easily identify uploads in IPFS Cluster
- ✅ See who uploaded each file at a glance
- ✅ Search pins by title or uploader
- ✅ Track upload dates without external database

**List pins by name:**
```bash
# List all pins with names
ipfs-cluster-ctl pin ls --name

# Search for specific uploader
ipfs-cluster-ctl pin ls --name | grep "a1b2c3d4"

# Search by date
ipfs-cluster-ctl pin ls --name | grep "2025-10-10"

# Search by title
ipfs-cluster-ctl pin ls --name | grep "Brad Sucks"
```

**Requirements:**
- `ipfs-cluster-ctl` must be installed and in PATH
- IPFS Cluster must be running and accessible
- IPFS Cluster version >= 1.0.0 (for pin naming support)
- The lens-node process must have permission to execute `ipfs-cluster-ctl`

**Configuration:**
The IPFS Cluster service should be configured with:
- CRDT consensus mode (for multi-node clusters)
- Proper peering between cluster nodes
- Sufficient replication factor

See the IPFS Cluster documentation for setup details.

## Error Handling

All endpoints return consistent error responses:

```json
{
  "success": false,
  "error": "Description of what went wrong"
}
```

Common errors:
- `Missing X-Public-Key header` - Authentication required
- `You don't have permission to upload files` - User needs uploader/moderator/admin role
- `Only moderators and admins can approve uploads` - Insufficient permissions
- `Upload not found` - Invalid upload ID
- `Failed to pin to IPFS` - IPFS Cluster unavailable or error

## Security Considerations

1. **Authentication**: The X-Public-Key header is used for authentication. In production, this should be cryptographically signed to prevent impersonation.

2. **Rate Limiting**: Consider implementing rate limits on upload endpoints to prevent abuse.

3. **File Validation**: Validate file types, sizes, and content before accepting uploads.

4. **Admin Authorization**: The `/api/v1/admin/authorize` endpoint should be restricted (IP whitelist, VPN, or disabled after initial setup).

5. **HTTPS**: Always use HTTPS in production to encrypt authentication headers.

6. **CORS**: Configure CORS appropriately to restrict which domains can access the API.

7. **Upload Limits**: Configure reasonable limits in Nginx:
   - `client_max_body_size` - Maximum upload size
   - `proxy_read_timeout` - Timeout for large uploads

## Testing

See the `nginx-example.conf` file for curl examples and testing commands.

For integration testing, you can use the provided test suite:

```bash
cd /opt/castle/workspace/flagship/crates/lens-v2-node
cargo test
```

## Support

For issues, questions, or contributions:
- GitHub: https://github.com/riff-cc/flagship
- Documentation: https://docs.riff.cc/lens-node-v2/
- Community: https://community.riff.cc/
