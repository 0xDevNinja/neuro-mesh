//! NeuroMesh networking primitives.
//!
//! Defines the protocol identifier, gossip topic scheme, and a minimal
//! in-memory peer directory. The concrete libp2p Swarm wiring (Kademlia,
//! Gossipsub, Identify, NAT traversal) lives in downstream crates that
//! depend on this one — pinning those versions up-front would couple every
//! downstream crate to a specific libp2p release. Instead this crate stays
//! pure-Rust and easy to test.

use std::collections::HashMap;

pub const PROTOCOL_ID: &str = "/neuromesh/1.0.0";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Role {
    Miner,
    Validator,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Role::Miner => "miners",
            Role::Validator => "validators",
        }
    }
}

/// Topic naming scheme shared by all NeuroMesh nodes.
///
/// Examples:
/// - `/neuromesh/1.0.0/subnet/1/miners`
/// - `/neuromesh/1.0.0/subnet/1/validators`
/// - `/neuromesh/1.0.0/global/announcements`
pub fn subnet_topic(subnet_id: u32, role: Role) -> String {
    format!("{PROTOCOL_ID}/subnet/{subnet_id}/{}", role.as_str())
}

pub fn global_announcements_topic() -> String {
    format!("{PROTOCOL_ID}/global/announcements")
}

/// Peer metadata as gossipped over the network.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PeerInfo {
    pub peer_id: String,
    pub subnet_id: u32,
    pub uid: u32,
    pub role: Role,
    pub endpoint: String,
    /// Unix seconds of the last status update. Older entries are culled.
    pub last_seen: u64,
}

/// In-memory peer directory. Nodes feed `PeerInfo` updates in; consumers query
/// by subnet/role. This is the shape a libp2p `NetworkBehaviour` will expose
/// after a Gossipsub message is decoded.
#[derive(Default)]
pub struct PeerDirectory {
    peers: HashMap<String, PeerInfo>,
    ttl_secs: u64,
}

impl PeerDirectory {
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            peers: HashMap::new(),
            ttl_secs: ttl_secs.max(1),
        }
    }

    pub fn upsert(&mut self, info: PeerInfo) {
        self.peers.insert(info.peer_id.clone(), info);
    }

    pub fn remove(&mut self, peer_id: &str) -> Option<PeerInfo> {
        self.peers.remove(peer_id)
    }

    /// Remove entries older than `ttl_secs` relative to `now`. Returns how
    /// many were culled.
    pub fn prune(&mut self, now: u64) -> usize {
        let before = self.peers.len();
        self.peers
            .retain(|_, p| now.saturating_sub(p.last_seen) <= self.ttl_secs);
        before - self.peers.len()
    }

    pub fn peers_in(&self, subnet_id: u32, role: Role) -> Vec<&PeerInfo> {
        self.peers
            .values()
            .filter(|p| p.subnet_id == subnet_id && p.role == role)
            .collect()
    }

    pub fn len(&self) -> usize {
        self.peers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.peers.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn peer(peer_id: &str, subnet: u32, uid: u32, role: Role, last_seen: u64) -> PeerInfo {
        PeerInfo {
            peer_id: peer_id.into(),
            subnet_id: subnet,
            uid,
            role,
            endpoint: format!("http://{peer_id}"),
            last_seen,
        }
    }

    #[test]
    fn topic_scheme_matches_spec() {
        assert_eq!(
            subnet_topic(1, Role::Miner),
            "/neuromesh/1.0.0/subnet/1/miners"
        );
        assert_eq!(
            subnet_topic(42, Role::Validator),
            "/neuromesh/1.0.0/subnet/42/validators"
        );
        assert_eq!(
            global_announcements_topic(),
            "/neuromesh/1.0.0/global/announcements"
        );
    }

    #[test]
    fn directory_upsert_and_query() {
        let mut dir = PeerDirectory::new(60);
        dir.upsert(peer("a", 1, 0, Role::Miner, 10));
        dir.upsert(peer("b", 1, 1, Role::Miner, 10));
        dir.upsert(peer("c", 1, 0, Role::Validator, 10));
        assert_eq!(dir.peers_in(1, Role::Miner).len(), 2);
        assert_eq!(dir.peers_in(1, Role::Validator).len(), 1);
        assert_eq!(dir.peers_in(2, Role::Miner).len(), 0);
    }

    #[test]
    fn directory_overwrites_same_peer_id() {
        let mut dir = PeerDirectory::new(60);
        dir.upsert(peer("a", 1, 0, Role::Miner, 10));
        dir.upsert(peer("a", 1, 0, Role::Miner, 20));
        assert_eq!(dir.len(), 1);
        assert_eq!(dir.peers_in(1, Role::Miner)[0].last_seen, 20);
    }

    #[test]
    fn directory_remove_works() {
        let mut dir = PeerDirectory::new(60);
        dir.upsert(peer("a", 1, 0, Role::Miner, 10));
        assert!(dir.remove("a").is_some());
        assert!(dir.is_empty());
    }

    #[test]
    fn prune_drops_stale_entries() {
        let mut dir = PeerDirectory::new(30);
        dir.upsert(peer("fresh", 1, 0, Role::Miner, 140));
        dir.upsert(peer("stale", 1, 1, Role::Miner, 10));
        let culled = dir.prune(150);
        assert_eq!(culled, 1);
        assert_eq!(dir.len(), 1);
        assert_eq!(dir.peers_in(1, Role::Miner)[0].peer_id, "fresh");
    }

    #[test]
    fn prune_keeps_entries_within_ttl() {
        let mut dir = PeerDirectory::new(30);
        dir.upsert(peer("a", 1, 0, Role::Miner, 120));
        let culled = dir.prune(140);
        assert_eq!(culled, 0);
        assert_eq!(dir.len(), 1);
    }

    #[test]
    fn role_as_str_stable() {
        assert_eq!(Role::Miner.as_str(), "miners");
        assert_eq!(Role::Validator.as_str(), "validators");
    }
}
