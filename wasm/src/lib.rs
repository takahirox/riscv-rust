extern crate wasm_bindgen;
extern crate riscv_emu_rust;

mod wasm_terminal;

use wasm_bindgen::prelude::*;

use riscv_emu_rust::Emulator;
use wasm_terminal::WasmTerminal;

#[wasm_bindgen]
pub struct WasmRiscv {
	emulator: Emulator
}

#[wasm_bindgen]
impl WasmRiscv {
	pub fn new() -> Self {
		WasmRiscv {
			emulator: Emulator::new(Box::new(WasmTerminal::new()))
		}
	}

	pub fn init(&mut self, kernel_contents: Vec<u8>, fs_contents: Vec<u8>, dtb_contents: Vec<u8>) {
		self.emulator.setup_from_elf(kernel_contents);
		self.emulator.setup_filesystem(fs_contents);
		self.emulator.setup_dtb(dtb_contents);
	}

	pub fn run(&mut self) {
		self.emulator.run();
	}

	pub fn run_cycles(&mut self, cycles: u32) {
		for _i in 0..cycles {
			self.emulator.tick();
		}
	}

	pub fn disassemble_next_instruction(&mut self) {
		let s = self.emulator.get_mut_cpu().disassemble_next_instruction();
		let bytes = s.as_bytes();
		for i in 0..bytes.len() {
			self.emulator.get_mut_terminal().put_byte(bytes[i]);
		}
	}

	pub fn get_output(&mut self) -> u8 {
		self.emulator.get_mut_terminal().get_output()
	}

	pub fn put_input(&mut self, data: u8) {
		self.emulator.get_mut_terminal().put_input(data);
	}
}