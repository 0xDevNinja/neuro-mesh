"""
Reference miner implementation.

This script provides a minimal scaffold for a NeuroMesh miner.  In
practice, miners will host AI models or pipelines and expose a gRPC
endpoint conforming to the subnet’s input/output schema.  For now,
this script starts an HTTP server that responds to inference requests
with a static message.
"""

import json
from http.server import BaseHTTPRequestHandler, HTTPServer
import click


class InferenceHandler(BaseHTTPRequestHandler):
    """A very simple HTTP handler to simulate inference."""

    def do_POST(self):
        content_length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(content_length)
        try:
            request = json.loads(body.decode("utf-8"))
        except json.JSONDecodeError:
            self.send_response(400)
            self.end_headers()
            self.wfile.write(b"Invalid JSON")
            return

        # Placeholder logic: echo the input with a greeting.
        response = {"output": f"Hello from NeuroMesh miner! You sent: {request.get('input')}"}

        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(json.dumps(response).encode("utf-8"))


@click.command()
@click.option("--host", default="0.0.0.0", help="Host address to bind.")
@click.option("--port", default=5000, help="Port number to listen on.")
def serve(host: str, port: int) -> None:
    """Start the reference miner HTTP server."""
    server = HTTPServer((host, port), InferenceHandler)
    click.echo(f"Starting NeuroMesh miner on http://{host}:{port}")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        click.echo("Shutting down miner...")


if __name__ == "__main__":
    serve()