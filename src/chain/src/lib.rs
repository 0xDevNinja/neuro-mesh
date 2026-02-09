//! NeuroChain runtime
//!
//! This crate defines the Substrate runtime for the NeuroMesh protocol.
//! It currently provides a minimal skeleton with placeholders for
//! pallets and extrinsics.  See the `pallets` module for details.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod pallets;

// Re-export useful Substrate primitives.  These will be extended as
// additional pallets and runtime APIs are implemented.
pub use sp_std::prelude::*;

/// The runtime version.  Bump this when making breaking changes.
pub const VERSION: u32 = 1;

// TODO: Construct the runtime using FRAME and include pallets such as
// balances, staking, subnets, miner registry, validator registry,
// emissions, and consensus logic.  See the backlog for tasks.