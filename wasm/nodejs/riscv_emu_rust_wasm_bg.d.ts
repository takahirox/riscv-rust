/* tslint:disable */
/* eslint-disable */
export const memory: WebAssembly.Memory;
export function __wbg_wasmriscv_free(a: number): void;
export function wasmriscv_new(): number;
export function wasmriscv_setup_program(a: number, b: number, c: number): void;
export function wasmriscv_load_program_for_symbols(a: number, b: number, c: number): void;
export function wasmriscv_setup_filesystem(a: number, b: number, c: number): void;
export function wasmriscv_setup_dtb(a: number, b: number, c: number): void;
export function wasmriscv_run(a: number): void;
export function wasmriscv_run_cycles(a: number, b: number): void;
export function wasmriscv_run_until_breakpoints(a: number, b: number, c: number, d: number): number;
export function wasmriscv_disassemble_next_instruction(a: number): void;
export function wasmriscv_load_doubleword(a: number, b: number, c: number, d: number, e: number, f: number): void;
export function wasmriscv_read_register(a: number, b: number, c: number): void;
export function wasmriscv_read_pc(a: number, b: number): void;
export function wasmriscv_get_output(a: number): number;
export function wasmriscv_put_input(a: number, b: number): void;
export function wasmriscv_enable_page_cache(a: number, b: number): void;
export function wasmriscv_get_address_of_symbol(a: number, b: number, c: number, d: number, e: number, f: number): void;
export function __wbindgen_malloc(a: number): number;
export function __wbindgen_free(a: number, b: number): void;
export function __wbindgen_realloc(a: number, b: number, c: number): number;
