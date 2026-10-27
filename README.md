<h1 align="center">BRC2.0 - Programmable Module</h1>
<p align="center">Smart contract execution engine compatible with BRC20 standard.</p>
<div align="center">

[![BRC2.0](https://github.com/bestinslot-xyz/brc20-programmable-module/actions/workflows/rust.yml/badge.svg)](https://github.com/bestinslot-xyz/brc20-programmable-module/actions)

Join us: [Discord](https://discord.com/invite/6G8yPAcP3Z) / [Telegram](https://t.me/bestinslotxyz)

</div>

BRC2.0 programmable module provides smart contract execution capabilities for BRC20 indexers.

This module allows users to inscribe smart contracts and function calls on Bitcoin blockchain to implement decentralised applications.

BRC2.0 runs on a custom `EVM` execution engine using [`revm`](https://github.com/bluealloy/revm). Our main reasons for choosing `EVM` are listed below:

- Rich open-source ecosystem for tooling, including several different execution engines
- Heavily tested open-source smart contract libraries that are readily available for various financial applications
- Large and active developer community - many smart contract developers are already familiar with `EVM` and `Solidity`
- `EVM` is deterministic and Turing complete.

See our proposal at [bestinslot-xyz/brc20-prog-module-proposal](https://github.com/bestinslot-xyz/brc20-prog-module-proposal) for detailed information about how the BRC2.0 programmable module works.

See the [Indexer Integration Guide](docs/indexer-integration.md) on how to integrate the programmable module your BRC20 indexer.

For questions, comments and requests, use [the issues section](https://github.com/bestinslot-xyz/brc20-programmable-module/issues) or [Best in Slot discord server](https://discord.com/invite/6G8yPAcP3Z).

> [!NOTE]
> This module is enabled on Bitcoin mainnet.

## Usage

BRC2.0 Programmable Module is written in [Rust](https://www.rust-lang.org/), so you need Cargo installed in order to build and run the server.

Precompiled contracts require environment variables to work properly, see [Configuration](docs/configuration.md) and [Precompiles](docs/precompiles.md) to learn how to set them up, otherwise precompiled contracts will fail.

**Build and run brc20_prog:**

```
cargo run --release
```

> [!NOTE]
> You must use clang as CC. Try installing clang `sudo apt install clang` before running `brc20_prog`.
> 
> Eg. `CC=/usr/bin/clang CXX=/usr/bin/clang++`. Clang llvm version must be the same as the one used by rust compiler. On the rust side you should use `RUSTFLAGS="-Clinker-plugin-lto -Clinker=clang -Clink-arg=-fuse-ld=lld"`.

**Run with Docker:**

A prebuilt image is published to Docker Hub as [`bestinslot/brc20-prog`](https://hub.docker.com/r/bestinslot/brc20-prog). Image tags track release versions (e.g. `bestinslot/brc20-prog:0.16.0`), with `latest` pointing at the newest release.

```
docker run --rm \
  -p 18545:18545 \
  -v brc20_prog_data:/var/lib/brc20-prog/data \
  --env-file .env \
  bestinslot/brc20-prog:latest
```

The image binds the JSON-RPC server to `0.0.0.0:18545` and stores its database under `/var/lib/brc20-prog/data` (mount a volume there to persist it). All other configuration is provided through environment variables, see [`env.sample`](env.sample) and [Configuration](docs/configuration.md). Note that `BITCOIN_RPC_URL` must point at a Bitcoin RPC endpoint reachable from inside the container.

To build the image from source instead, run `docker build -t brc20-prog .` from the repository root.

**Embed in an indexer:**

An indexer written in Rust can run the module in its own process instead of calling it over JSON-RPC. See [Embedding in an indexer](docs/embedding.md).

## Configuration

The server is configured through environment variables, and loads a `.env` file from the working directory. Start from [`env.sample`](env.sample); every variable, its default, authorization and the admin RPC listener are described in [Configuration](docs/configuration.md).

## Documentation

- [Configuration](docs/configuration.md): environment variables, command-line options, authorization and the admin RPC server.
- [Indexer Integration Guide](docs/indexer-integration.md): BRC2.0 inscriptions, initialisation, the per-block indexing loop and the indexer checklist.
- [Indexer JSON-RPC methods](docs/indexer-api.md): reference for the `brc20_*` methods.
- [Ethereum JSON-RPC methods](docs/eth-api.md): the `eth_*`, `debug_*` and `txpool_content` methods.
- [Embedding in an indexer](docs/embedding.md): running the module in-process through `Brc20Prog`.
- [Precompiles](docs/precompiles.md): BIP322 verification and the Bitcoin precompiles, with their Solidity interfaces.
- [Differences from Ethereum](docs/differences-from-ethereum.md): consensus behaviour that differs from Ethereum, and the hardfork activation heights.
- [Testing](docs/testing.md): consensus fixtures, the inscription decoding corpus, the public API snapshot and the Bitcoin RPC tests.
- [CHANGELOG](CHANGELOG.md): release notes.

## License

Licensed under the [Apache License, Version 2.0](LICENSE). See [NOTICE](NOTICE).
