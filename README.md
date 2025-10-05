# Spider

Spider is a compiler backend based on WebAssembly semantics. A single target front end is currently supported, which consumes `.wasm` binary modules and produces `.luau` source files.

## Install

You can install the standalone executable through `cargo`, and then run `wasm2luau --help` for usage.

```sh
$ cargo install --git "https://github.com/SovereignSatellite/Spider"
$ wasm2luau --help
```
