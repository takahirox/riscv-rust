pub enum InterruptType {
	None,
	KeyInput,
	Timer,
	Virtio
}

pub struct Plic {
	clock: u64,
	irq: u32,
	enabled: bool
}

impl Plic {
	pub fn new() -> Self {
		Plic {
			clock: 0,
			irq: 0,
			enabled: false
		}
	}

	pub fn tick(&mut self) {
		self.clock = self.clock.wrapping_add(1);
	}

	pub fn update(&mut self, interrupt_type: &InterruptType) {
		match interrupt_type {
			InterruptType::Virtio => {
				self.irq = 1;
			}
			InterruptType::KeyInput => {
				self.irq = 10;
			}
			InterruptType::None |
			InterruptType::Timer => {
				self.irq = 0;
			}
		}
	}

	pub fn store(&mut self, address: u64, _value: u8) {
		match address {
			0x0c002080 => { // PLIC_SENABLE(hart) (PLIC + 0x2080 + (hart)*0x100)
				self.enabled = true;
			},
			_ => {}
		};
	}

	pub fn load(&self, address: u64) -> u32 {
		match address {
			0x0c201004 => self.irq, // PLIC_SCLAIM(hart) (PLIC + 0x201004 + (hart)*0x2000)
			_ => 0
		}
	}
}
