/* tslint:disable */
/* eslint-disable */
/**
* `WasmRiscv` is an interface between user JavaScript code and
* WebAssembly RISC-V emulator. The following code is example
* JavaScript user code.
*
* ```ignore
* // JavaScript code
* const riscv = WasmRiscv.new();
* // Setup program content binary
* riscv.setup_program(new Uint8Array(elfBuffer));
* // Setup filesystem content binary
* riscv.setup_filesystem(new Uint8Array(fsBuffer));
*
* // Emulator needs to break program regularly to handle input/output
* // because the emulator is currenlty designed to run in a single thread.
* // Once `SharedArrayBuffer` lands by default in major browsers
* // we would run input/output handler in another thread.
* const runCycles = () => {
*   // Run 0x100000 (or certain) cycles, handle input/out,
*   // and fire next cycles.
*   // Note: Evety instruction is completed in a cycle.
*   setTimeout(runCycles, 0);
*   riscv.run_cycles(0x100000);
*
*   // Output handling
*   while (true) {
*     const data = riscv.get_output();
*     if (data !== 0) {
*       // print data
*     } else {
*       break;
*     }
*   }
*
*   // Input handling. Assuming inputs holds
*   // input ascii data.
*   while (inputs.length > 0) {
*     riscv.put_input(inputs.shift());
*   }
* };
* runCycles();
* ```
*/
export class WasmRiscv {
  free(): void;
/**
* Creates a new `WasmRiscv`.
* @returns {WasmRiscv}
*/
  static new(): WasmRiscv;
/**
* Sets up program run by the program. This method is expected to be called
* only once.
*
* # Arguments
* * `content` Program binary
* @param {Uint8Array} content
*/
  setup_program(content: Uint8Array): void;
/**
* Loads symbols of program and adds them to symbol - virtual address
* mapping in `Emulator`.
*
* # Arguments
* * `content` Program binary
* @param {Uint8Array} content
*/
  load_program_for_symbols(content: Uint8Array): void;
/**
* Sets up filesystem. Use this method if program (e.g. Linux) uses
* filesystem. This method is expected to be called up to only once.
*
* # Arguments
* * `content` File system content binary
* @param {Uint8Array} content
*/
  setup_filesystem(content: Uint8Array): void;
/**
* Sets up device tree. The emulator has default device tree configuration.
* If you want to override it, use this method. This method is expected to
* to be called up to only once.
*
* # Arguments
* * `content` DTB content binary
* @param {Uint8Array} content
*/
  setup_dtb(content: Uint8Array): void;
/**
* Runs program set by `setup_program()`. The emulator won't stop forever
* unless [`riscv-tests`](https://github.com/riscv/riscv-tests) programs.
* The emulator stops if program is `riscv-tests` program and it finishes.
*/
  run(): void;
/**
* Runs program set by `setup_program()` in `cycles` cycles.
*
* # Arguments
* * `cycles`
* @param {number} cycles
*/
  run_cycles(cycles: number): void;
/**
* Runs program until breakpoints. Also known as debugger's continue command.
* This method takes `max_cycles`. If the program doesn't hit any breakpoint
* in `max_cycles` cycles this method returns `false`. Otherwise `true`.
*
* Even without this method, you can write the same behavior JavaScript code
* as the following code. But JS-WASM bridge cost isn't ignorable now. So
* this method has been introduced.
*
* ```ignore
* const runUntilBreakpoints = (riscv, breakpoints, maxCycles) => {
*   for (let i = 0; i < maxCycles; i++) {
*     riscv.run_cycles(1);
*     const pc = riscv.read_pc()
*     if (breakpoints.includes(pc)) {
*       return true;
*     }
*   }
*   return false;
* };
* ```
*
* # Arguments
* * `breakpoints` An array including breakpoint virtual addresses
* * `max_cycles` See the above description
* @param {BigUint64Array} breakpoints
* @param {number} max_cycles
* @returns {boolean}
*/
  run_until_breakpoints(breakpoints: BigUint64Array, max_cycles: number): boolean;
/**
* Disassembles an instruction Program Counter points to.
* Use `get_output()` to get the disassembled strings.
*/
  disassemble_next_instruction(): void;
/**
* Loads eight-byte data from memory. Loading can cause an error or trap.
* If an error or trap happens `error[0]` holds non-zero error code and
* this method returns zero. Otherwise `error[0]` holds zero and this
* method returns loaded data.
*
* # Arguments
* * `address` eight-byte virtual address
* * `error` If an error or trap happens error[0] holds non-zero.
*    Otherwize zero.
*   * 0: No error
*   * 1: Page fault
*   * 2: Invalid address (e.g. translated physical address points to out
*        of valid memory address range)
* @param {BigInt} address
* @param {Uint8Array} error
* @returns {BigInt}
*/
  load_doubleword(address: BigInt, error: Uint8Array): BigInt;
/**
* Reads integer register content.
*
* # Arguments
* * `reg` register number. Must be 0-31.
* @param {number} reg
* @returns {BigInt}
*/
  read_register(reg: number): BigInt;
/**
* Reads Program Counter content.
* @returns {BigInt}
*/
  read_pc(): BigInt;
/**
* Gets ascii code byte sent from the emulator to terminal.
* The emulator holds output buffer inside. This method returns zero
* if the output buffer is empty. So if you want to read all buffered
* output content, repeatedly call this method until zero is returned.
*
* ```ignore
* // JavaScript code
* while (true) {
*   const data = riscv.get_output();
*   if (data !== 0) {
*     // print data
*   } else {
*     break;
*   }
* }
* ```
* @returns {number}
*/
  get_output(): number;
/**
* Puts ascii code byte sent from terminal to the emulator.
*
* # Arguments
* * `data` Ascii code byte
* @param {number} data
*/
  put_input(data: number): void;
/**
* Enables or disables page cache optimization.
* Page cache optimization is an experimental feature.
* Refer to [`Mmu`](../riscv_emu_rust/mmu/struct.Mmu.html) for the detail.
*
* # Arguments
* * `enabled`
* @param {boolean} enabled
*/
  enable_page_cache(enabled: boolean): void;
/**
* Gets virtual address corresponding to symbol strings.
*
* # Arguments
* * `s` Symbol strings
* * `error` If symbol is not found error[0] holds non-zero.
*    Otherwize zero.
* @param {string} s
* @param {Uint8Array} error
* @returns {BigInt}
*/
  get_address_of_symbol(s: string, error: Uint8Array): BigInt;
}
