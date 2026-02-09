# Reference Miner

This directory contains a simple reference implementation of a
**NeuroMesh miner** written in Python.  In a production system,
miners host AI models or pipelines and expose inference endpoints.  The
reference miner here serves as a placeholder until more complex models
and gRPC services are implemented.

## Running the Miner

Install dependencies:

```bash
python3 -m venv .venv && source .venv/bin/activate
pip install -r requirements.txt
```

Then start the miner:

```bash
python miner.py --port 5000
```

Send a request using `curl`:

```bash
curl -X POST -H "Content-Type: application/json" \
  -d '{"input": "2+2"}' http://localhost:5000
```

The server will respond with a JSON object containing a greeting and
the input you sent.  This stubbed behavior will be replaced by
subnet‑specific inference logic in the future.

## Notes

* Miners must register on the chain with a stake and declare the
  subnets they support.
* Real miners should use gRPC and adhere to the protobuf definitions
  defined by each subnet.  See `src/aggregator/protos` for examples.
* Refer to the backlog issue **NODE‑002** for tasks related to the
  miner server.