# Reference Validator

This directory contains a rudimentary **NeuroMesh validator** implemented
in Python.  Validators are responsible for sampling miners, sending
inference queries, computing scores, converting those scores into
weight vectors, and submitting the result on‑chain.  The current
version does not interact with the chain; it simply demonstrates how
a validator might query miners and compute a normalized weight vector.

## Running the Validator

Install dependencies:

```bash
python3 -m venv .venv && source .venv/bin/activate
pip install -r requirements.txt
```

Start one or more miners (see `src/miner`) and then run:

```bash
python validator.py --miners 127.0.0.1:5000 --miners 127.0.0.1:5600 \
  --query "What is the capital of France?"
```

The validator will send the query to each miner, compute a trivial
score based on the length of the response, and print the resulting
weights.  The logic for scoring and weight calculation will evolve as
subnet evaluation specs and consensus rules are implemented.

## Notes

* Validators must register on the chain with a stake and declare their
  evaluation capacity.
* In future versions, sampling of miners will be randomized and
  validators will submit compressed weight vectors to the chain.
* This script uses blocking HTTP requests for simplicity.  A real
  implementation should use async gRPC.