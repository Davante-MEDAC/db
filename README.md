## Development

### Caveats

#### macOS LLVM wasm32 support

System `clang` on macOS lacks `wasm32-unknown-unknown` backend. `sqlite-wasm-rs` compiles C to WASM
and needs LLVM with WASM support.

```bash
brew install llvm
export CC_wasm32_unknown_unknown="$(brew --prefix llvm)/bin/clang"
export AR_wasm32_unknown_unknown="$(brew --prefix llvm)/bin/llvm-ar"
just run
```
