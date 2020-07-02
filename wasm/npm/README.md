# riscv_emu_rust_wasm

[![npm version](https://badge.fury.io/js/riscv_emu_rust_wasm.svg)](https://badge.fury.io/js/riscv_emu_rust_wasm)

riscv_emu_rust_wasm is a WebAssembly [RISC-V](https://riscv.org/) processor and peripheral devices emulator
based on [riscv-rust](https://github.com/takahirox/riscv-rust).

## How to install

```
$ npm install riscv_emu_rust_wasm
```

## How to use

```javascript
const riscv = require('riscv_emu_rust_wasm').WasmRiscv.new();
// Setup program content binary
riscv.setup_program(new Uint8Array(elfBuffer));
// Setup filesystem content binary
riscv.setup_filesystem(new Uint8Array(fsBuffer));

// Emulator needs to break program regularly to handle input/output
// because the emulator is currenlty designed to run in a single thread.
// Once `SharedArrayBuffer` lands and becomes stable
// we would run input/output handler in another thread.
const runCycles = () => {
  // Run 0x100000 (or certain) cycles, handle input/out,
  // and fire next cycles.
  // Note: Evety instruction is completed in a cycle.
  setTimeout(runCycles, 0);
  riscv.run_cycles(0x100000);

  // Output handling
  while (true) {
    const data = riscv.get_output();
    if (data !== 0) {
      // print data
    } else {
      break;
    }
  }

  // Input handling. Assuming inputs holds
  // input ascii data.
  while (inputs.length > 0) {
    riscv.put_input(inputs.shift());
  }
};
runCycles();
```

## API

Refer to [the comments in WasmRiscv](https://github.com/takahirox/riscv-rust/blob/master/wasm/src/lib.rs)

## How to build WebAssembly RISC-V emulator locally

Prerequirements
- Install [wasm-bindgen client](https://rustwasm.github.io/docs/wasm-bindgen/)

```sh
$ git clone https://github.com/takahirox/riscv-rust.git
$ cd riscv-rust/wasm
$ bash build.sh
```
