use cpu::{MIP_MSIP, MIP_MTIP};

pub struct Clint {
	clock: u64,
	msip: u32,
	mtimecmp: u64,
	mtime: u64
}

impl Clint {
	pub fn new() -> Self {
		Clint {
			clock: 0,
			msip: 0,
			mtimecmp: 0,
			mtime: 0 // @TODO: Should be bound to csr time register
		}
	}

	pub fn tick(&mut self, mip: &mut u64) {
		self.clock = self.clock.wrapping_add(1);

		// core clock : mtime clock = 8 : 1 is just an arbiraty ratio.
		// @TODO: Implement more properly
		if (self.clock % 8) == 0 {
			self.mtime = self.mtime.wrapping_add(1);
		}

		if (self.msip & 1) != 0 {
			*mip |= MIP_MSIP;
		}

		// I'm not sure why but if clock interrupt happens while Linux boot
		// virtio block device access fails. So disable the clock interrupt
		// until likely Linux boots up as workaround.
		// @TODO: Figure out the root issue and fix.
		if self.mtime > 0x1000000 && self.mtimecmp > 0 && self.mtime > self.mtimecmp {
			*mip |= MIP_MTIP;
		} else {
			*mip &= !MIP_MTIP;
		}
	}

	pub fn load(&self, address: u64) -> u8 {
		//println!("CLINT Load AD:{:X}", address);
		match address {
			// MSIP register 4 bytes
			0x02000000 => {
				(self.msip & 0xff) as u8
			},
			0x02000001 => {
				((self.msip >> 8) & 0xff) as u8
			},
			0x02000002 => {
				((self.msip >> 16) & 0xff) as u8
			},
			0x02000003 => {
				((self.msip >> 24) & 0xff) as u8
			},
			// MTIMECMP Registers 8 bytes
			0x02004000 => {
				self.mtimecmp as u8
			},
			0x02004001 => {
				(self.mtimecmp >> 8) as u8
			},
			0x02004002 => {
				(self.mtimecmp >> 16) as u8
			},
			0x02004003 => {
				(self.mtimecmp >> 24) as u8
			},
			0x02004004 => {
				(self.mtimecmp >> 32) as u8
			},
			0x02004005 => {
				(self.mtimecmp >> 40) as u8
			},
			0x02004006 => {
				(self.mtimecmp >> 48) as u8
			},
			0x02004007 => {
				(self.mtimecmp >> 56) as u8
			},
			0x0200bff8 => {
				self.mtime as u8
			},
			0x0200bff9 => {
				(self.mtime >> 8) as u8
			},
			0x0200bffa => {
				(self.mtime >> 16) as u8
			},
			0x0200bffb => {
				(self.mtime >> 24) as u8
			},
			0x0200bffc => {
				(self.mtime >> 32) as u8
			},
			0x0200bffd => {
				(self.mtime >> 40) as u8
			},
			0x0200bffe => {
				(self.mtime >> 48) as u8
			},
			0x0200bfff => {
				(self.mtime >> 56) as u8
			},
			_ => 0,
		}
	}

	pub fn store(&mut self, address: u64, value: u8) {
		//println!("CLINT Store AD:{:X} VAL:{:X}", address, value);
		match address {
			// MSIP register 4 bytes
			0x02000000 => {
				self.msip = (self.msip & !0xff) | (value as u32);
			},
			0x02000001 => {
				self.msip = (self.msip & !(0xff << 8)) | ((value as u32) << 8);
			},
			0x02000002 => {
				self.msip = (self.msip & !(0xff << 16)) | ((value as u32) << 16);
			},
			0x02000003 => {
				self.msip = (self.msip & !(0xff << 24)) | ((value as u32) << 24);
			},
			// MTIMECMP Registers 8 bytes
			0x02004000 => {
				self.mtimecmp = (self.mtimecmp & !0xff) | (value as u64);
			},
			0x02004001 => {
				self.mtimecmp = (self.mtimecmp & !(0xff << 8)) | ((value as u64) << 8);
			},
			0x02004002 => {
				self.mtimecmp = (self.mtimecmp & !(0xff << 16)) | ((value as u64) << 16);
			},
			0x02004003 => {
				self.mtimecmp = (self.mtimecmp & !(0xff << 24)) | ((value as u64) << 24);
			},
			0x02004004 => {
				self.mtimecmp = (self.mtimecmp & !(0xff << 32)) | ((value as u64) << 32);
			},
			0x02004005 => {
				self.mtimecmp = (self.mtimecmp & !(0xff << 40)) | ((value as u64) << 40);
			},
			0x02004006 => {
				self.mtimecmp = (self.mtimecmp & !(0xff << 48)) | ((value as u64) << 48);
			},
			0x02004007 => {
				self.mtimecmp = (self.mtimecmp & !(0xff << 56)) | ((value as u64) << 56);
			},
			// MTIME registers 8 bytes
			0x0200bff8 => {
				self.mtime = (self.mtime & !0xff) | (value as u64);
			},
			0x0200bff9 => {
				self.mtime = (self.mtime & !(0xff << 8)) | ((value as u64) << 8);
			},
			0x0200bffa => {
				self.mtime = (self.mtime & !(0xff << 16)) | ((value as u64) << 16);
			},
			0x0200bffb => {
				self.mtime = (self.mtime & !(0xff << 24)) | ((value as u64) << 24);
			},
			0x0200bffc => {
				self.mtime = (self.mtime & !(0xff << 32)) | ((value as u64) << 32);
			},
			0x0200bffd => {
				self.mtime = (self.mtime & !(0xff << 40)) | ((value as u64) << 40);
			},
			0x0200bffe => {
				self.mtime = (self.mtime & !(0xff << 48)) | ((value as u64) << 48);
			},
			0x0200bfff => {
				self.mtime = (self.mtime & !(0xff << 56)) | ((value as u64) << 56);
			},
			_ => {}
		};
	}
}
