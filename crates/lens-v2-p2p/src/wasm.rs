//! WASM-specific P2P implementation using WebSockets

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use web_sys::{MessageEvent, WebSocket};

/// WASM P2P transport layer
#[cfg(target_arch = "wasm32")]
pub struct WasmTransport {
    socket: Option<WebSocket>,
}

#[cfg(target_arch = "wasm32")]
impl WasmTransport {
    pub fn new() -> Self {
        Self { socket: None }
    }

    pub fn connect(&mut self, url: &str) -> Result<(), JsValue> {
        let socket = WebSocket::new(url)?;
        socket.set_binary_type(web_sys::BinaryType::Arraybuffer);
        self.socket = Some(socket);
        Ok(())
    }

    pub fn send(&self, data: &[u8]) -> Result<(), JsValue> {
        if let Some(socket) = &self.socket {
            socket.send_with_u8_array(data)?;
        }
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
impl Default for WasmTransport {
    fn default() -> Self {
        Self::new()
    }
}

// Placeholder for non-WASM builds
#[cfg(not(target_arch = "wasm32"))]
pub struct WasmTransport;

#[cfg(not(target_arch = "wasm32"))]
impl WasmTransport {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for WasmTransport {
    fn default() -> Self {
        Self::new()
    }
}
