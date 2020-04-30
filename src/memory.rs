pub const DRAM_BASE: u64 = 0x80000000;

pub struct Memory {
	data: Vec<u8>
}

impl Memory {
	pub fn new() -> Self {
		Memory {
			data: vec![]
		}
	}

	pub fn init(&mut self, capacity: u64) {
		for _i in 0..capacity {
			self.data.push(0);
		}
	}
	
	pub fn read_byte(&self, address: u64) -> u8 {
		if address < DRAM_BASE {
			panic!("Wrong DRAM memory address {:X}", address);
		}
		self.data[(address - DRAM_BASE) as usize]
	}

	pub fn read_bytes(&self, address: u64, width: u64) -> u64 {
		let mut data = 0 as u64;
		for i in 0..width {
			data |= (self.read_byte(address.wrapping_add(i)) as u64) << (i * 8);
		}
		data
	}

	pub fn write_byte(&mut self, address: u64, value: u8) {
		if address < DRAM_BASE {
			panic!("Wrong DRAM memory address {:X}", address);
		}
		self.data[(address - DRAM_BASE) as usize] = value;
	}

	pub fn write_bytes(&mut self, address: u64, value: u64, width: u64) {
		for i in 0..width {
			self.write_byte(address.wrapping_add(i), (value >> (i * 8)) as u8);
		}
	}
}