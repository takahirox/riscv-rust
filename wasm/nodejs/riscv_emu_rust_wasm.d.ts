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
