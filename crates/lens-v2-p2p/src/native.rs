//! Native P2P implementation (non-WASM)
//!
//! Uses tokio for async runtime (optional via feature flag)

/// Native P2P transport layer
#[cfg(not(target_arch = "wasm32"))]
pub struct NativeTransport {
    // Future: Add tokio-based transport here
}

#[cfg(not(target_arch = "wasm32"))]
impl NativeTransport {
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for NativeTransport {
    fn default() -> Self {
        Self::new()
    }
}

// Placeholder for WASM builds
#[cfg(target_arch = "wasm32")]
pub struct NativeTransport;

#[cfg(target_arch = "wasm32")]
impl NativeTransport {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(target_arch = "wasm32")]
impl Default for NativeTransport {
    fn default() -> Self {
        Self::new()
    }
}
