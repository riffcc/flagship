//! Cluster Configuration
//!
//! Manages cluster membership for lens-v2-node clusters.
//! All nodes in a cluster share the same content and sync via TGP.

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[cfg(feature = "dht")]
use citadel_core::topology::{MeshConfig, SlotCoordinate};

/// Cluster configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    /// Unique cluster ID (shared by all nodes in cluster)
    pub cluster_id: String,

    /// This node's ID within the cluster (0-40 for 41 nodes)
    pub node_id: u32,

    /// Total nodes in cluster
    pub cluster_size: u32,

    /// Relay WebSocket URL for peer discovery
    pub relay_url: String,

    /// UDP listen address for TGP
    pub udp_listen: SocketAddr,

    /// Bootstrap peers (initial nodes to connect to)
    pub bootstrap_peers: Vec<PeerAddress>,

    /// DHT mesh configuration (optional, enables DHT routing)
    ///
    /// When set, enables Citadel DHT routing with hexagonal mesh topology.
    /// Nodes are assigned slots in the mesh and use greedy routing for
    /// efficient block discovery.
    #[cfg(feature = "dht")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mesh_config: Option<MeshConfig>,

    /// This node's slot coordinate in the DHT mesh (optional)
    ///
    /// Assigned automatically based on node_id when mesh_config is set.
    /// Used for greedy routing to locate blocks efficiently.
    #[cfg(feature = "dht")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot_coordinate: Option<SlotCoordinate>,
}

/// Peer address information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerAddress {
    pub node_id: u32,
    pub udp_addr: SocketAddr,
    pub http_addr: String,
}

impl ClusterConfig {
    /// Create default cluster config for a 41-node cluster
    pub fn default_41_node_cluster(node_id: u32) -> Self {
        #[cfg(feature = "dht")]
        let (mesh_config, slot_coordinate) = {
            // Default to 10x10x5 mesh (500 slots) for development
            let mesh = MeshConfig::new(10, 10, 5);
            let slot = Self::node_id_to_slot(node_id, &mesh);
            (Some(mesh), Some(slot))
        };

        Self {
            cluster_id: "lens-cluster-default".to_string(),
            node_id,
            cluster_size: 41,
            relay_url: "ws://localhost:5002/api/v1/relay/ws".to_string(),
            udp_listen: format!("0.0.0.0:{}", 6000 + node_id).parse().unwrap(),
            bootstrap_peers: Vec::new(),
            #[cfg(feature = "dht")]
            mesh_config,
            #[cfg(feature = "dht")]
            slot_coordinate,
        }
    }

    /// Convert node ID to slot coordinate using deterministic mapping
    ///
    /// Maps node IDs to mesh slots in a predictable way that distributes
    /// nodes evenly across the mesh topology.
    #[cfg(feature = "dht")]
    pub fn node_id_to_slot(node_id: u32, mesh_config: &MeshConfig) -> SlotCoordinate {
        let total_slots = mesh_config.total_slots() as u32;
        let slot_index = node_id % total_slots;

        // Convert flat index to 3D coordinates
        let z = slot_index / (mesh_config.width as u32 * mesh_config.height as u32);
        let remainder = slot_index % (mesh_config.width as u32 * mesh_config.height as u32);
        let y = remainder / mesh_config.width as u32;
        let x = remainder % mesh_config.width as u32;

        SlotCoordinate::new(x as i32, y as i32, z as i32)
    }

    /// Load from environment variables
    pub fn from_env() -> anyhow::Result<Self> {
        let cluster_id = std::env::var("LENS_CLUSTER_ID")
            .unwrap_or_else(|_| "lens-cluster-default".to_string());

        let node_id = std::env::var("LENS_NODE_ID")
            .unwrap_or_else(|_| "0".to_string())
            .parse()?;

        let cluster_size = std::env::var("LENS_CLUSTER_SIZE")
            .unwrap_or_else(|_| "41".to_string())
            .parse()?;

        let relay_url = std::env::var("RELAY_URL")
            .unwrap_or_else(|_| "ws://localhost:5002/api/v1/relay/ws".to_string());

        let udp_port = 6000 + node_id;
        let udp_listen = format!("0.0.0.0:{}", udp_port).parse()?;

        // Parse bootstrap peers from comma-separated list
        let bootstrap_peers = std::env::var("LENS_BOOTSTRAP_PEERS")
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.is_empty())
            .filter_map(|peer_str| {
                // Format: node_id:udp_addr:http_addr
                let parts: Vec<&str> = peer_str.split(':').collect();
                if parts.len() >= 3 {
                    Some(PeerAddress {
                        node_id: parts[0].parse().ok()?,
                        udp_addr: format!("{}:{}", parts[1], parts[2]).parse().ok()?,
                        http_addr: format!("http://{}:{}", parts[1], parts.get(3).unwrap_or(&"5002")),
                    })
                } else {
                    None
                }
            })
            .collect();

        // DHT mesh configuration from environment
        #[cfg(feature = "dht")]
        let (mesh_config, slot_coordinate) = {
            // Check if DHT is enabled via env var
            let dht_enabled = std::env::var("LENS_DHT_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse::<bool>()
                .unwrap_or(true);

            if dht_enabled {
                // Parse mesh dimensions (default: 10x10x5 = 500 slots)
                let width: usize = std::env::var("LENS_DHT_WIDTH")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()?;
                let height: usize = std::env::var("LENS_DHT_HEIGHT")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()?;
                let depth: usize = std::env::var("LENS_DHT_DEPTH")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()?;

                let mesh = MeshConfig::new(width, height, depth);
                let slot = Self::node_id_to_slot(node_id, &mesh);
                (Some(mesh), Some(slot))
            } else {
                (None, None)
            }
        };

        Ok(Self {
            cluster_id,
            node_id,
            cluster_size,
            relay_url,
            udp_listen,
            bootstrap_peers,
            #[cfg(feature = "dht")]
            mesh_config,
            #[cfg(feature = "dht")]
            slot_coordinate,
        })
    }

    /// Generate bootstrap peers for a local cluster (all on localhost)
    pub fn generate_local_bootstrap_peers(&mut self, exclude_self: bool) {
        self.bootstrap_peers.clear();

        for i in 0..self.cluster_size {
            if exclude_self && i == self.node_id {
                continue;
            }

            self.bootstrap_peers.push(PeerAddress {
                node_id: i,
                udp_addr: format!("127.0.0.1:{}", 6000 + i).parse().unwrap(),
                http_addr: format!("http://127.0.0.1:{}", 5002 + i),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_cluster_config() {
        let config = ClusterConfig::default_41_node_cluster(5);
        assert_eq!(config.node_id, 5);
        assert_eq!(config.cluster_size, 41);
        assert_eq!(config.udp_listen.port(), 6005);
    }

    #[test]
    fn test_generate_local_bootstrap_peers() {
        let mut config = ClusterConfig::default_41_node_cluster(0);
        config.cluster_size = 5; // Smaller for testing
        config.generate_local_bootstrap_peers(true);

        assert_eq!(config.bootstrap_peers.len(), 4); // Excludes self
        assert_eq!(config.bootstrap_peers[0].node_id, 1);
        assert_eq!(config.bootstrap_peers[0].udp_addr.port(), 6001);
    }
}
