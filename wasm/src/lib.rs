extern crate wasm_bindgen;
extern crate riscv_emu_rust;

use wasm_bindgen::prelude::*;
use std::collections::HashMap;

use riscv_emu_rust::Emulator;
use riscv_emu_rust::default_terminal::DefaultTerminal;

/// `WasmRiscv` is an interface between user JavaScript code and
/// WebAssembly RISC-V emulator. The following code is example
/// JavaScript user code.
///
/// ```ignore
/// // JavaScript code
/// const riscv = WasmRiscv.new();
/// // Setup program content binary
/// riscv.setup_program(new Uint8Array(elfBuffer));
/// // Setup filesystem content binary
/// riscv.setup_filesystem(new Uint8Array(fsBuffer));
///
/// // Emulator needs to break program regularly to handle input/output
/// // because the emulator is currenlty designed to run in a single thread.
/// // Once `SharedArrayBuffer` lands by default in major browsers
/// // we would run input/output handler in another thread.
/// const runCycles = () => {
///   // Run 0x100000 (or certain) cycles, handle input/out,
///   // and fire next cycles.
///   // Note: Evety instruction is completed in a cycle.
///   setTimeout(runCycles, 0);
///   riscv.run_cycles(0x100000);
///
///   // Output handling
///   while (true) {
///     const data = riscv.get_output();
///     if (data !== 0) {
///       // print data
///     } else {
///       break;
///     }
///   }
///
///   // Input handling. Assuming inputs holds
///   // input ascii data.
///   while (inputs.length > 0) {
///     riscv.put_input(inputs.shift());
///   }
/// };
/// runCycles();
/// ```
#[wasm_bindgen]
pub struct WasmRiscv {
	emulator: Emulator
}

#[wasm_bindgen]
impl WasmRiscv {
	/// Creates a new `WasmRiscv`.
	pub fn new() -> Self {
		WasmRiscv {
			emulator: Emulator::new(Box::new(DefaultTerminal::new()))
		}
	}

	/// Sets up program run by the program. This method is expected to be called
	/// only once.
	///
	/// # Arguments
	/// * `content` Program binary
	pub fn setup_program(&mut self, content: Vec<u8>) {
		self.emulator.setup_program(content);
	}

	/// Loads symbols of program and adds them to symbol - virtual address
	/// mapping in `Emulator`.
	///
	/// # Arguments
	/// * `content` Program binary
	pub fn load_program_for_symbols(&mut self, content: Vec<u8>) {
		self.emulator.load_program_for_symbols(content);
	}

	/// Sets up filesystem. Use this method if program (e.g. Linux) uses
	/// filesystem. This method is expected to be called up to only once.
	///
	/// # Arguments
	/// * `content` File system content binary
	pub fn setup_filesystem(&mut self, content: Vec<u8>) {
		self.emulator.setup_filesystem(content);
	}

	/// Sets up device tree. The emulator has default device tree configuration.
	/// If you want to override it, use this method. This method is expected to
	/// to be called up to only once.
	///
	/// # Arguments
	/// * `content` DTB content binary
	pub fn setup_dtb(&mut self, content: Vec<u8>) {
		self.emulator.setup_dtb(content);
	}

	/// Runs program set by `setup_program()`. The emulator won't stop forever
	/// unless [`riscv-tests`](https://github.com/riscv/riscv-tests) programs.
	/// The emulator stops if program is `riscv-tests` program and it finishes.
	pub fn run(&mut self) {
		self.emulator.run();
	}

	/// Runs program set by `setup_program()` in `cycles` cycles.
	///
	/// # Arguments
	/// * `cycles`
	pub fn run_cycles(&mut self, cycles: u32) {
		for _i in 0..cycles {
			self.emulator.tick();
		}
	}

	/// Runs program until breakpoints. Also known as debugger's continue command.
	/// This method takes `max_cycles`. If the program doesn't hit any breakpoint
	/// in `max_cycles` cycles this method returns `false`. Otherwise `true`.
	///
	/// Even without this method, you can write the same behavior JavaScript code
	/// as the following code. But JS-WASM bridge cost isn't ignorable now. So
	/// this method has been introduced.
	///
	/// ```ignore
	/// const runUntilBreakpoints = (riscv, breakpoints, maxCycles) => {
	///   for (let i = 0; i < maxCycles; i++) {
	///     riscv.run_cycles(1);
	///     const pc = riscv.read_pc()
	///     if (breakpoints.includes(pc)) {
	///       return true;
	///     }
	///   }
	///   return false;
	/// };
	/// ```
	///
	/// # Arguments
	/// * `breakpoints` An array including breakpoint virtual addresses
	/// * `max_cycles` See the above description
	pub fn run_until_breakpoints(&mut self, breakpoints: Vec<u64>, max_cycles: u32) -> bool {
		let mut table = HashMap::new();
		for i in 0..breakpoints.len() {
			table.insert(breakpoints[i], true);
		}
		for _i in 0..max_cycles {
			self.emulator.tick();
			let pc = self.emulator.get_cpu().read_pc();
			if table.contains_key(&pc) {
				return true;
			}
		}
		return false;
	}

	/// Disassembles an instruction Program Counter points to.
	/// Use `get_output()` to get the disassembled strings.
	pub fn disassemble_next_instruction(&mut self) {
		let s = self.emulator.get_mut_cpu().disassemble_next_instruction();
		let bytes = s.as_bytes();
		for i in 0..bytes.len() {
			self.emulator.get_mut_terminal().put_byte(bytes[i]);
		}
	}

	/// Loads eight-byte data from memory. Loading can cause an error or trap.
	/// If an error or trap happens `error[0]` holds non-zero error code and
	/// this method returns zero. Otherwise `error[0]` holds zero and this
	/// method returns loaded data.
	///
	/// # Arguments
	/// * `address` eight-byte virtual address
	/// * `error` If an error or trap happens error[0] holds non-zero.
	///    Otherwize zero.
	///   * 0: No error
	///   * 1: Page fault
	///   * 2: Invalid address (e.g. translated physical address points to out
	///        of valid memory address range)
	pub fn load_doubleword(&mut self, address: u64, error: &mut [u8]) -> u64 {
		for i in 0..8 {
			match self.emulator.get_mut_cpu()
				.get_mut_mmu().validate_address(address.wrapping_add(i)) {
				Ok(valid) => {
					if !valid {
						error[0] = 2;
						return 0;
					}
				},
				Err(()) => {
					error[0] = 1;
					return 0;
				}
			}
		}
		match self.emulator.get_mut_cpu().get_mut_mmu().load_doubleword(address) {
			Ok(data) => {
				error[0] = 0;
				data
			},
			Err(_trap) => {
				error[0] = 1;
				0
			}
		}
	}

	/// Reads integer register content.
	///
	/// # Arguments
	/// * `reg` register number. Must be 0-31.
	pub fn read_register(&self, reg: u8) -> u64 {
		self.emulator.get_cpu().read_register(reg) as u64
	}

	/// Reads Program Counter content.
	pub fn read_pc(&self) -> u64 {
		self.emulator.get_cpu().read_pc()
	}

	/// Gets ascii code byte sent from the emulator to terminal.
	/// The emulator holds output buffer inside. This method returns zero
	/// if the output buffer is empty. So if you want to read all buffered
	/// output content, repeatedly call this method until zero is returned.
	///
	/// ```ignore
	/// // JavaScript code
	/// while (true) {
	///   const data = riscv.get_output();
	///   if (data !== 0) {
	///     // print data
	///   } else {
	///     break;
	///   }
	/// }
	/// ```
	pub fn get_output(&mut self) -> u8 {
		self.emulator.get_mut_terminal().get_output()
	}

	/// Puts ascii code byte sent from terminal to the emulator.
	///
	/// # Arguments
	/// * `data` Ascii code byte
	pub fn put_input(&mut self, data: u8) {
		self.emulator.get_mut_terminal().put_input(data);
	}

	/// Enables or disables page cache optimization.
	/// Page cache optimization is an experimental feature.
	/// Refer to [`Mmu`](../riscv_emu_rust/mmu/struct.Mmu.html) for the detail.
	///
	/// # Arguments
	/// * `enabled`
	pub fn enable_page_cache(&mut self, enabled: bool) {
		self.emulator.enable_page_cache(enabled);
	}

	/// Gets virtual address corresponding to symbol strings.
	///
	/// # Arguments
	/// * `s` Symbol strings
	/// * `error` If symbol is not found error[0] holds non-zero.
	///    Otherwize zero.
	pub fn get_address_of_symbol(&mut self, s: String, error: &mut [u8]) -> u64 {
		match self.emulator.get_addredd_of_symbol(&s) {
			Some(address) => {
				error[0] = 0;
				address
			},
			None => {
				error[0] = 1;
				0
			}
		}
	}
}