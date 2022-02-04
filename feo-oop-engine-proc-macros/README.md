<img align="left" alt="" src="../assets/standard-assets/textures/feo-oop-engine-logo.png" height="155" />

# [FeO OOP Engine <b>Proc Macros</b>](https://github.com/littleTitan/feo-oop-engine/feo-oop-engine-proc-macros)

[![Crates.io](https://img.shields.io/crates/v/feo-oop-engine-proc-macros.svg)](https://crates.io/crates/feo-oop-engine-proc-macros)
[![Docs](https://docs.rs/feo-oop-engine/badge.svg)](https://docs.rs/feo-oop-engine-proc-macros)
[![Build Status](https://github.com/littleTitan/feo-oop-engine/workflows/Rust/badge.svg)](https://github.com/littleTitan/feo-oop-engine/actions?query=workflow%3ARust)

see: [feo-oop-engine](https://github.com/littleTitan/feo-oop-engine)

## Description
> Proc Macro Crate for [feo-oop-engine](https://github.com/littleTitan/feo-oop-engine)

This crate allows for the use of derive macros to facilitate the development with and extension of [feo-oop-engine's](https://github.com/littleTitan/feo-oop-engine) crate features.

## Usage
Using a derive macro is simple enough. Simply import the crate and use the macro.
```rust
#[macro_use] extern crate feo_oop_engine_proc_macros;

#[derive(Global)]
struct Globals {
    ..
}
```

# License 
[MIT](../LICENSE.md)