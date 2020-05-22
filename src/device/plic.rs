use cpu::MIP_SEIP;

// Based on SiFive Interrupt Cookbook
// https://sifive.cdn.prismic.io/sifive/0d163928-2128-42be-a75a-464df65e04e0_sifive-interrupt-cookbook.pdf

/// Emulates PLIC known as Interrupt Controller.
/// Refer to the [specification](https://sifive.cdn.prismic.io/sifive%2Fc89f6e5a-cf9e-44c3-a3db-04420702dcc1_sifive+e31+manual+v19.08.pdf)
/// for the detail.
pub struct Plic {
	clock: u64,
	irq: u32,
	enabled: u64,
	threshold: u32,
	priorities: [u32; 1024]
}

impl Plic {
	/// Creates a new `Plic`.
	pub fn new() -> Self {
		Plic {
			clock: 0,
			irq: 0,
			enabled: 0,
			threshold: 0,
			priorities: [0; 1024]
		}
	}

	/// Runs one cycle. Takes interrupting signals from devices and
	/// raises an interrupt to CPU depending on configuration.
	/// It interrupt occurs CPU a certain bit of `mip` regiser is risen
	/// depending on interrupt type.
	///
	/// # Arguments
	/// * `virtio_is_interrupting`
	/// * `uart_is_interrupting`
	/// * `mip`
	pub fn tick(&mut self, virtio_is_interrupting: bool,
		uart_is_interrupting: bool, mip: &mut u64) {
		self.clock = self.clock.wrapping_add(1);

		// @TODO: IRQ num should be configurable with dtb
		let virtio_irq = 1;
		let uart_irq = 10;

		// Which should be prioritized, local timer interrupt or global interrupts?

		let virtio_priority = self.priorities[virtio_irq as usize];
		let uart_priority = self.priorities[uart_irq as usize];

		let virtio_enabled = ((self.enabled >> virtio_irq) & 1) == 1;
		let uart_enabled = ((self.enabled >> uart_irq) & 1) == 1;

		let interruptings = [virtio_is_interrupting, uart_is_interrupting];
		let enables = [virtio_enabled, uart_enabled];
		let priorities = [virtio_priority, uart_priority];
		let irqs = [virtio_irq, uart_irq];

		let mut irq = 0;
		let mut priority = 0;
		for i in 0..2 {
			if interruptings[i] && enables[i] &&
				priorities[i] > self.threshold &&
				priorities[i] > priority {
					irq = irqs[i];
					priority = priorities[i];
			}
		}

		if irq != 0 {
			self.irq = irq;
			//println!("IRQ: {:X}", self.irq);
			*mip |= MIP_SEIP;
		}
	}

	/// Loads register content
	///
	/// # Arguments
	/// * `address`
	pub fn load(&self, address: u64) -> u8 {
		//println!("PLIC Load AD:{:X}", address);
		match address {
			0x0c000000..=0x0c000fff => {
				let offset = address % 4;
				let index = ((address - 0xc000000) >> 2) as usize;
				let pos = offset << 3;
				(self.priorities[index] >> pos) as u8
			},
			0x0c002080 => self.enabled as u8,
			0x0c002081 => (self.enabled >> 8) as u8,
			0x0c002082 => (self.enabled >> 16) as u8,
			0x0c002083 => (self.enabled >> 24) as u8,
			0x0c002084 => (self.enabled >> 32) as u8,
			0x0c002085 => (self.enabled >> 40) as u8,
			0x0c002086 => (self.enabled >> 48) as u8,
			0x0c002087 => (self.enabled >> 56) as u8,
			0x0c201000 => self.threshold as u8,
			0x0c201001 => (self.threshold >> 8) as u8,
			0x0c201002 => (self.threshold >> 16) as u8,
			0x0c201003 => (self.threshold >> 24) as u8,
			0x0c201004 => self.irq as u8,
			0x0c201005 => (self.irq >> 8) as u8,
			0x0c201006 => (self.irq >> 16) as u8,
			0x0c201007 => (self.irq >> 24) as u8,
			_ => 0
		}
	}

	/// Stores register content
	///
	/// # Arguments
	/// * `address`
	/// * `value`
	pub fn store(&mut self, address: u64, value: u8) {
		//println!("PLIC Store AD:{:X} VAL:{:X}", address, value);
		match address {
			0x0c000000..=0x0c000fff => {
				let offset = address % 4;
				let index = ((address - 0xc000000) >> 2) as usize;
				let pos = offset << 3;
				self.priorities[index] = (self.priorities[index] & !(0xff << pos)) | ((value as u32) << pos);
			},
			0x0c002080 => {
				self.enabled = (self.enabled & !0xff) | (value as u64);
			},
			0x0c002081 => {
				self.enabled = (self.enabled & !(0xff << 8)) | ((value as u64) << 8);
			},
			0x0c002082 => {
				self.enabled = (self.enabled & !(0xff << 16)) | ((value as u64) << 16);
			},
			0x0c002083 => {
				self.enabled = (self.enabled & !(0xff << 24)) | ((value as u64) << 24);
			},
			0x0c002084 => {
				self.enabled = (self.enabled & !(0xff << 32)) | ((value as u64) << 32);
			},
			0x0c002085 => {
				self.enabled = (self.enabled & !(0xff << 40)) | ((value as u64) << 40);
			},
			0x0c002086 => {
				self.enabled = (self.enabled & !(0xff << 48)) | ((value as u64) << 48);
			},
			0x0c002087 => {
				self.enabled = (self.enabled & !(0xff << 56)) | ((value as u64) << 56);
			},
			0x0c201000 => {
				self.threshold = (self.threshold & !0xff) | (value as u32);
			},
			0x0c201001 => {
				self.threshold = (self.threshold & !(0xff << 8)) | ((value as u32) << 8);
			},
			0x0c201002 => {
				self.threshold = (self.threshold & !(0xff << 16)) | ((value as u32) << 16);
			},
			0x0c201003 => {
				self.threshold = (self.threshold & !(0xff << 24)) | ((value as u32) << 24);
			},
			0x0c201004 => {
				if self.irq as u8 == value {
					self.irq = 0;
				}
			},
			_ => {}
		};
	}
}
