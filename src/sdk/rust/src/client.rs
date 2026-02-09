//! Client for interacting with NeuroChain nodes.

use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use sp_core::sr25519;

/// A simple wrapper around a JSON‑RPC client that connects to a
/// NeuroChain node and exposes common API methods.
pub struct NeurochainClient {
    client: HttpClient,
    signer: Option<sr25519::Pair>,
}

impl NeurochainClient {
    /// Create a new client for the given node URL.
    pub fn new(url: &str) -> Self {
        let client = HttpClientBuilder::default()
            .build(url)
            .expect("Failed to create HTTP client");
        Self { client, signer: None }
    }

    /// Attach a signer (keypair) for sending signed extrinsics.
    pub fn with_signer(mut self, pair: sr25519::Pair) -> Self {
        self.signer = Some(pair);
        self
    }

    /// Fetch the current block number.
    pub async fn block_number(&self) -> Result<u64, Box<dyn std::error::Error>> {
        let result: serde_json::Value = self
            .client
            .request("chain_getHeader", None)
            .await?;
        let block_number_hex = result["number"]
            .as_str()
            .ok_or("Invalid response")?;
        let block_number = u64::from_str_radix(block_number_hex.trim_start_matches("0x"), 16)?;
        Ok(block_number)
    }
}