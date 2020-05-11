let imports = {};
imports['__wbindgen_placeholder__'] = module.exports;
let wasm;
const { TextDecoder } = require(String.raw`util`);

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

let cachegetUint8Memory0 = null;
function getUint8Memory0() {
    if (cachegetUint8Memory0 === null || cachegetUint8Memory0.buffer !== wasm.memory.buffer) {
        cachegetUint8Memory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachegetUint8Memory0;
}

function getStringFromWasm0(ptr, len) {
    return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
}

let WASM_VECTOR_LEN = 0;

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1);
    getUint8Memory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}
/**
*/
class WasmRiscv {

    static __wrap(ptr) {
        const obj = Object.create(WasmRiscv.prototype);
        obj.ptr = ptr;

        return obj;
    }

    free() {
        const ptr = this.ptr;
        this.ptr = 0;

        wasm.__wbg_wasmriscv_free(ptr);
    }
    /**
    * @returns {WasmRiscv}
    */
    static new() {
        var ret = wasm.wasmriscv_new();
        return WasmRiscv.__wrap(ret);
    }
    /**
    * @param {Uint8Array} kernel_contents
    * @param {Uint8Array} fs_contents
    * @param {Uint8Array} dtb_contents
    */
    init(kernel_contents, fs_contents, dtb_contents) {
        var ptr0 = passArray8ToWasm0(kernel_contents, wasm.__wbindgen_malloc);
        var len0 = WASM_VECTOR_LEN;
        var ptr1 = passArray8ToWasm0(fs_contents, wasm.__wbindgen_malloc);
        var len1 = WASM_VECTOR_LEN;
        var ptr2 = passArray8ToWasm0(dtb_contents, wasm.__wbindgen_malloc);
        var len2 = WASM_VECTOR_LEN;
        wasm.wasmriscv_init(this.ptr, ptr0, len0, ptr1, len1, ptr2, len2);
    }
    /**
    */
    run() {
        wasm.wasmriscv_run(this.ptr);
    }
    /**
    * @param {number} cycles
    */
    run_cycles(cycles) {
        wasm.wasmriscv_run_cycles(this.ptr, cycles);
    }
    /**
    */
    disassemble_next_instruction() {
        wasm.wasmriscv_disassemble_next_instruction(this.ptr);
    }
    /**
    * @returns {number}
    */
    get_output() {
        var ret = wasm.wasmriscv_get_output(this.ptr);
        return ret;
    }
    /**
    * @param {number} data
    */
    put_input(data) {
        wasm.wasmriscv_put_input(this.ptr, data);
    }
}
module.exports.WasmRiscv = WasmRiscv;

module.exports.__wbindgen_throw = function(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
};

const path = require('path').join(__dirname, 'riscv_emu_rust_wasm_bg.wasm');
const bytes = require('fs').readFileSync(path);

const wasmModule = new WebAssembly.Module(bytes);
const wasmInstance = new WebAssembly.Instance(wasmModule, imports);
wasm = wasmInstance.exports;
module.exports.__wasm = wasm;

