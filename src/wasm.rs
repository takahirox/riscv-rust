extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

mod cpu;
mod display;
mod wasm_display;

use cpu::Cpu;
use cpu::Xlen;
use wasm_display::WasmDisplay;

#[wasm_bindgen]
pub struct WasmRiscv {
	cpu: Cpu
}

#[wasm_bindgen]
impl WasmRiscv {
	pub fn new() -> Self {
		let display = Box::new(WasmDisplay::new());
		WasmRiscv {
			cpu: Cpu::new(Xlen::Bit64, display)
		}
	}
	
	pub fn init(&mut self, kernel_contents: Vec<u8>, image_contents: Vec<u8>) {
		self.cpu.init(kernel_contents, image_contents);
	}

	pub fn run(&mut self, clocks: u32) {
		for i in 0..clocks {
			self.cpu.tick();
		}
	}
	
	pub fn get_output(&mut self) -> u8 {
		self.cpu.get_output()
	}

	pub fn put_input(&mut self, data: u8) {
		self.cpu.put_input(data);
	}
}