//! NeuroMesh Node Library
//!
//! This crate implements the networking layer for miners and validators
//! using libp2p and gRPC.  It provides functions for peer discovery,
//! pub/sub topics, and service definitions.  At the moment, it
//! contains placeholder code to illustrate the structure.

use async_std::task;
use libp2p::{identity, mdns, swarm::{NetworkBehaviour, Swarm}, PeerId};

/// Start a simple libp2p node that announces itself on the mDNS
/// network.  This function is for demonstration purposes only and
/// will be replaced by a full implementation.
pub fn start_mdns_node() {
    // Generate a random peer ID.
    let id_keys = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(id_keys.public());
    println!("Local node id: {}", peer_id);

    #[derive(NetworkBehaviour)]
    struct MyBehaviour {
        mdns: mdns::async_io::Behaviour,
    }

    let behaviour = MyBehaviour {
        mdns: mdns::async_io::Behaviour::new(mdns::Config::default(), peer_id)
            .expect("can create mdns behaviour"),
    };

    let mut swarm = Swarm::with_async_std_executor(
        libp2p::SwarmBuilder::new(id_keys, behaviour, peer_id)
            .build(),
    );

    task::block_on(async move {
        Swarm::listen_on(&mut swarm, "/ip4/0.0.0.0/tcp/0".parse().unwrap())
            .expect("can start listening");
        loop {
            match swarm.next_event().await {
                _ => {}
            }
        }
    });
}