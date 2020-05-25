[riscv-rust/wasm/web](https://github.com/takahirox/riscv-rust/tree/master/wasm/web) is a directory for WebAssembly RISC-V emulator compiled from [riscv-rust](https://github.com/takahirox/riscv-rust) and its online demo. You can import the emulator into your web page.

## Online Demo

[index.html](https://takahirox.github.io/riscv-rust/wasm/web/index.html)

## How to import in a web page

Download [riscv_emu_rust_wasm.js](https://github.com/takahirox/riscv-rust/blob/master/wasm/web/riscv_emu_rust_wasm.js) and [riscv_emu_rust_wasm_bg.wasm](https://github.com/takahirox/riscv-rust/blob/master/wasm/web/riscv_emu_rust_wasm_bg.wasm), and place them to where a web page can access.

Below is the example code to import and use them.

```javascript
<script type="module">
  import init, { WasmRiscv } from "./riscv_emu_rust_wasm.js";
  init().then(async wasm => {
    const riscv = WasmRiscv.new();
    const programBuffer = await fetch(path_to_program).then(res => res.arrayBuffer());
    riscv.setup_program(new Uint8Array(programBuffer));

    // Emulator needs to break program regularly to handle input/output
    // because the emulator is currenlty designed to run in a single thread.
    // Once `SharedArrayBuffer` lands and becomes stable in all mejor browsers
    // we would run input/output handler in another thread.
    const runCycles = () => {
      // Run 0x100000 (or certain) cycles, handle input/out,
      // and fire next cycles.
      // Note: Every instruction is completed in a cycle.
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
  });
</script>
```

## API

Refer to the comments in [`WasmRiscv`](https://github.com/takahirox/riscv-rust/blob/master/wasm/src/lib.rs)

## How to build WebAssembly RISC-V emulator and run demo in web browser locally

Prerequirements
- Install [wasm-bindgen client](https://rustwasm.github.io/docs/wasm-bindgen/)

```sh
$ git clone https://github.com/takahirox/riscv-rust.git
$ cd riscv-rust/wasm
$ bash build.sh
# boot local server and access riscv-rust/wasm/web/index.html
```
