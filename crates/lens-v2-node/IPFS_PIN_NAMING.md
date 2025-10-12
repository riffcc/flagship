# IPFS Cluster Pin Naming

## Overview

All uploads approved through lens-node are automatically pinned to IPFS Cluster with **descriptive, searchable names** that include metadata about the upload.

## Pin Name Format

```
<title> | <filename> | <uploader> | <timestamp>
```

### Components

| Field | Description | Example |
|-------|-------------|---------|
| **Title** | User-provided title from metadata, or filename if no title | `Brad Sucks Album` |
| **Filename** | Original filename | `album-cover.jpg` |
| **Uploader** | First 8 characters of uploader's public key | `a1b2c3d4` |
| **Timestamp** | Upload timestamp in UTC | `2025-10-10 14:30 UTC` |

## Examples

### Auto-Approved Upload (with title)
```
Brad Sucks - Full Discography | discography.zip | a1b2c3d4 | 2025-10-10 14:30 UTC
```

### Auto-Approved Upload (no title)
```
sunset-photo.jpg | sunset-photo.jpg | x9y8z7w6 | 2025-10-10 15:45 UTC
```

### Moderator-Approved Upload
```
Important Document | contract.pdf | m5n4k3l2 | 2025-10-09 10:15 UTC
```

## Searching Pins

### List All Pins with Names

```bash
ipfs-cluster-ctl pin ls --name
```

**Output:**
```
QmXXX... | Brad Sucks Album | album-cover.jpg | a1b2c3d4 | 2025-10-10 14:30 UTC
QmYYY... | My Photo Collection | sunset.jpg | x9y8z7w6 | 2025-10-10 15:45 UTC
QmZZZ... | document.pdf | document.pdf | m5n4k3l2 | 2025-10-10 16:20 UTC
```

### Search by Uploader

Find all uploads from a specific user:

```bash
ipfs-cluster-ctl pin ls --name | grep "a1b2c3d4"
```

### Search by Date

Find all uploads from a specific date:

```bash
ipfs-cluster-ctl pin ls --name | grep "2025-10-10"
```

### Search by Title/Keyword

Find uploads matching a keyword:

```bash
ipfs-cluster-ctl pin ls --name | grep -i "brad sucks"
ipfs-cluster-ctl pin ls --name | grep -i "photo"
ipfs-cluster-ctl pin ls --name | grep -i "album"
```

### Search by File Extension

```bash
ipfs-cluster-ctl pin ls --name | grep "\.jpg"
ipfs-cluster-ctl pin ls --name | grep "\.pdf"
ipfs-cluster-ctl pin ls --name | grep "\.zip"
```

### Search by Time Range

```bash
# All uploads from October 2025
ipfs-cluster-ctl pin ls --name | grep "2025-10-"

# All uploads from a specific month and day
ipfs-cluster-ctl pin ls --name | grep "2025-10-10"
```

## Audit Trail

Pin names provide a built-in audit trail:

```bash
# Who uploaded what, when?
ipfs-cluster-ctl pin ls --name | awk -F'|' '{print $3 " uploaded " $2 " on " $4}'
```

**Output:**
```
 a1b2c3d4  uploaded  album-cover.jpg  on  2025-10-10 14:30 UTC
 x9y8z7w6  uploaded  sunset.jpg  on  2025-10-10 15:45 UTC
 m5n4k3l2  uploaded  document.pdf  on  2025-10-10 16:20 UTC
```

## Pin Name in Upload Logs

When a file is uploaded and pinned, the lens-node logs show:

```
INFO lens_v2_node::routes::upload: Pinning to IPFS Cluster with name: Brad Sucks Album | album-cover.jpg | a1b2c3d4 | 2025-10-10 14:30 UTC
INFO lens_v2_node::routes::upload: Auto-approved upload 550e8400-... pinned to IPFS: QmXXX...
```

## JSON Output

Get pin information as JSON for programmatic processing:

```bash
ipfs-cluster-ctl status --filter pinned | jq -r '.[] | "\(.name) | \(.cid)"'
```

## Use Cases

### 1. Content Moderation

Moderators can quickly see who uploaded potentially problematic content:

```bash
ipfs-cluster-ctl pin ls --name | grep "flagged-user-key"
```

### 2. User Activity Reports

Generate reports of user upload activity:

```bash
# Count uploads per user
ipfs-cluster-ctl pin ls --name | awk -F'|' '{print $3}' | sort | uniq -c
```

### 3. Storage Audits

See what types of files are being uploaded:

```bash
# Group by file extension
ipfs-cluster-ctl pin ls --name | grep -oP '\.[\w]+(?= \|)' | sort | uniq -c
```

### 4. Timeline Analysis

Track upload patterns over time:

```bash
# Uploads per day
ipfs-cluster-ctl pin ls --name | grep -oP '\d{4}-\d{2}-\d{2}' | sort | uniq -c
```

### 5. Collection Management

Find all files in a collection:

```bash
# All Brad Sucks uploads
ipfs-cluster-ctl pin ls --name | grep -i "brad sucks"

# Get their CIDs for further processing
ipfs-cluster-ctl pin ls --name | grep -i "brad sucks" | awk '{print $1}'
```

## Migration from Unnamed Pins

If you have existing unnamed pins, you can identify them:

```bash
# Show all pins without names (empty name field)
ipfs-cluster-ctl pin ls --name | grep -E "^[^ ]+ \|  \|"
```

## Best Practices

1. **Encourage Titles**: Ask users to provide descriptive titles in the metadata field
2. **Consistent Formatting**: The pipe-delimited format makes parsing easy
3. **Searchable Keywords**: Use searchable terms in titles (artist names, project names, etc.)
4. **Keep Public Keys**: Store the full public key in your user database for lookup
5. **Backup Pin Lists**: Periodically export pin lists with names for backup:
   ```bash
   ipfs-cluster-ctl pin ls --name > pins-backup-$(date +%Y%m%d).txt
   ```

## Technical Details

### Implementation

The pin name is constructed in `upload.rs:123-148`:

```rust
let title = metadata
    .additional_metadata
    .as_ref()
    .and_then(|m| m.get("title"))
    .and_then(|t| t.as_str())
    .unwrap_or(&metadata.filename);

let uploader_short = if metadata.uploader_public_key.len() > 8 {
    &metadata.uploader_public_key[..8]
} else {
    &metadata.uploader_public_key
};

let timestamp = metadata.timestamp.format("%Y-%m-%d %H:%M UTC");

let pin_name = format!(
    "{} | {} | {} | {}",
    title,
    metadata.filename,
    uploader_short,
    timestamp
);
```

### IPFS Cluster Command

```bash
ipfs-cluster-ctl add --name "Brad Sucks Album | album.zip | a1b2c3d4 | 2025-10-10 14:30 UTC" /path/to/file
```

### Cross-Site Replication

Pin names are **replicated across all IPFS Cluster nodes**, including remote sites. This means you can search for uploads on any cluster node and find the same metadata.

## Limitations

- Pin names are metadata only - they don't affect content addressing (CID remains the same)
- Pin names are not indexed by IPFS - searching requires iterating through the pin list
- Very long titles may be truncated (IPFS Cluster may have limits on name length)

## Future Enhancements

Possible future improvements:

- Add file hash to pin name for verification
- Include upload status (auto-approved vs manually approved)
- Add tags/categories from metadata
- Include approver's public key for manually approved uploads

## Summary

Pin naming provides:
- ✅ **Searchable metadata** without external database
- ✅ **Built-in audit trail** of who uploaded what and when
- ✅ **Easy content discovery** through grep/awk/jq
- ✅ **Cross-site visibility** via IPFS Cluster replication
- ✅ **No performance impact** - names are just metadata

This makes managing hundreds or thousands of uploads much more practical! 🎯
