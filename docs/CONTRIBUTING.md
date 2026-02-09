# Contributing Guide

Thank you for your interest in contributing to **NeuroMesh**!  We
welcome pull requests, bug reports, and feature requests from the
community.  This document outlines the process for participating in
the project.

## Reporting Issues

If you encounter a bug or have a feature request, please open a new
issue in the repository.  Use the issue templates provided to ensure
you include the necessary information:

* **Bug report** – Include steps to reproduce, expected behavior,
  actual behavior, and environment details (OS, version, etc.).
* **Feature request** – Describe the problem you’re trying to solve,
  proposed solution, and any alternatives considered.

Please search existing issues before opening a new one to avoid
duplicates.

## Development Workflow

1. **Pick an issue.**  Consult the [backlog](backlog.md) to find
   issues labeled `status:ready`.  Comment on the issue to indicate
   you’re working on it.
2. **Create a branch.**  Use the naming convention
   `issue-<number>-short-description`, for example:

   ```bash
   git checkout -b issue-42-subnet-pallet
   ```

3. **Develop.**  Make small, well‑documented commits.  Follow the
   coding standards of each language (Rust `rustfmt`/`clippy`, Python
   `black`/`flake8`, TypeScript `prettier`/`eslint`).

4. **Write tests.**  Add unit and integration tests for your changes.
   Ensure that `cargo test`, `pytest`, and `npm test` all pass.

5. **Submit a pull request.**  Reference the issue in your PR
   description using `Closes #<issue-number>`.  Include a summary of
   your changes and any trade‑offs or design decisions.  Ensure the
   CI pipeline is green before requesting review.

6. **Self‑review and respond to feedback.**  Conduct an initial
   self‑review, leaving comments to explain non‑obvious code.  Address
   reviewer comments promptly and respectfully.

7. **Merge.**  Once your PR is approved and CI passes, it will be
   merged by a maintainer.  We use a **squash merge** strategy to
   maintain a clean commit history on `main`.

## Coding Standards

* **Rust** – Follow the official [Rust style guide](https://github.com/rust-lang/rustfmt) and use `clippy` to catch common issues.
* **Python** – Use [PEP 8](https://peps.python.org/pep-0008/) and run
  `black` and `flake8` before committing.
* **TypeScript** – Use `eslint` and `prettier` with the configuration
  in `src/aggregator`.

## Commit Messages

Make your commit messages descriptive and concise.  Each commit should
contain a single logical change.  When simulating history for issues,
you may set `GIT_AUTHOR_DATE` and `GIT_COMMITTER_DATE` environment
variables to backdate commits for narrative purposes.

## Code of Conduct

Be respectful and inclusive.  We abide by the
[Contributor Covenant](https://www.contributor-covenant.org/version/2/0/code_of_conduct/).
Harassment or discrimination of any kind will not be tolerated.