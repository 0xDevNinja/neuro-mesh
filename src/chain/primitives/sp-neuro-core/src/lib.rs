//! Core primitives for NeuroMesh runtimes.
//!
//! This crate defines foundational traits shared by pallets, runtime logic,
//! and off-chain workers. The interfaces are intentionally minimal to allow
//! downstream crates to compose richer functionality.

#![cfg_attr(not(feature = "std"), no_std)]

use parity_scale_codec::{Decode, Encode};
use sp_std::prelude::*;

/// Represents a neural task that can be scheduled and executed on the mesh.
///
/// Implementers should keep input and output types SCALE-encodable so they can
/// be stored on-chain or transmitted across XCMP channels.
pub trait NeuralTask {
    /// A stable identifier for the task, suitable for on-chain storage.
    type TaskId: Encode + Decode + Clone + PartialEq + Eq;
    /// The input payload for the task.
    type Input: Encode + Decode + Clone + PartialEq + Eq;
    /// The output payload produced by the task.
    type Output: Encode + Decode + Clone + PartialEq + Eq;

    /// Returns the stable identifier for this task.
    fn task_id(&self) -> Self::TaskId;

    /// Returns the task input payload.
    fn input(&self) -> &Self::Input;

    /// Helper to SCALE-encode an input payload for transport.
    fn encode_input(input: &Self::Input) -> Vec<u8> {
        input.encode()
    }

    /// Helper to SCALE-decode an input payload from transport bytes.
    fn decode_input(data: &[u8]) -> Result<Self::Input, parity_scale_codec::Error> {
        Self::Input::decode(&mut &data[..])
    }

    /// Helper to SCALE-encode an output payload for transport.
    fn encode_output(output: &Self::Output) -> Vec<u8> {
        output.encode()
    }

    /// Helper to SCALE-decode an output payload from transport bytes.
    fn decode_output(data: &[u8]) -> Result<Self::Output, parity_scale_codec::Error> {
        Self::Output::decode(&mut &data[..])
    }
}

/// Represents a mesh provider capable of executing neural tasks.
///
/// Providers typically map to registered nodes or services that stake to
/// participate in the protocol.
pub trait MeshProvider {
    /// A stable identifier for the provider.
    type ProviderId: Encode + Decode + Clone + PartialEq + Eq;
    /// Metadata describing capabilities, endpoints, or SLAs.
    type Metadata: Encode + Decode + Clone + PartialEq + Eq;

    /// Returns the provider identifier.
    fn provider_id(&self) -> Self::ProviderId;

    /// Returns the provider metadata payload.
    fn metadata(&self) -> &Self::Metadata;

    /// Helper to SCALE-encode metadata for transport.
    fn encode_metadata(metadata: &Self::Metadata) -> Vec<u8> {
        metadata.encode()
    }

    /// Helper to SCALE-decode metadata from transport bytes.
    fn decode_metadata(data: &[u8]) -> Result<Self::Metadata, parity_scale_codec::Error> {
        Self::Metadata::decode(&mut &data[..])
    }
}

#[cfg(test)]
mod tests {
    use super::{MeshProvider, NeuralTask};

    #[derive(Clone, PartialEq, Eq)]
    struct ExampleTask {
        task_id: u32,
        input: Vec<u8>,
    }

    impl NeuralTask for ExampleTask {
        type TaskId = u32;
        type Input = Vec<u8>;
        type Output = Vec<u8>;

        fn task_id(&self) -> Self::TaskId {
            self.task_id
        }

        fn input(&self) -> &Self::Input {
            &self.input
        }
    }

    #[derive(Clone, PartialEq, Eq)]
    struct ExampleProvider {
        provider_id: u64,
        metadata: Vec<u8>,
    }

    impl MeshProvider for ExampleProvider {
        type ProviderId = u64;
        type Metadata = Vec<u8>;

        fn provider_id(&self) -> Self::ProviderId {
            self.provider_id
        }

        fn metadata(&self) -> &Self::Metadata {
            &self.metadata
        }
    }

    #[test]
    fn neural_task_helpers_round_trip() {
        let input = vec![1u8, 2, 3, 4];
        let encoded = ExampleTask::encode_input(&input);
        let decoded = ExampleTask::decode_input(&encoded).expect("decode succeeds");
        assert_eq!(decoded, input);
    }

    #[test]
    fn mesh_provider_helpers_round_trip() {
        let metadata = vec![42u8, 7, 9];
        let encoded = ExampleProvider::encode_metadata(&metadata);
        let decoded = ExampleProvider::decode_metadata(&encoded).expect("decode succeeds");
        assert_eq!(decoded, metadata);
    }
}
