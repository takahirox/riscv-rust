extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

mod application;
mod cpu;
mod mmu;
mod plic;
mod clint;
mod uart;
mod virtio_block_disk;
mod terminal;
mod wasm_terminal;

use wasm_terminal::WasmTerminal;
use application::Application;

#[wasm_bindgen]
pub struct WasmRiscv {
	application: Application
}

#[wasm_bindgen]
impl WasmRiscv {
	pub fn new() -> Self {
		WasmRiscv {
			application: Application::new(Box::new(WasmTerminal::new()))
		}
	}
	
	pub fn init(&mut self, kernel_contents: Vec<u8>, fs_contents: Vec<u8>, dtb_contents: Vec<u8>) {
		self.application.setup_from_elf(kernel_contents);
		self.application.setup_filesystem(fs_contents);
		self.application.setup_dtb(dtb_contents);
	}

	pub fn run(&mut self) {
		self.application.run();
	}

	pub fn run_cycles(&mut self, cycles: u32) {
		for _i in 0..cycles {
			self.application.tick();
		}
	}
	
	pub fn get_output(&mut self) -> u8 {
		self.application.get_output()
	}

	pub fn put_input(&mut self, data: u8) {
		self.application.put_input(data);
	}
}