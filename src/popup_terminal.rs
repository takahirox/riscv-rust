extern crate pancurses;

use terminal::Terminal;
use std::str;
use self::pancurses::*;

pub struct PopupTerminal {
	window: Window
}

impl PopupTerminal {
	pub fn new() -> Self {
		let window = initscr();
		window.scrollok(true);
		window.keypad(true);
		window.nodelay(true);
		noecho();
		curs_set(0);
		PopupTerminal {
			window: window
		}
	}
}
	
impl Terminal for PopupTerminal {
	fn put_byte(&mut self, value: u8) {
		let str = vec![value];
		self.window.printw(str::from_utf8(&str).unwrap());
		self.window.refresh();
	}
	
	fn get_input(&mut self) -> u8 {
		match self.window.getch() {
			Some(Input::Character(c)) => {
				c as u8
			},
			_ => 0
		}
	}

	// Wasm specific methods. No use.
	
	fn put_input(&mut self, _value: u8) {
	}
	
	fn get_output(&mut self) -> u8 {
		0 // dummy
	}
}