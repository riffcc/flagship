use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use lens_v2_p2p::P2pManager;
use std::sync::Arc;

/// Application state containing the P2P manager
#[derive(Clone)]
pub struct SyncState {
    pub p2p: Arc<P2pManager>,
}

/// Handler for /ready endpoint
///
/// Returns sync status indicating:
/// - Whether the node is fully synced
/// - How many blocks behind the network consensus
/// - Number of connected peers
pub async fn ready_handler(State(state): State<SyncState>) -> impl IntoResponse {
    match state.p2p.sync_status() {
        Ok(status) => {
            // Return HTTP 200 if synced, 503 Service Unavailable if behind
            let status_code = if status.is_synced {
                StatusCode::OK
            } else {
                StatusCode::SERVICE_UNAVAILABLE
            };

            (status_code, Json(status)).into_response()
        }
        Err(e) => {
            let error = serde_json::json!({
                "error": format!("Failed to get sync status: {}", e)
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lens_v2_p2p::{BlockMeta, P2pConfig};

    #[tokio::test]
    async fn test_ready_not_synced() {
        let manager = Arc::new(P2pManager::new(P2pConfig::default()));
        let state = SyncState { p2p: manager };

        let response = ready_handler(State(state)).await.into_response();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_ready_synced() {
        let manager = Arc::new(P2pManager::new(P2pConfig::default()));

        // Add peer (defaults to Server type)
        manager.add_peer(1, None).unwrap();

        // Add blocks (synced at height 0 with 1 peer)
        let state = SyncState {
            p2p: manager.clone(),
        };

        let response = ready_handler(State(state)).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_ready_behind() {
        let manager = Arc::new(P2pManager::new(P2pConfig::default()));

        // Add peer (defaults to Server type)
        manager.add_peer(1, None).unwrap();

        // Add consensus blocks we don't have locally
        manager
            .update_consensus(vec![
                BlockMeta {
                    id: "block1".to_string(),
                    height: 1,
                    prev: None,
                    timestamp: 0,
                },
                BlockMeta {
                    id: "block2".to_string(),
                    height: 2,
                    prev: Some("block1".to_string()),
                    timestamp: 0,
                },
            ])
            .unwrap();

        let state = SyncState {
            p2p: manager.clone(),
        };

        let response = ready_handler(State(state)).await.into_response();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}
