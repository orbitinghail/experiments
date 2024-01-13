# experiments

## Hashing stuff

I'm pretty interested in both blake3 and polymur hash functions. This repo contains some quick and dirty programs which can be used to test these functions both natively and with various wasm targets. This repo also includes some test of umash, but they don't compile to wasm.

```bash
# benchmark native using hyperfine
cargo build --release -p hash && hyperfine -L hasher blake3,polymur,umash-fingerprint 'target/release/hash {hasher} 4096 10000'

# benchmark wasm32-unknown-unknown using wasm-pack
cd crates/hash-js && wasm-pack test --chrome --release
# then open http://127.0.0.1:8000

# benchmark wasm32-wasi using hyperfine & wasmer
cargo build --release --target wasm32-wasi && hyperfine -L hasher blake3,polymur -L buf 4096,16384 'wasmer run target/wasm32-wasi/release/hash.wasm {hasher} {buf} 10000' 
```