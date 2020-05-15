/* tslint:disable */
/* eslint-disable */
export const memory: WebAssembly.Memory;
export function __wbg_wasmriscv_free(a: number): void;
export function wasmriscv_new(): number;
export function wasmriscv_init(a: number, b: number, c: number, d: number, e: number, f: number, g: number): void;
export function wasmriscv_run(a: number): void;
export function wasmriscv_run_cycles(a: number, b: number): void;
export function wasmriscv_run_until_breakpoints(a: number, b: number, c: number, d: number): number;
export function wasmriscv_disassemble_next_instruction(a: number): void;
export function wasmriscv_load_doubleword(a: number, b: number, c: number, d: number, e: number, f: number): void;
export function wasmriscv_read_register(a: number, b: number, c: number): void;
export function wasmriscv_read_pc(a: number, b: number): void;
export function wasmriscv_get_output(a: number): number;
export function wasmriscv_put_input(a: number, b: number): void;
export function __wbindgen_malloc(a: number): number;
export function __wbindgen_free(a: number, b: number): void;
