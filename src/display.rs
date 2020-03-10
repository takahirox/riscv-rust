pub trait Display {
	fn put_byte(&mut self, value: u8);
	fn get_input(&mut self) -> u8;
	// Wasm specific
	fn get_output(&mut self) -> u8;
	fn put_input(&mut self, data: u8);
}
