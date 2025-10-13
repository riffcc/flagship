use axum::{
    extract::State,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::storage::dht_storage::DHTMetrics;

/// DHT state for health check endpoint
#[derive(Clone)]
pub struct DHTState {
    pub metrics: Arc<std::sync::Mutex<DHTMetrics>>,
}

impl DHTState {
    pub fn new(metrics: Arc<std::sync::Mutex<DHTMetrics>>) -> Self {
        Self { metrics }
    }

    pub fn get_metrics(&self) -> DHTMetrics {
        self.metrics.lock().unwrap().clone()
    }
}

/// DHT health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct DHTHealthResponse {
    pub status: String,
    pub metrics: DHTHealthMetrics,
}

/// DHT health check metrics
#[derive(Debug, Serialize, Deserialize)]
pub struct DHTHealthMetrics {
    pub get_count: u64,
    pub put_count: u64,
    pub delete_count: u64,
    pub total_operations: u64,
    pub avg_get_latency_ms: f64,
    pub avg_put_latency_ms: f64,
    pub avg_delete_latency_ms: f64,
    pub error_count: u64,
    pub error_rate: f64,
}

impl From<DHTMetrics> for DHTHealthMetrics {
    fn from(metrics: DHTMetrics) -> Self {
        let total_ops = metrics.get_count + metrics.put_count + metrics.delete_count;
        let error_rate = if total_ops > 0 {
            (metrics.errors as f64) / (total_ops as f64)
        } else {
            0.0
        };

        let avg_get_latency = if metrics.get_count > 0 {
            (metrics.total_get_latency_ms as f64) / (metrics.get_count as f64)
        } else {
            0.0
        };

        let avg_put_latency = if metrics.put_count > 0 {
            (metrics.total_put_latency_ms as f64) / (metrics.put_count as f64)
        } else {
            0.0
        };

        let avg_delete_latency = if metrics.delete_count > 0 {
            (metrics.total_delete_latency_ms as f64) / (metrics.delete_count as f64)
        } else {
            0.0
        };

        Self {
            get_count: metrics.get_count,
            put_count: metrics.put_count,
            delete_count: metrics.delete_count,
            total_operations: total_ops,
            avg_get_latency_ms: avg_get_latency,
            avg_put_latency_ms: avg_put_latency,
            avg_delete_latency_ms: avg_delete_latency,
            error_count: metrics.errors,
            error_rate,
        }
    }
}

/// GET /api/v1/dht/health - Get DHT health status and metrics
pub async fn dht_health_check(
    State(state): State<DHTState>,
) -> Json<DHTHealthResponse> {
    let metrics = state.get_metrics();
    let health_metrics = DHTHealthMetrics::from(metrics);

    // Determine status based on error rate
    let status = if health_metrics.error_rate > 0.1 {
        "degraded".to_string()
    } else if health_metrics.error_rate > 0.0 {
        "warning".to_string()
    } else {
        "healthy".to_string()
    };

    Json(DHTHealthResponse {
        status,
        metrics: health_metrics,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{Request, StatusCode};
    use axum::Router;
    use axum::routing::get;
    use tower::ServiceExt; // for `oneshot`

    #[test]
    fn test_dht_health_metrics_from_empty() {
        let metrics = DHTMetrics::default();
        let health_metrics = DHTHealthMetrics::from(metrics);

        assert_eq!(health_metrics.get_count, 0);
        assert_eq!(health_metrics.put_count, 0);
        assert_eq!(health_metrics.delete_count, 0);
        assert_eq!(health_metrics.total_operations, 0);
        assert_eq!(health_metrics.error_count, 0);
        assert_eq!(health_metrics.error_rate, 0.0);
        assert_eq!(health_metrics.avg_get_latency_ms, 0.0);
        assert_eq!(health_metrics.avg_put_latency_ms, 0.0);
        assert_eq!(health_metrics.avg_delete_latency_ms, 0.0);
    }

    #[test]
    fn test_dht_health_metrics_with_operations() {
        let metrics = DHTMetrics {
            get_count: 100,
            put_count: 50,
            delete_count: 25,
            total_get_latency_ms: 1000,
            total_put_latency_ms: 2000,
            total_delete_latency_ms: 500,
            errors: 5,
        };

        let health_metrics = DHTHealthMetrics::from(metrics);

        assert_eq!(health_metrics.get_count, 100);
        assert_eq!(health_metrics.put_count, 50);
        assert_eq!(health_metrics.delete_count, 25);
        assert_eq!(health_metrics.total_operations, 175);
        assert_eq!(health_metrics.error_count, 5);
        assert!((health_metrics.error_rate - 0.02857).abs() < 0.0001);
        assert!((health_metrics.avg_get_latency_ms - 10.0).abs() < 0.01);
        assert!((health_metrics.avg_put_latency_ms - 40.0).abs() < 0.01);
        assert!((health_metrics.avg_delete_latency_ms - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_dht_health_metrics_error_rate() {
        // Test high error rate
        let metrics = DHTMetrics {
            get_count: 10,
            put_count: 0,
            delete_count: 0,
            total_get_latency_ms: 100,
            total_put_latency_ms: 0,
            total_delete_latency_ms: 0,
            errors: 5,
        };

        let health_metrics = DHTHealthMetrics::from(metrics);
        assert_eq!(health_metrics.error_rate, 0.5);
    }

    #[tokio::test]
    async fn test_dht_health_check_endpoint_healthy() {
        // Create DHT state with healthy metrics
        let metrics = Arc::new(std::sync::Mutex::new(DHTMetrics {
            get_count: 100,
            put_count: 50,
            delete_count: 10,
            total_get_latency_ms: 500,
            total_put_latency_ms: 1000,
            total_delete_latency_ms: 100,
            errors: 0,
        }));

        let state = DHTState::new(metrics);

        // Create router with endpoint
        let app = Router::new()
            .route("/api/v1/dht/health", get(dht_health_check))
            .with_state(state);

        // Make request
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/dht/health")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Parse response
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let health: DHTHealthResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(health.status, "healthy");
        assert_eq!(health.metrics.get_count, 100);
        assert_eq!(health.metrics.put_count, 50);
        assert_eq!(health.metrics.delete_count, 10);
        assert_eq!(health.metrics.total_operations, 160);
        assert_eq!(health.metrics.error_count, 0);
        assert_eq!(health.metrics.error_rate, 0.0);
    }

    #[tokio::test]
    async fn test_dht_health_check_endpoint_degraded() {
        // Create DHT state with high error rate (>10%)
        let metrics = Arc::new(std::sync::Mutex::new(DHTMetrics {
            get_count: 100,
            put_count: 0,
            delete_count: 0,
            total_get_latency_ms: 1000,
            total_put_latency_ms: 0,
            total_delete_latency_ms: 0,
            errors: 15, // 15% error rate
        }));

        let state = DHTState::new(metrics);

        // Create router with endpoint
        let app = Router::new()
            .route("/api/v1/dht/health", get(dht_health_check))
            .with_state(state);

        // Make request
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/dht/health")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Parse response
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let health: DHTHealthResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(health.status, "degraded");
        assert!(health.metrics.error_rate > 0.1);
    }

    #[tokio::test]
    async fn test_dht_health_check_endpoint_warning() {
        // Create DHT state with low error rate (between 0 and 10%)
        let metrics = Arc::new(std::sync::Mutex::new(DHTMetrics {
            get_count: 100,
            put_count: 0,
            delete_count: 0,
            total_get_latency_ms: 1000,
            total_put_latency_ms: 0,
            total_delete_latency_ms: 0,
            errors: 5, // 5% error rate
        }));

        let state = DHTState::new(metrics);

        // Create router with endpoint
        let app = Router::new()
            .route("/api/v1/dht/health", get(dht_health_check))
            .with_state(state);

        // Make request
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/dht/health")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Parse response
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let health: DHTHealthResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(health.status, "warning");
        assert!(health.metrics.error_rate > 0.0 && health.metrics.error_rate <= 0.1);
    }
}
