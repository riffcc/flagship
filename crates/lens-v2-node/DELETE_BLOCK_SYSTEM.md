# DeleteBlock System - Proof of Erasure

## Overview

The DeleteBlock system implements **proof-of-erasure** for distributed content deletion with consensus. When an admin deletes spam/abuse content, the deletion propagates through the network, each node confirms erasure, and once consensus is reached, even the delete block itself is pruned.

## Architecture

### 1. DeleteBlock Structure

```rust
pub struct DeleteBlock {
    pub id: String,                          // delete-{hash}
    pub deleted_block_ids: Vec<String>,      // Blocks to delete
    pub reason: DeleteReason,                // Why deleted
    pub deleted_by: String,                  // Admin public key
    pub timestamp: u64,
    pub erasure_confirmations: HashSet<String>, // Peers who confirmed
    pub required_confirmations: usize,       // Threshold for consensus
    pub consensus_achieved: bool,
}
```

### 2. Delete Reasons

- `Spam` - Spam content
- `Abuse` - Abusive/harmful content
- `Copyright` - Copyright violation
- `Malware` - Security threat
- `UserRequest` - User-requested deletion
- `Other(String)` - Custom reason

### 3. Erasure Confirmation

```rust
pub struct ErasureConfirmation {
    pub delete_block_id: String,
    pub peer_id: String,
    pub erased_block_ids: Vec<String>,
    pub timestamp: u64,
    pub signature: Option<String>,  // Cryptographic proof
}
```

## Flow

### Phase 1: Delete Initiation (Admin Only)

1. Admin calls `DELETE /api/v1/releases/{id}` with reason
2. Node creates `DeleteBlock` with:
   - Target block IDs to delete
   - Reason for deletion
   - Required confirmations = peer_count
3. Node broadcasts `DeleteBlock` to all peers
4. Node stores `DeleteBlock` in database (key: `delete:{delete_block_id}`)

### Phase 2: Propagation & Erasure

Each peer that receives `DeleteBlock`:

1. Validates delete is from admin
2. Deletes target blocks from local storage
3. Removes blocks from SPORE
4. Sends `ErasureConfirmation` back to network
5. Updates own copy of `DeleteBlock` with confirmation

### Phase 3: Consensus Tracking

As `ErasureConfirmation` messages arrive:

1. Add peer ID to `erasure_confirmations` set
2. Check if `confirmations.len() >= required_confirmations`
3. If yes: mark `consensus_achieved = true`

### Phase 4: Proof of Erasure Complete

Once consensus achieved:

1. All nodes have confirmed deletion
2. Target blocks are gone from all nodes
3. Delete block itself can now be safely pruned
4. Remove `delete:{delete_block_id}` from database
5. Remove delete block from SPORE

## NetworkEvent Flow

```
Admin Node                          Peer Nodes
    |                                    |
    | DELETE /releases/{id}              |
    |                                    |
    | Creates DeleteBlock                |
    | broadcast_block(delete_block) ---->| DeleteBlockReceived
    |                                    | - Delete target blocks
    |                                    | - Remove from SPORE
    |                                    | - Send ErasureConfirmation
    | <--------------------------------- |
    | ErasureConfirmationReceived        |
    | - Update DeleteBlock               |
    | - Check consensus                  |
    |                                    |
    | [Consensus Achieved]               |
    | - Prune DeleteBlock                |
    | - Remove from SPORE                |
    |                                    |
```

## Database Storage

### Active DeleteBlocks

```
Key: "delete:{delete_block_id}"
Value: DeleteBlock (JSON)
```

### Consensus Tracking

DeleteBlocks stay in database until consensus achieved, then are pruned.

## Security Considerations

1. **Admin-Only**: Only admins can create DeleteBlocks
2. **Signatures**: ErasureConfirmations can be signed for proof
3. **Immutable History**: DeleteBlocks are logged before pruning
4. **Threshold**: Requires majority consensus (configurable)
5. **Audit Trail**: All deletions logged with reason

## Implementation Status

### ✅ Completed

- DeleteBlock structure (`delete_block.rs`)
- ErasureConfirmation structure
- NetworkEvent enum extended
- Basic consensus tracking

### 🚧 In Progress

- Network broadcast methods
- Sync orchestrator handlers
- Database persistence
- SPORE pruning

### 📋 TODO

- Relay routing for delete_block messages
- Erasure confirmation aggregation
- Consensus achievement detection
- Automatic delete block pruning
- Testing with 3-node cluster

## Usage Example

```rust
// Admin deletes spam release
let delete_block = DeleteBlock::new(
    vec!["release-spam123".to_string()],
    DeleteReason::Spam,
    "admin-pubkey".to_string(),
    3, // Require 3 confirmations
);

// Broadcast to network
network.broadcast_block(encode_delete_block(&delete_block)).await?;

// Peers receive and confirm
let confirmation = ErasureConfirmation::new(
    delete_block.id.clone(),
    peer_id,
    vec!["release-spam123".to_string()],
);

network.send_erasure_confirmation(confirmation).await?;

// Once 3 confirmations received:
// - delete_block.consensus_achieved = true
// - Delete delete_block from database
// - Spam is gone from entire network!
```

## Benefits

1. **Distributed Consensus**: No single point of failure
2. **Proof of Erasure**: Cryptographic confirmation of deletion
3. **Self-Cleaning**: Delete blocks auto-prune after consensus
4. **Audit Trail**: Full history of what was deleted and why
5. **Admin Control**: Only authorized admins can delete
6. **Spam Defense**: Quick response to abuse

## Next Steps

1. Complete network broadcast implementations
2. Add relay routing for delete messages
3. Implement sync_orchestrator handlers
4. Add database persistence for DeleteBlocks
5. Test full flow with docker cluster
6. Add monitoring/metrics for deletion consensus
