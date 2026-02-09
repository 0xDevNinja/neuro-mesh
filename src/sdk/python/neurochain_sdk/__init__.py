"""NeuroChain Python SDK.

This package provides a convenient Python interface for interacting
with NeuroChain nodes.  It wraps common RPC calls and will offer
higher‑level abstractions for miners and validators.  See
`client.py` for the current client implementation.
"""

from .client import NeurochainClient

__all__ = ["NeurochainClient"]