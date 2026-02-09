"""Unit tests for the reference miner."""

import json
import sys
from http.client import HTTPConnection
from threading import Thread

# Ensure we can import the miner module when running tests from the
# project root.  This adds the src/miner directory to sys.path.
from pathlib import Path
sys.path.append(str(Path(__file__).resolve().parent.parent))

import miner


def test_miner_response():
    # Run the miner in a separate thread on a random port.
    port = 5600

    def run_server():
        miner.serve.callback(host="127.0.0.1", port=port)

    thread = Thread(target=run_server, daemon=True)
    thread.start()

    conn = HTTPConnection("127.0.0.1", port)
    payload = json.dumps({"input": "hello"}).encode("utf-8")
    headers = {"Content-Type": "application/json"}
    conn.request("POST", "/", body=payload, headers=headers)
    response = conn.getresponse()
    body = response.read().decode("utf-8")
    conn.close()

    assert response.status == 200
    data = json.loads(body)
    assert "Hello from NeuroMesh miner" in data["output"]