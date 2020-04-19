
let wasm;

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
export class WasmRiscv {

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

async function load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {

        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);

            } catch (e) {
                if (module.headers.get('Content-Type') != 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);

    } else {

        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };

        } else {
            return instance;
        }
    }
}

async function init(input) {
    if (typeof input === 'undefined') {
        input = import.meta.url.replace(/\.js$/, '_bg.wasm');
    }
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbindgen_throw = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };

    if (typeof input === 'string' || (typeof Request === 'function' && input instanceof Request) || (typeof URL === 'function' && input instanceof URL)) {
        input = fetch(input);
    }

    const { instance, module } = await load(await input, imports);

    wasm = instance.exports;
    init.__wbindgen_wasm_module = module;

    return wasm;
}

export default init;

