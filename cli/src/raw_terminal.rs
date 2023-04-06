use std::str;
use std::io::{stdin, stdout, Read, Write};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;

use riscv_emu_rust::terminal::Terminal;

/// Raw `Terminal`. Output will be displayed in command line
/// and input will be read character-by-character from stdin
/// in a separate thread.
pub struct RawTerminal {
    rx_input: Receiver<u8>,
}

impl RawTerminal {
	pub fn new() -> Self {
		let (tx_input, rx_input) = mpsc::channel::<u8>();
		thread::spawn(move || loop {
		let mut buf = [0; 1];
		if let Ok(n) = stdin().read(&mut buf) {
		if n > 1 {
			panic!("Read {} characters into a 1 byte buffer", n);
		}
		if n == 1 {
			tx_input.send(buf[0]).unwrap();
		}
		// Nothing needs to be sent for n == 0
		}
	});
		RawTerminal {
			rx_input,
		}
	}
}

impl Terminal for RawTerminal {
	fn put_byte(&mut self, value: u8) {
		let str = vec![value];
		match str::from_utf8(&str) {
			Ok(s) => {
				print!("{}", s);
			},
			Err(_e) => {}
		};
		match stdout().flush() {
			_ => {} // Ignoring error so far
		};
	}

	fn get_input(&mut self) -> u8 {
		match self.rx_input.try_recv() {
			Ok(c) => return c,
			_ => return 0,
		}
	}

	// Wasm specific methods. No use.

	fn put_input(&mut self, _value: u8) {
	}

	fn get_output(&mut self) -> u8 {
		0
	}
}
