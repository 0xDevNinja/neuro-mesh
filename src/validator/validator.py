"""
Reference validator implementation.

This script illustrates the core loop of a NeuroMesh validator:

1. Discover miners (hard‑coded list for now).
2. Send inference queries.
3. Score the outputs using a simple heuristic.
4. Produce a weight vector over the miners.
5. (Not implemented) Submit the weight vector to the chain.

This is a placeholder until the full sampling and scoring protocol is
implemented.
"""

import json
import time
from http.client import HTTPConnection
from typing import Dict, List
import click
import numpy as np


@click.command()
@click.option("--miners", multiple=True, default=["127.0.0.1:5000"], help="List of miner host:port endpoints to query.")
@click.option("--query", default="What is 2+2?", help="Inference prompt to send to miners.")
def run_validator(miners: List[str], query: str) -> None:
    """Run the validator sampling loop once."""
    click.echo(f"Running validator against miners: {', '.join(miners)}")
    scores: Dict[str, float] = {}

    for miner_addr in miners:
        host, port = miner_addr.split(":")
        conn = HTTPConnection(host, int(port))
        payload = json.dumps({"input": query}).encode("utf-8")
        headers = {"Content-Type": "application/json"}
        try:
            conn.request("POST", "/", body=payload, headers=headers)
            response = conn.getresponse()
            body = response.read().decode("utf-8")
            data = json.loads(body)
            output = data.get("output", "")
            # Placeholder scoring heuristic: length of output string.
            scores[miner_addr] = float(len(output))
            click.echo(f"Received response from {miner_addr}: {output}")
        except Exception as e:
            click.echo(f"Error contacting miner {miner_addr}: {e}")
            scores[miner_addr] = 0.0
        finally:
            conn.close()

    # Normalize scores to a weight vector.
    values = np.array(list(scores.values()))
    total = values.sum() if values.sum() > 0 else 1.0
    weights = values / total

    click.echo("Computed weight vector:")
    for miner, weight in zip(scores.keys(), weights):
        click.echo(f"  {miner}: {weight:.3f}")

    # TODO: Submit weights on‑chain via the SDK


if __name__ == "__main__":
    run_validator()