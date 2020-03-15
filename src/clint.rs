pub struct Clint {
	clock: u64,
	period_clock: u64,
	interrupting: bool
}

impl Clint {
	pub fn new() -> Self {
		Clint {
			clock: 0,
			period_clock: 0,
			interrupting: false
		}
	}

	pub fn tick(&mut self) {
		// @TODO: Implement more properly
		if self.period_clock > 0 && (self.clock % self.period_clock) == 0 {
			self.interrupting = true;
		}
		self.clock = self.clock.wrapping_add(1);
	}

	pub fn load(&self, _address: u64) -> u8 {
		0 // @TODO: Implement properly
	}

	pub fn store(&mut self, address: u64, value: u8) {
		match address {
			0x02004000 => {
				self.period_clock = (self.period_clock & !0xff) | (value as u64);
			},
			0x02004001 => {
				self.period_clock = (self.period_clock & !0xff00) | ((value as u64) << 8);
			},
			0x02004002 => {
				self.period_clock = (self.period_clock & !0xff0000) | ((value as u64) << 16);
			},
			0x02004003 => {
				self.period_clock = (self.period_clock & !0xff000000) | ((value as u64) << 24);
			},
			_ => {}
		};
	}

	pub fn is_interrupting(&self) -> bool {
		self.interrupting
	}

	pub fn reset_interrupting(&mut self) {
		self.interrupting = false;
	}
}
