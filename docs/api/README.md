# NeuroMesh Aggregator API

This directory contains the public API contracts that downstream SDKs and
integrations target.

## Files

- `openapi.yaml` — OpenAPI 3.0 spec for the public HTTP surface (query, miner
  listing, subnet status, health).

## Authentication

All endpoints except `/v1/health` require an API key via the
`X-NeuroMesh-Key` header. Keys are issued through the aggregator's admin
tooling — see `src/api/aggregator/README.md` for provisioning.

## Rate limits

- Default: **60 requests/minute** per API key for `POST /v1/subnets/{id}/query`.
- Default: **600 requests/minute** per API key for the `GET` endpoints.
- 429 responses include `Retry-After`, `X-RateLimit-Limit`, and
  `X-RateLimit-Remaining` headers.

## Ensemble strategies

| Strategy        | Behavior                                                  |
| --------------- | --------------------------------------------------------- |
| `majority`      | Simple majority vote over top-k responses.                |
| `weighted_avg`  | Weighted average by each miner's current on-chain weight. |
| `best_of_k`     | Return the single miner response with the highest weight. |

## Error model

All errors are JSON objects matching the `Error` schema:

```json
{ "code": "RATE_LIMITED", "message": "…", "request_id": "…" }
```

`code` is machine-readable and stable across releases; `message` is
human-readable and may change. `request_id` mirrors the aggregator's
internal trace id for support escalations.
