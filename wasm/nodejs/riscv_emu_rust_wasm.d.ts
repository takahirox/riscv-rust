/* tslint:disable */
/* eslint-disable */
/**
*/
export class WasmRiscv {
  free(): void;
/**
* @returns {WasmRiscv} 
*/
  static new(): WasmRiscv;
/**
* @param {Uint8Array} kernel_contents 
* @param {Uint8Array} fs_contents 
* @param {Uint8Array} dtb_contents 
*/
  init(kernel_contents: Uint8Array, fs_contents: Uint8Array, dtb_contents: Uint8Array): void;
/**
*/
  run(): void;
/**
* @param {number} cycles 
*/
  run_cycles(cycles: number): void;
/**
* Runs program until breakpoints. Also known as debugger's continue command.
* This method takes max_cycles. If the program doesn't hit any breakpoints
* in max_cycles cycles this method returns false. Otherwise true.
*
* Even without this method, you can write the same behavior JS code as the
* following code. But JS-WASM bridge cost isn't ignorable now. So this method
* has been introduced.
*
* ```
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
* @param {number} reg 
* @returns {BigInt} 
*/
  read_register(reg: number): BigInt;
/**
* @returns {BigInt} 
*/
  read_pc(): BigInt;
/**
* @returns {number} 
*/
  get_output(): number;
/**
* @param {number} data 
*/
  put_input(data: number): void;
}
