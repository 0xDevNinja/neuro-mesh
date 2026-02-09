"""Tests for the Python NeuroChain SDK."""

import pytest
from neurochain_sdk import NeurochainClient


def test_client_instantiation():
    client = NeurochainClient(url="ws://localhost:9944")
    # We cannot query a real node in tests; ensure attribute exists
    assert hasattr(client, "substrate")