//! NeuroChain Runtime Configuration
//!
//! This module configures the Substrate runtime for NeuroChain,
//! the application-specific blockchain powering NeuroMesh.

use frame_support::weights::constants::WEIGHT_REF_TIME_PER_SECOND;

/// Block time in milliseconds
pub const MILLISECS_PER_BLOCK: u64 = 6000;

/// Slot duration
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

/// Block number type
pub type BlockNumber = u32;

/// Account balance type
pub type Balance = u128;

/// Runtime version
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("neurochain"),
    impl_name: create_runtime_str!("neurochain-node"),
    authoring_version: 1,
    spec_version: 100,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
    state_version: 1,
};
