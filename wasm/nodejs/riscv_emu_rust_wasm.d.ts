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
* @returns {number} 
*/
  get_output(): number;
/**
* @param {number} data 
*/
  put_input(data: number): void;
}
