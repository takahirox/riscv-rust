cargo build --release --target wasm32-unknown-unknown
wasm-bindgen ../target/wasm32-unknown-unknown/release/riscv_emu_rust_wasm.wasm --out-dir ./web --target web --no-typescript
wasm-bindgen ../target/wasm32-unknown-unknown/release/riscv_emu_rust_wasm.wasm --out-dir ./npm/nodejs --nodejs
wasm-bindgen ../target/wasm32-unknown-unknown/release/riscv_emu_rust_wasm.wasm --out-dir ./npm/pkg
