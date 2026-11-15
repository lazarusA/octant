# Contributing to octant

Thank you for your interest in contributing to `octant`! Contributions of all kinds—bug reports, feature requests, documentation, and pull requests—are welcome.

## Contributor License Agreement (CLA)

First-time contributors will be automatically prompted to sign our [Contributor License Agreement (CLA)](CLA.md) via a GitHub bot when submitting a Pull Request.

- **One-time signature**: The signature is tracked per GitHub username and only needs to be completed once.
- **Relicensing & Dual-Licensing**: The CLA ensures that `octant` maintainers retain clean intellectual property rights for dual-licensing (MIT/Apache-2.0) and future distribution.

## AI-Assisted Contributions

This project is **AI-friendly**. You are welcome to use AI assistants (coding agents, LLMs, auto-completion tools, etc.) to help write code, write tests, or improve documentation.

However, all AI-generated or AI-assisted contributions must adhere to the following policy:

- **Human Verification Required**: As the author of the pull request, you are responsible for personally reading, understanding, reviewing, and testing the functionality and code before submitting a PR. Unreviewed or unverified automated outputs will be rejected.

## Development Guidelines

1. **Format & Lint**: Ensure code is formatted with `cargo fmt --all -- --check` and passes `cargo clippy --all-targets -- -D warnings`.
2. **Verification**: Verify that the project compiles and tests pass locally (`cargo test --tests`).
3. **Pull Requests**: Open a PR with a clear summary of the changes and motivation behind them.

## Extending Octant (New Plot Types & Storage Formats)

Octant features a decoupled component architecture designed for straightforward extensibility:

- **Adding a New Visualization (e.g. Hexagonal DGGS, Vector Quiver, Custom Meshes)**: Follow [Adding a plot type in ARCHITECTURE.md](ARCHITECTURE.md#adding-a-plot-type).
- **Adding a Storage Backend (e.g. GeoParquet, Cloud Stores)**: Follow [Adding a format in ARCHITECTURE.md](ARCHITECTURE.md#adding-a-format).
- **Complex & Non-Grid Workflows (Lagrangian Trajectories, Streamlines)**: Not built yet; see [Not built yet in ARCHITECTURE.md](ARCHITECTURE.md#not-built-yet) and open an issue to discuss the design first.
