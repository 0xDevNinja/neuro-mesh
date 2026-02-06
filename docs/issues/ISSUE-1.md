# Issue 1: Initialize Workspace and Primitive Crates

## Scope
- Add a `sp-neuro-core` primitives crate to define core traits shared by pallets,
  off-chain workers, and runtime logic.
- Provide initial trait contracts for `NeuralTask` and `MeshProvider`.
- Register the new crate in the workspace so it can be consumed by other crates.

## Acceptance Criteria
- The `sp-neuro-core` crate builds in `no_std` mode and includes clear `///` doc
  comments for all public types and functions.
- The `NeuralTask` and `MeshProvider` traits are defined with SCALE-encodable
  associated types suitable for on-chain usage.
- `cargo test -p sp-neuro-core` passes in the workspace.

## Technical Hurdles
- Keep the crate `no_std` compatible and avoid pulling in heavy dependencies.
- Design trait bounds that work for both runtime and off-chain contexts.
- Avoid locking the project into a premature concrete task representation.
