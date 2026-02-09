"""
NeuroChain Python client.

This client uses `substrate-interface` to connect to a NeuroChain node
via WebSockets or HTTP.  It provides minimal wrappers around common
RPC calls.  Additional methods will be added as the runtime evolves.
"""

from substrateinterface import SubstrateInterface, Keypair
from typing import Optional


class NeurochainClient:
    """A thin wrapper around SubstrateInterface."""

    def __init__(self, url: str, mnemonic: Optional[str] = None):
        self.substrate = SubstrateInterface(url=url)
        self.keypair = Keypair.create_from_mnemonic(mnemonic) if mnemonic else None

    def get_block_number(self) -> int:
        """Return the current block number."""
        header = self.substrate.get_block_header()
        return header['header']['number']

    # TODO: implement methods for registration, weight submission,
    # and state queries.