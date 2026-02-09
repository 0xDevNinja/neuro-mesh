#[test]
fn test_block_number_mock() {
    // This test simply creates a client instance.  We don't connect to
    // a real node in this placeholder.  In future, use mock RPC or
    // integration tests.
    let client = neurochain_sdk::NeurochainClient::new("http://localhost:9933");
    // Ensure the client is created without panicking.
    assert!(client.block_number().is_err());
}