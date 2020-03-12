cargo build --release --lib --target wasm32-unknown-unknown
wasm-bindgen ./target/wasm32-unknown-unknown/release/riscv_rust.wasm --out-dir ./wasm/ --target web --no-typescript