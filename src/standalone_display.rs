extern crate pancurses;

use display::Display;
use std::str;
use self::pancurses::*;

pub struct StandaloneDisplay {
	window: Window
}

impl StandaloneDisplay {
	pub fn new() -> Self {
		let window = initscr();
		window.scrollok(true);
		window.keypad(true);
		window.nodelay(true);
		noecho();
		curs_set(0);
		StandaloneDisplay {
			window: window
		}
	}
}
	
impl Display for StandaloneDisplay {
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

	
	fn put_input(&mut self, value: u8) {
	}
	
	fn get_output(&mut self) -> u8 {
		0 // dummy
	}
}