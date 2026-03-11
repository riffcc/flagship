# Upload Service Architecture

## Overview

The Upload Service is an external microservice that handles file uploads for Riff.CC. It operates independently from the lens-node backend, using ed25519 signature-based authentication to verify user identity and permissions.

**Endpoint:** `https://uploads.global.riff.cc/upload`

## Why Separate from Lens Node?

- **Separation of concerns** - Lens nodes handle content metadata and P2P sync, not file storage
- **Independent scaling** - Upload infrastructure can be scaled separately from content nodes
- **Storage flexibility** - Easy to swap storage backends (IPFS, S3, etc.) without touching lens-node
- **Security** - Nginx can validate signatures and route to appropriate storage based on user roles

## Request Format

### HTTP Method
`POST /upload`

### Headers

| Header | Required | Description |
|--------|----------|-------------|
| `X-Public-Key` | Yes | User's ed25519 public key in format `ed25519p/{hex}` |
| `X-Signature` | Yes | ed25519 signature of the payload (hex encoded) |
| `X-Timestamp` | Yes | Unix timestamp in milliseconds when signature was created |

### Signature Payload

The client signs the following string:
```
{timestamp}:{publicKey}:{fileName}:{fileSize}
```

Example:
```
1728756000000:ed25519p/661f20293170ac54c64abcca6c24c4c773245e469904f200b8b633d1c4a5888b:song.mp3:4567890
```

### Body

Standard multipart/form-data with:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `file` | File | Yes | The file being uploaded |
| `metadata` | JSON string | Optional | Additional metadata (see below) |

### Metadata Format

```json
{
  "title": "My Cool Song",
  "description": "A description of the upload",
  "path": "folder/subfolder/file.mp3",
  "fileName": "file.mp3",
  "batchIndex": 1,
  "batchTotal": 5
}
```

## Response Format

### Success Response (200 OK)

```json
{
  "success": true,
  "upload_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "approved",
  "ipfs_cid": "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG",
  "message": "File uploaded and approved"
}
```

### Status Values

- `"pending"` - Upload awaiting moderator approval (normal users)
- `"approved"` - Upload automatically approved and pinned to IPFS (trusted users)
- `"rejected"` - Upload rejected (should not normally occur on upload)

### Error Responses

#### 400 Bad Request
```json
{
  "success": false,
  "error": "Invalid signature format"
}
```

#### 401 Unauthorized
```json
{
  "success": false,
  "error": "Signature verification failed"
}
```

#### 403 Forbidden
```json
{
  "success": false,
  "error": "User does not have upload permission"
}
```

#### 413 Payload Too Large
```json
{
  "success": false,
  "error": "File size exceeds maximum allowed (100MB)"
}
```

#### 429 Too Many Requests
```json
{
  "success": false,
  "error": "Rate limit exceeded. Try again in 60 seconds"
}
```

#### 500 Internal Server Error
```json
{
  "success": false,
  "error": "Failed to pin to IPFS: connection timeout"
}
```

## Implementation Requirements

### 1. Signature Verification

The service MUST verify the ed25519 signature before processing any upload:

```rust
// Pseudo-code
fn verify_upload_signature(
    public_key: &str,
    signature: &str,
    timestamp: u64,
    file_name: &str,
    file_size: u64,
) -> Result<bool> {
    // 1. Validate timestamp is recent (within 5 minutes)
    let now = current_timestamp_ms();
    if (now - timestamp).abs() > 300_000 {
        return Err("Timestamp expired");
    }

    // 2. Reconstruct signature payload
    let payload = format!("{}:{}:{}:{}", timestamp, public_key, file_name, file_size);

    // 3. Strip ed25519p/ prefix and convert hex to bytes
    let pub_key_hex = public_key.strip_prefix("ed25519p/")
        .ok_or("Invalid public key format")?;
    let pub_key_bytes = hex::decode(pub_key_hex)?;
    let sig_bytes = hex::decode(signature)?;

    // 4. Verify ed25519 signature
    use ed25519_dalek::{Verifier, PublicKey, Signature};
    let public_key = PublicKey::from_bytes(&pub_key_bytes)?;
    let signature = Signature::from_bytes(&sig_bytes)?;

    public_key.verify(payload.as_bytes(), &signature)
        .map(|_| true)
        .map_err(|_| "Signature verification failed")
}
```

### 2. Permission Check

Query the lens-node to verify user permissions:

```bash
GET https://api.global.riff.cc/api/v1/account/{public_key}
```

Response:
```json
{
  "publicKey": "ed25519p/661f20293170ac54c64abcca6c24c4c773245e469904f200b8b633d1c4a5888b",
  "permissions": ["upload", "create_release"],
  "roles": ["uploader"],
  "isAdmin": false
}
```

**Auto-approval logic:**
- User has `"upload"` permission → Can upload
- User has role `"uploader"`, `"moderator"`, or `isAdmin: true` → Auto-approve + pin to IPFS immediately
- Otherwise → Upload to landing pad for manual approval

### 3. Storage Routing

#### Normal Users (Pending Approval)
```
POST /upload → Upload Service → Landing Pad (filesystem staging)
                                   ↓
                              Manual Review by Moderator
                                   ↓
                              PIN to IPFS Cluster
```

#### Trusted Users (Auto-Approved)
```
POST /upload → Upload Service → IPFS Cluster (direct pin)
```

**Landing Pad Structure:**
```
/var/uploads/staging/
├── {upload_id}/
│   ├── file.mp3              # The uploaded file
│   └── metadata.json         # Upload metadata
```

**Metadata Example:**
```json
{
  "upload_id": "550e8400-e29b-41d4-a716-446655440000",
  "uploader_public_key": "ed25519p/661f20293170ac54c64abcca6c24c4c773245e469904f200b8b633d1c4a5888b",
  "timestamp": "2025-10-12T15:30:00Z",
  "filename": "song.mp3",
  "size_bytes": 4567890,
  "mime_type": "audio/mpeg",
  "status": "pending",
  "auto_approved": false,
  "additional_metadata": {
    "title": "My Cool Song",
    "description": "A great track"
  }
}
```

### 4. IPFS Cluster Integration

For auto-approved uploads, pin directly to IPFS Cluster:

```bash
ipfs-cluster-ctl add \
  --name "My Cool Song | song.mp3 | ed25519p/661f2029 | 2025-10-12 15:30 UTC" \
  /path/to/file.mp3
```

Parse the CID from output:
```
added QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG song.mp3
```

### 5. Rate Limiting

Implement per-user rate limits:
- **10 uploads per minute** - Normal users
- **100 uploads per minute** - Trusted users (uploader role)
- **Unlimited** - Admins

Use Redis or in-memory cache to track:
```redis
INCR upload:rate:{public_key}:{minute_bucket}
EXPIRE upload:rate:{public_key}:{minute_bucket} 60
```

### 6. File Size Limits

- **Max file size:** 100MB per file
- **Max batch size:** 500MB total per request
- **Max files per batch:** 100 files

These can be configured via environment variables.

### 7. Security Considerations

#### CORS Headers
```nginx
Access-Control-Allow-Origin: https://global.riff.cc
Access-Control-Allow-Methods: POST, OPTIONS
Access-Control-Allow-Headers: X-Public-Key, X-Signature, X-Timestamp, Content-Type
Access-Control-Max-Age: 86400
```

#### Input Validation
- Validate `X-Public-Key` matches format `ed25519p/[a-f0-9]{64}`
- Validate `X-Signature` is 128 hex characters (64 bytes)
- Validate `X-Timestamp` is within ±5 minutes of server time
- Sanitize file names (remove path traversal attempts)
- Validate MIME types against allowed list

#### Malware Scanning
Consider integrating ClamAV or similar:
```bash
clamscan --infected --remove /path/to/uploaded/file
```

## Nginx Configuration

### Signature Validation Module

You can implement a simple validation script:

```nginx
location /upload {
    # Validate signature using Lua or external auth service
    access_by_lua_block {
        local signature = ngx.var.http_x_signature
        local public_key = ngx.var.http_x_public_key
        local timestamp = ngx.var.http_x_timestamp

        -- Call validation service
        local res = ngx.location.capture("/internal/validate", {
            method = ngx.HTTP_POST,
            body = ngx.encode_args({
                signature = signature,
                public_key = public_key,
                timestamp = timestamp,
            })
        })

        if res.status ~= 200 then
            ngx.status = 401
            ngx.say('{"error": "Invalid signature"}')
            return ngx.exit(401)
        end
    }

    # Check user role and route accordingly
    proxy_pass http://upload_backend;
}
```

### Role-Based Routing

```nginx
map $user_role $upload_backend {
    "admin"     "ipfs_cluster";
    "uploader"  "ipfs_cluster";
    "moderator" "ipfs_cluster";
    default     "landing_pad";
}

upstream ipfs_cluster {
    server ipfs-cluster-1:9094;
    server ipfs-cluster-2:9094;
    server ipfs-cluster-3:9094;
}

upstream landing_pad {
    server landing-pad-1:8080;
    server landing-pad-2:8080;
}
```

## Deployment Architecture

```
┌─────────────┐
│   Browser   │
│  (Flagship) │
└──────┬──────┘
       │ POST /upload
       │ (signed with ed25519)
       ▼
┌─────────────┐
│    Nginx    │
│  (Validate) │
└──────┬──────┘
       │
       ├─► Normal User → Landing Pad → Manual Approval → IPFS Cluster
       │
       └─► Trusted User ──────────────────────────────► IPFS Cluster
```

## Monitoring & Logging

### Metrics to Track
- Upload success/failure rate
- Average upload time
- IPFS pin success rate
- Signature validation failures
- Rate limit hits
- Storage usage (landing pad)

### Log Format
```json
{
  "timestamp": "2025-10-12T15:30:00Z",
  "upload_id": "550e8400-e29b-41d4-a716-446655440000",
  "public_key": "ed25519p/661f2029...",
  "filename": "song.mp3",
  "size_bytes": 4567890,
  "status": "approved",
  "ipfs_cid": "QmYwAPJzv...",
  "processing_time_ms": 1234,
  "auto_approved": true
}
```

## Environment Variables

```bash
# Upload Service Configuration
UPLOAD_MAX_FILE_SIZE=104857600          # 100MB in bytes
UPLOAD_MAX_BATCH_SIZE=524288000         # 500MB in bytes
UPLOAD_MAX_FILES_PER_BATCH=100
UPLOAD_STAGING_DIR=/var/uploads/staging
UPLOAD_RATE_LIMIT_NORMAL=10             # uploads per minute
UPLOAD_RATE_LIMIT_TRUSTED=100           # uploads per minute

# IPFS Cluster
IPFS_CLUSTER_API=http://localhost:9094
IPFS_CLUSTER_CTL=/usr/local/bin/ipfs-cluster-ctl

# Lens Node API (for permission checks)
LENS_NODE_API=https://api.global.riff.cc/api/v1

# Redis (for rate limiting)
REDIS_URL=redis://localhost:6379

# Security
SIGNATURE_TIMESTAMP_TOLERANCE_MS=300000  # 5 minutes
ALLOWED_MIME_TYPES=audio/*,video/*,image/*,application/pdf

# Monitoring
METRICS_PORT=9090
LOG_LEVEL=info
```

## API Testing

### Example cURL Request

```bash
# Generate signature (using Node.js or similar)
TIMESTAMP=$(date +%s000)
PAYLOAD="$TIMESTAMP:ed25519p/661f20293170ac54c64abcca6c24c4c773245e469904f200b8b633d1c4a5888b:test.mp3:12345"
SIGNATURE=$(echo -n "$PAYLOAD" | openssl dgst -sha512 -sign private.pem -hex)

# Upload file
curl -X POST https://uploads.global.riff.cc/upload \
  -H "X-Public-Key: ed25519p/661f20293170ac54c64abcca6c24c4c773245e469904f200b8b633d1c4a5888b" \
  -H "X-Signature: $SIGNATURE" \
  -H "X-Timestamp: $TIMESTAMP" \
  -F "file=@test.mp3" \
  -F 'metadata={"title":"Test Song","description":"Testing upload"}'
```

## Future Enhancements

1. **Chunked Uploads** - Support resumable uploads for large files
2. **Multi-region** - Deploy upload service in multiple regions for low latency
3. **CDN Integration** - Cache popular uploads on CDN edge nodes
4. **Virus Scanning** - Integrate real-time malware detection
5. **Content Moderation** - AI-based content policy enforcement
6. **Deduplication** - Check if file already exists by hash before uploading
7. **Compression** - Auto-compress images/videos before storage
8. **Encryption** - Support client-side encryption for private uploads

## Reference Implementation

A reference implementation in Rust using Axum:
- Repository: TBD
- Language: Rust
- Framework: Axum
- Dependencies: ed25519-dalek, tokio, redis, reqwest

## Questions?

For questions or clarifications about the Upload Service specification, please:
- Open an issue on the flagship repository
- Contact the Riff.CC development team
- Refer to the main architecture docs at `/docs/ARCHITECTURE.md`
