use terminal::Terminal;

pub struct WasmTerminal {
	input_data: Vec<u8>,
	output_data: Vec<u8>
}

impl WasmTerminal {
	pub fn new() -> Self {
		WasmTerminal {
			input_data: vec![],
			output_data: vec![]
		}
	}
}

impl Terminal for WasmTerminal {
	fn put_byte(&mut self, value: u8) {
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