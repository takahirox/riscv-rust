use terminal::Terminal;

pub struct Uart {
	clock: u64,
	receive_register: u8,
	line_status_register: u8,
	interrupting: bool,
	terminal: Box<dyn Terminal>
}

impl Uart {
	pub fn new(terminal: Box<dyn Terminal>) -> Self {
		Uart {
			clock: 0,
			receive_register: 0,
			line_status_register: 0x20,
			interrupting: false,
			terminal: terminal
		}
	}

	pub fn tick(&mut self) {
		self.clock = self.clock.wrapping_add(1);
		if (self.clock % 0x10000) == 0 && !self.interrupting {
			let value = self.terminal.get_input();
			if value != 0 {
				self.interrupting = true;
				self.receive_register = value;
				self.line_status_register = 1;
			}
		}
	}

	pub fn is_interrupting(&self) -> bool {
		self.interrupting
	}

	pub fn reset_interrupting(&mut self) {
		self.interrupting = false;
	}

	pub fn load(&mut self, address: u64) -> u8 {
		match address {
			0x10000000 => {
				let value = self.receive_register;
				self.receive_register = 0x0;
				self.line_status_register = 0x20;
				value
			},
			0x10000005 => self.line_status_register, // UART0 LSR
			_ => 0
		}
	}

	pub fn store(&mut self, address: u64, value: u8) {
		match address {
			0x10000000 => { // UART0 THR
				self.terminal.put_byte(value);
			},
			_ => {}
		};
	}

	// Wasm specific

	pub fn get_output(&mut self) -> u8 {
		self.terminal.get_output()
	}

	pub fn put_output(&mut self, data: u8) {
		self.terminal.put_byte(data);
	}

	pub fn put_input(&mut self, data: u8) {
		self.terminal.put_input(data);
	}
}
