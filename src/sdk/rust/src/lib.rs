//! NeuroChain Rust SDK
//!
//! This crate provides a lightweight client for interacting with a
//! NeuroChain node.  It will eventually wrap the JSON‑RPC and
//! substrate API, providing functions to register miners/validators,
//! submit weights, and query chain state.  The current version
//! includes only minimal scaffolding.

pub mod client;

pub use client::NeurochainClient;