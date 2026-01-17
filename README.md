# github-bot-sdk

[![Crates.io](https://img.shields.io/crates/v/github-bot-sdk.svg)](https://crates.io/crates/github-bot-sdk)
[![Documentation](https://docs.rs/github-bot-sdk/badge.svg)](https://docs.rs/github-bot-sdk)
[![CI](https://github.com/pvandervelde/github-bot-sdk/actions/workflows/ci.yml/badge.svg)](https://github.com/pvandervelde/github-bot-sdk/actions/workflows/ci.yml)
[![Codecov](https://codecov.io/gh/pvandervelde/github-bot-sdk/branch/master/graph/badge.svg)](https://codecov.io/gh/pvandervelde/github-bot-sdk)
[![License](https://img.shields.io/crates/l/github-bot-sdk.svg)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)

A comprehensive GitHub Bot SDK for Rust, providing authentication, webhook validation, and API client functionality.

## Features

- **GitHub App Authentication** - JWT signing and installation token management
- **Webhook Validation** - HMAC-SHA256 signature verification
- **API Client** - Typed GitHub REST API wrapper with rate limiting
- **Event Processing** - Webhook event normalization and handling
- **Type Safety** - Leverages Rust's type system for correctness
- **Async/Await** - Built for tokio runtime

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
github-bot-sdk = "0.1.0"
```

## Quick Start

```rust
// TODO: Add quick start example
```

## Documentation

- [API Documentation](https://docs.rs/github-bot-sdk)
- [Specification](docs/specs/)

## Examples

See the [examples/](examples/) directory for complete working examples.

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.
