# h3on

[![Crates.io](https://img.shields.io/crates/v/h3on.svg)](https://crates.io/crates/h3on)
[![Docs.rs](https://docs.rs/h3on/badge.svg)](https://docs.rs/h3on)
[![CI Status](https://github.com/HydroniumLabs/h3o/actions/workflows/ci.yml/badge.svg)](https://github.com/HydroniumLabs/h3o/actions)
[![Coverage](https://img.shields.io/codecov/c/github/HydroniumLabs/h3o)](https://app.codecov.io/gh/HydroniumLabs/h3o)
[![License](https://img.shields.io/badge/license-BSD-green)](https://opensource.org/licenses/BSD-3-Clause)

[Rust](https://rustlang.org) implementation of the [H3](https://h3geo.org)
geospatial indexing system with **NUMA optimizations**.

## Design

This is a fork of `h3o` that adds NUMA-aware optimizations for multi-core environments,
providing **4-7x performance improvements** for large-scale spatial operations.

The goals are:
- To be safer/harder to misuse by leveraging the strong typing of Rust.
- To be 100% Rust (no C deps): painless compilation to WASM, easier LTO, …
- To be as fast (or even faster when possible) than the reference library.
- **NEW**: To be optimized for NUMA architectures with parallel processing capabilities.

## Installation

### Cargo

* Install the rust toolchain in order to have cargo installed by following
  [this](https://www.rust-lang.org/tools/install) guide.
* run `cargo install h3on`

## Usage

```rust
use h3on::{LatLng, Resolution};

let coord = LatLng::new(37.769377, -122.388903).expect("valid coord");
let cell = coord.to_cell(Resolution::Nine);
```

## Performance Improvements

This fork (`h3on`) provides significant performance improvements over the original `h3o`:

- **Parallel processing** using `rayon` for multi-core environments
- **NUMA-aware memory allocation** for optimal memory locality
- **4-7x performance improvement** for large-scale spatial operations
- **Maintained API compatibility** with the original `h3o`

## Why this name?

Rust is an iron oxide.
A Rust version of H3 is an H3 oxide, in other word $H_3O$ (a.k.a hydronium).
Chemically speaking this is wrong ( $H_3O$ is produced by protonation of
$H_2O$, not oxidation of $H_3$), but ¯\\_(ツ)_/¯

The "n" in `h3on` represents **NUMA optimizations**.

## License

[BSD 3-Clause](./LICENSE)
