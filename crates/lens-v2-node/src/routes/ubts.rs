use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn};

use crate::db::Database;
use crate::ubts::{UBTSBlock, UBTSTransaction};
use crate::delete_block::DeleteReason;
use crate::ubts_codec::UBTSCodec;
use crate::routes::account::AccountState;
use lens_v2_p2p::{P2pManager, P2pNetwork};
use lens_v2_p2p::network::BlockData;

/// State for UBTS transaction endpoints
#[derive(Clone)]
pub struct UBTSState {
    pub db: Database,
    pub account_state: AccountState,
    pub p2p_manager: Arc<P2pManager>,
    pub network: Arc<P2pNetwork>,
}

impl UBTSState {
    pub fn new(db: Database, account_state: AccountState, p2p_manager: Arc<P2pManager>, network: Arc<P2pNetwork>) -> Self {
        Self {
            db,
            account_state,
            p2p_manager,
            network,
        }
    }
}

/// Request to create a UBTS block with transactions
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateUBTSBlockRequest {
    pub transactions: Vec<UBTSTransactionRequest>,
    pub signature: Option<String>,
}

/// Transaction request format (simplified for API)
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UBTSTransactionRequest {
    CreateRelease {
        release: serde_json::Value,
        signature: Option<String>,
    },
    UpdateRelease {
        id: String,
        patch: serde_json::Value,
        signature: Option<String>,
    },
    DeleteRelease {
        id: String,
        signature: Option<String>,
    },
    DeleteWithConsensus {
        delete_id: String,
        deleted_block_ids: Vec<String>,
        reason: DeleteReason,
        deleted_by: String,
        required_confirmations: usize,
        signature: Option<String>,
    },
    CreateTombstone {
        tombstone_hash: String,
        reason: DeleteReason,
    },
    RemoveTombstone {
        tombstone_hash: String,
        removed_by: String,
        signature: Option<String>,
    },
    AuthorizeAdmin {
        public_key: String,
        authorized_by: String,
        signature: Option<String>,
    },
    RevokeAdmin {
        public_key: String,
        revoked_by: String,
        signature: Option<String>,
    },
    SetFeatured {
        featured_releases: Vec<serde_json::Value>,
        signature: Option<String>,
    },
    AddFeatured {
        release_ids: Vec<String>,
        signature: Option<String>,
    },
    RemoveFeatured {
        release_ids: Vec<String>,
        signature: Option<String>,
    },
    DeleteFeaturedRelease {
        id: String,
        signature: Option<String>,
    },
}

/// Response for UBTS block creation
#[derive(Debug, Serialize)]
pub struct CreateUBTSBlockResponse {
    pub block_id: String,
    pub height: u64,
    pub transaction_count: usize,
    pub timestamp: u64,
}

/// Create a new UBTS block with transactions and broadcast to network
pub async fn create_ubts_block(
    State(state): State<UBTSState>,
    Json(req): Json<CreateUBTSBlockRequest>,
) -> Result<Json<CreateUBTSBlockResponse>, (StatusCode, String)> {
    info!("🔷 Creating UBTS block with {} transactions", req.transactions.len());

    // Convert API transaction format to internal format
    let mut transactions = Vec::new();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    for tx_req in req.transactions {
        let tx = match tx_req {
            UBTSTransactionRequest::CreateRelease { release, signature } => {
                // Parse release JSON into Release struct
                let release_obj: crate::routes::releases::Release = serde_json::from_value(release)
                    .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid release format: {}", e)))?;

                UBTSTransaction::CreateRelease {
                    release: release_obj,
                    signature,
                }
            }
            UBTSTransactionRequest::UpdateRelease { id, patch, signature } => {
                // Parse patch JSON
                let patch_obj: crate::ubts::ReleasePatch = serde_json::from_value(patch)
                    .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid patch format: {}", e)))?;

                UBTSTransaction::UpdateRelease {
                    id,
                    patch: patch_obj,
                    signature,
                }
            }
            UBTSTransactionRequest::DeleteRelease { id, signature } => {
                UBTSTransaction::DeleteRelease { id, signature }
            }
            UBTSTransactionRequest::DeleteWithConsensus {
                delete_id,
                deleted_block_ids,
                reason,
                deleted_by,
                required_confirmations,
                signature,
            } => {
                UBTSTransaction::DeleteWithConsensus {
                    delete_id,
                    deleted_block_ids,
                    reason,
                    deleted_by,
                    required_confirmations,
                    timestamp,
                    signature,
                }
            }
            UBTSTransactionRequest::CreateTombstone { tombstone_hash, reason } => {
                UBTSTransaction::CreateTombstone {
                    tombstone_hash,
                    reason,
                    timestamp,
                }
            }
            UBTSTransactionRequest::RemoveTombstone { tombstone_hash, removed_by, signature } => {
                UBTSTransaction::RemoveTombstone {
                    tombstone_hash,
                    removed_by,
                    timestamp,
                    signature,
                }
            }
            UBTSTransactionRequest::AuthorizeAdmin { public_key, authorized_by, signature } => {
                UBTSTransaction::AuthorizeAdmin {
                    public_key,
                    authorized_by,
                    timestamp,
                    signature,
                }
            }
            UBTSTransactionRequest::RevokeAdmin { public_key, revoked_by, signature } => {
                UBTSTransaction::RevokeAdmin {
                    public_key,
                    revoked_by,
                    timestamp,
                    signature,
                }
            }
            UBTSTransactionRequest::SetFeatured { featured_releases, signature } => {
                // Parse featured releases JSON into FeaturedRelease structs
                let mut featured_objs = Vec::new();
                for featured_val in featured_releases {
                    let featured: crate::routes::featured::FeaturedRelease = serde_json::from_value(featured_val)
                        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid featured release format: {}", e)))?;
                    featured_objs.push(featured);
                }

                UBTSTransaction::SetFeatured {
                    featured_releases: featured_objs,
                    signature,
                }
            }
            UBTSTransactionRequest::AddFeatured { release_ids, signature } => {
                UBTSTransaction::AddFeatured { release_ids, signature }
            }
            UBTSTransactionRequest::RemoveFeatured { release_ids, signature } => {
                UBTSTransaction::RemoveFeatured { release_ids, signature }
            }
            UBTSTransactionRequest::DeleteFeaturedRelease { id, signature } => {
                UBTSTransaction::DeleteFeaturedRelease { id, signature }
            }
        };
        transactions.push(tx);
    }

    // Get current block height from P2P manager
    let height = state.p2p_manager.sync_status()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get sync status: {}", e)))?
        .network_height as u64;

    // Get previous block ID (if any)
    let prev = if height > 0 {
        // TODO: Get actual previous block ID from database
        Some(format!("ubts-prev-{}", height - 1))
    } else {
        None
    };

    // Create UBTS block
    let block = UBTSBlock::new(height, prev.clone(), transactions);
    let block_id = block.id.clone();
    let transaction_count = block.transactions.len();
    let block_timestamp = block.timestamp;

    // Encode block to BlockData
    let block_data = UBTSCodec::encode(&block)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to encode block: {}", e)))?;

    // Broadcast to P2P network
    state.network.broadcast_block(block_data).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to broadcast block: {}", e)))?;
    info!("✅ Created and broadcast UBTS block: {} with {} transactions", block_id, transaction_count);

    Ok(Json(CreateUBTSBlockResponse {
        block_id,
        height,
        transaction_count,
        timestamp: block_timestamp,
    }))
}

/// Delete with consensus endpoint (simplified wrapper for DeleteWithConsensus transaction)
#[derive(Debug, Deserialize)]
pub struct DeleteWithConsensusRequest {
    pub block_ids: Vec<String>,
    pub reason: DeleteReason,
    pub required_confirmations: usize,
    pub public_key: String,
    pub signature: Option<String>,
}

pub async fn delete_with_consensus(
    State(state): State<UBTSState>,
    Json(req): Json<DeleteWithConsensusRequest>,
) -> Result<Json<CreateUBTSBlockResponse>, (StatusCode, String)> {
    info!("🗑️  Delete with consensus: {} blocks, reason: {:?}", req.block_ids.len(), req.reason);

    // Verify admin authorization
    if !state.account_state.is_admin(&req.public_key).await {
        return Err((StatusCode::FORBIDDEN, "Only admins can delete with consensus".to_string()));
    }

    // Create delete ID
    let delete_id = format!("delete-{}", uuid::Uuid::new_v4());
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Create DeleteWithConsensus transaction
    let tx = UBTSTransaction::DeleteWithConsensus {
        delete_id,
        deleted_block_ids: req.block_ids,
        reason: req.reason,
        deleted_by: req.public_key,
        required_confirmations: req.required_confirmations,
        timestamp,
        signature: req.signature,
    };

    // Create UBTS block with transaction
    let height = state.p2p_manager.sync_status()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get sync status: {}", e)))?
        .network_height as u64;
    let prev = if height > 0 {
        Some(format!("ubts-prev-{}", height - 1))
    } else {
        None
    };

    let block = UBTSBlock::new(height, prev.clone(), vec![tx]);
    let block_id = block.id.clone();
    let block_timestamp = block.timestamp;

    // Encode and broadcast
    // Encode block to BlockData
    let block_data = UBTSCodec::encode(&block)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to encode block: {}", e)))?;


    state.network.broadcast_block(block_data).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to broadcast block: {}", e)))?;
    info!("✅ Broadcast DeleteWithConsensus block: {}", block_id);

    Ok(Json(CreateUBTSBlockResponse {
        block_id,
        height,
        transaction_count: 1,
        timestamp: block_timestamp,
    }))
}

/// Set featured releases (replaces entire list)
#[derive(Debug, Deserialize)]
pub struct SetFeaturedRequest {
    pub release_ids: Vec<String>,
    pub public_key: String,
    pub signature: Option<String>,
}

pub async fn set_featured(
    State(state): State<UBTSState>,
    Json(req): Json<SetFeaturedRequest>,
) -> Result<Json<CreateUBTSBlockResponse>, (StatusCode, String)> {
    info!("⭐ Set featured releases: {} releases", req.release_ids.len());

    // Verify admin authorization
    if !state.account_state.is_admin(&req.public_key).await {
        return Err((StatusCode::FORBIDDEN, "Only admins can set featured releases".to_string()));
    }

    // Create featured release objects from IDs (for the dedicated endpoint)
    let now = chrono::Utc::now().to_rfc3339();
    let thirty_days_later = (chrono::Utc::now() + chrono::Duration::days(30)).to_rfc3339();

    let mut featured_releases = Vec::new();
    for (index, release_id) in req.release_ids.iter().enumerate() {
        let featured_id = uuid::Uuid::new_v4().to_string();
        let priority = ((req.release_ids.len() - index) * 10) as i32;

        let featured = crate::routes::featured::FeaturedRelease {
            id: featured_id,
            release_id: release_id.clone(),
            priority,
            promoted: index == 0,
            tags: vec![],
            start_time: Some(now.clone()),
            end_time: Some(thirty_days_later.clone()),
            custom_title: None,
            custom_description: None,
            custom_thumbnail: None,
            regions: None,
            languages: None,
            views: 0,
            clicks: 0,
            variant: None,
            metadata: None,
            created_at: now.clone(),
            updated_at: None,
        };
        featured_releases.push(featured);
    }

    // Create SetFeatured transaction
    let tx = UBTSTransaction::SetFeatured {
        featured_releases,
        signature: req.signature,
    };

    // Create and broadcast UBTS block
    let height = state.p2p_manager.sync_status()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get sync status: {}", e)))?
        .network_height as u64;
    let prev = if height > 0 {
        Some(format!("ubts-prev-{}", height - 1))
    } else {
        None
    };

    let block = UBTSBlock::new(height, prev.clone(), vec![tx]);
    let block_id = block.id.clone();
    let block_timestamp = block.timestamp;

    // Encode block to BlockData
    let block_data = UBTSCodec::encode(&block)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to encode block: {}", e)))?;


    state.network.broadcast_block(block_data).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to broadcast block: {}", e)))?;
    info!("✅ Broadcast SetFeatured block: {}", block_id);

    Ok(Json(CreateUBTSBlockResponse {
        block_id,
        height,
        transaction_count: 1,
        timestamp: block_timestamp,
    }))
}

/// Add releases to featured list
#[derive(Debug, Deserialize)]
pub struct AddFeaturedRequest {
    pub release_ids: Vec<String>,
    pub public_key: String,
    pub signature: Option<String>,
}

pub async fn add_featured(
    State(state): State<UBTSState>,
    Json(req): Json<AddFeaturedRequest>,
) -> Result<Json<CreateUBTSBlockResponse>, (StatusCode, String)> {
    info!("⭐ Add featured releases: {} releases", req.release_ids.len());

    // Verify admin authorization
    if !state.account_state.is_admin(&req.public_key).await {
        return Err((StatusCode::FORBIDDEN, "Only admins can add featured releases".to_string()));
    }

    // Create AddFeatured transaction
    let tx = UBTSTransaction::AddFeatured {
        release_ids: req.release_ids,
        signature: req.signature,
    };

    // Create and broadcast UBTS block
    let height = state.p2p_manager.sync_status()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get sync status: {}", e)))?
        .network_height as u64;
    let prev = if height > 0 {
        Some(format!("ubts-prev-{}", height - 1))
    } else {
        None
    };

    let block = UBTSBlock::new(height, prev.clone(), vec![tx]);
    let block_id = block.id.clone();
    let block_timestamp = block.timestamp;

    // Encode block to BlockData
    let block_data = UBTSCodec::encode(&block)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to encode block: {}", e)))?;


    state.network.broadcast_block(block_data).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to broadcast block: {}", e)))?;
    info!("✅ Broadcast AddFeatured block: {}", block_id);

    Ok(Json(CreateUBTSBlockResponse {
        block_id,
        height,
        transaction_count: 1,
        timestamp: block_timestamp,
    }))
}

/// Remove releases from featured list
#[derive(Debug, Deserialize)]
pub struct RemoveFeaturedRequest {
    pub release_ids: Vec<String>,
    pub public_key: String,
    pub signature: Option<String>,
}

pub async fn remove_featured(
    State(state): State<UBTSState>,
    Json(req): Json<RemoveFeaturedRequest>,
) -> Result<Json<CreateUBTSBlockResponse>, (StatusCode, String)> {
    info!("⭐ Remove featured releases: {} releases", req.release_ids.len());

    // Verify admin authorization
    if !state.account_state.is_admin(&req.public_key).await {
        return Err((StatusCode::FORBIDDEN, "Only admins can remove featured releases".to_string()));
    }

    // Create RemoveFeatured transaction
    let tx = UBTSTransaction::RemoveFeatured {
        release_ids: req.release_ids,
        signature: req.signature,
    };

    // Create and broadcast UBTS block
    let height = state.p2p_manager.sync_status()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get sync status: {}", e)))?
        .network_height as u64;
    let prev = if height > 0 {
        Some(format!("ubts-prev-{}", height - 1))
    } else {
        None
    };

    let block = UBTSBlock::new(height, prev.clone(), vec![tx]);
    let block_id = block.id.clone();
    let block_timestamp = block.timestamp;

    // Encode block to BlockData
    let block_data = UBTSCodec::encode(&block)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to encode block: {}", e)))?;


    state.network.broadcast_block(block_data).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to broadcast block: {}", e)))?;
    info!("✅ Broadcast RemoveFeatured block: {}", block_id);

    Ok(Json(CreateUBTSBlockResponse {
        block_id,
        height,
        transaction_count: 1,
        timestamp: block_timestamp,
    }))
}
