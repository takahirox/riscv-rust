use terminal::Terminal;

pub struct WasmTerminal {
	input_data: Vec<u8>,
	output_data: Vec<u8>,
	in_escape_sequence: bool
}

impl WasmTerminal {
	pub fn new() -> Self {
		WasmTerminal {
			input_data: vec![],
			output_data: vec![],
			in_escape_sequence: false
		}
	}
}

impl Terminal for WasmTerminal {
	fn put_byte(&mut self, value: u8) {
		if !self.in_escape_sequence {
			if value == 0x1b {
				self.in_escape_sequence = true;
			}
		}
		if self.in_escape_sequence {
			if value == 0x6d {
				self.in_escape_sequence = false;
			}
			return;
		}
		self.output_data.push(value);
	}
	
	fn get_input(&mut self) -> u8 {
		match self.input_data.len() > 0 {
			true => self.input_data.remove(0),
			false => 0
		}
	}
	
	fn put_input(&mut self, value: u8) {
		self.input_data.push(value);
	}
	
	fn get_output(&mut self) -> u8 {
		match self.output_data.len() > 0 {
			true => self.output_data.remove(0),
			false => 0
		}
	}
}