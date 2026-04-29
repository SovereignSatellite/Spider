# Spider

Spider is an experimental compiler based on WebAssembly semantics and the Regionalized Value State Dependence Graph research. It compiles `.wasm` binaries to Luau or LuaJIT source files.

## Install

Prebuilt binaries for Windows, Linux, and macOS are available in the "Releases" tab. The CLI can be installed directly from `cargo` too. Prefer pulling the `stable` branch or a tag when doing so.

```sh
$ cargo install --branch stable --git "https://github.com/SovereignSatellite/Spider" spider-cli
$ spider-cli --help
```
