use memory::Memory;
use cpu::{PrivilegeMode, Trap, TrapType, Xlen};
use device::virtio_block_disk::VirtioBlockDisk;
use device::plic::Plic;
use device::clint::Clint;
use device::uart::Uart;
use terminal::Terminal;

pub struct Mmu {
	clock: u64,
	xlen: Xlen,
	ppn: u64,
	addressing_mode: AddressingMode,
	privilege_mode: PrivilegeMode,
	memory: Memory,
	dtb: Vec<u8>,
	disk: VirtioBlockDisk,
	plic: Plic,
	clint: Clint,
	uart: Uart
}

pub enum AddressingMode {
	None,
	SV32,
	SV39,
	SV48 // @TODO: Implement
}

enum MemoryAccessType {
	Execute,
	Read,
	Write,
	DontCare
}

fn _get_addressing_mode_name(mode: &AddressingMode) -> &'static str {
	match mode {
		AddressingMode::None => "None",
		AddressingMode::SV32 => "SV32",
		AddressingMode::SV39 => "SV39",
		AddressingMode::SV48 => "SV48"
	}
}

impl Mmu {
	pub fn new(xlen: Xlen, terminal: Box<dyn Terminal>) -> Self {
		Mmu {
			clock: 0,
			xlen: xlen,
			ppn: 0,
			addressing_mode: AddressingMode::None,
			privilege_mode: PrivilegeMode::Machine,
			memory: Memory::new(),
			dtb: vec![],
			disk: VirtioBlockDisk::new(),
			plic: Plic::new(),
			clint: Clint::new(),
			uart: Uart::new(terminal)
		}
	}

	pub fn update_xlen(&mut self, xlen: Xlen) {
		self.xlen = xlen;
	}

	pub fn init_memory(&mut self, capacity: u64) {
		self.memory.init(capacity);
	}
	
	pub fn init_disk(&mut self, data: Vec<u8>) {
		self.disk.init(data);
	}

	pub fn init_dtb(&mut self, data: Vec<u8>) {
		println!("DTB SIZE:{:X}", data.len());
		for i in 0..data.len() {
			self.dtb.push(data[i]);
		}
	}

	pub fn tick(&mut self, mip: &mut u64) {
		self.clint.tick(mip);
		self.disk.tick(&mut self.memory);
		self.uart.tick();
		self.plic.tick(self.disk.is_interrupting(), self.uart.is_interrupting(), mip);
		self.clock = self.clock.wrapping_add(1);
	}

	pub fn update_addressing_mode(&mut self, new_addressing_mode: AddressingMode) {
		self.addressing_mode = new_addressing_mode;
	}

	pub fn update_privilege_mode(&mut self, mode: PrivilegeMode) {
		self.privilege_mode = mode;
	}

	pub fn update_ppn(&mut self, ppn: u64) {
		self.ppn = ppn;
	}

	fn get_effective_address(&self, address: u64) -> u64 {
		match self.xlen {
			Xlen::Bit32 => address & 0xffffffff,
			Xlen::Bit64 => address
		}
	}

	pub fn fetch(&mut self, v_address: u64) -> Result<u8, Trap> {
		let effective_address = self.get_effective_address(v_address);
		let p_address = match self.translate_address(effective_address, MemoryAccessType::Execute) {
			Ok(address) => address,
			Err(()) => return Err(Trap {
				trap_type: TrapType::InstructionPageFault,
				value: v_address
			})
		};
		Ok(self.load_raw(p_address))
	}

	fn fetch_bytes(&mut self, v_address: u64, width: u64) -> Result<u64, Trap> {
		let mut data = 0 as u64;
		match (v_address & 0xfff) <= (0x1000 - width) {
			true => {
				let effective_address = self.get_effective_address(v_address);
				let p_address = match self.translate_address(effective_address, MemoryAccessType::Execute) {
					Ok(address) => address,
					Err(()) => return Err(Trap {
						trap_type: TrapType::InstructionPageFault,
						value: v_address
					})
				};
				for i in 0..width {
					data |= (self.load_raw(p_address.wrapping_add(i) as u64) as u64) << (i * 8);
				}
			},
			false => {
				for i in 0..width {
					match self.fetch(v_address.wrapping_add(i)) {
						Ok(byte) => {
							data |= (byte as u64) << (i * 8)
						},
						Err(e) => return Err(e)
					};
				}
			}
		}
		Ok(data)
	}

	pub fn fetch_word(&mut self, v_address: u64) -> Result<u32, Trap> {
		match self.fetch_bytes(v_address, 4) {
			Ok(data) => Ok(data as u32),
			Err(e) => Err(e)
		}
	}

	pub fn load(&mut self, v_address: u64) -> Result<u8, Trap> {
		let effective_address = self.get_effective_address(v_address);
		let p_address = match self.translate_address(effective_address, MemoryAccessType::Read) {
			Ok(address) => address,
			Err(()) => return Err(Trap {
				trap_type: TrapType::LoadPageFault,
				value: v_address
			})
		};
		Ok(self.load_raw(p_address))
	}

	fn load_bytes(&mut self, v_address: u64, width: u64) -> Result<u64, Trap> {
		let mut data = 0 as u64;
		match (v_address & 0xfff) <= (0x1000 - width) {
			true => {
				let effective_address = self.get_effective_address(v_address);
				let p_address = match self.translate_address(effective_address, MemoryAccessType::Read) {
					Ok(address) => address,
					Err(()) => return Err(Trap {
						trap_type: TrapType::LoadPageFault,
						value: v_address
					})
				};
				for i in 0..width {
					data |= (self.load_raw(p_address.wrapping_add(i) as u64) as u64) << (i * 8);
				}
			},
			false => {
				for i in 0..width {
					match self.load(v_address.wrapping_add(i)) {
						Ok(byte) => {
							data |= (byte as u64) << (i * 8)
						},
						Err(e) => return Err(e)
					};
				}
			}
		}
		Ok(data)
	}

	pub fn load_halfword(&mut self, v_address: u64) -> Result<u16, Trap> {
		match self.load_bytes(v_address, 2) {
			Ok(data) => Ok(data as u16),
			Err(e) => Err(e)
		}
	}

	pub fn load_word(&mut self, v_address: u64) -> Result<u32, Trap> {
		match self.load_bytes(v_address, 4) {
			Ok(data) => Ok(data as u32),
			Err(e) => Err(e)
		}
	}

	pub fn load_doubleword(&mut self, v_address: u64) -> Result<u64, Trap> {
		match self.load_bytes(v_address, 8) {
			Ok(data) => Ok(data as u64),
			Err(e) => Err(e)
		}
	}

	pub fn store(&mut self, v_address: u64, value: u8) -> Result<(), Trap> {
		let effective_address = self.get_effective_address(v_address);
		let p_address = match self.translate_address(effective_address, MemoryAccessType::Write) {
			Ok(address) => address,
			Err(()) => return Err(Trap {
				trap_type: TrapType::StorePageFault,
				value: v_address
			})
		};
		self.store_raw(p_address, value);
		Ok(())
	}

	fn store_bytes(&mut self, v_address: u64, value: u64, width: u64) -> Result<(), Trap> {
		match (v_address & 0xfff) <= (0x1000 - width) {
			true => {
				let effective_address = self.get_effective_address(v_address);
				let p_address = match self.translate_address(effective_address, MemoryAccessType::Write) {
					Ok(address) => address,
					Err(()) => return Err(Trap {
						trap_type: TrapType::StorePageFault,
						value: v_address
					})
				};
				for i in 0..width {
					self.store_raw(p_address.wrapping_add(i), ((value >> (i * 8)) & 0xff) as u8);
				}
			},
			false => {
				for i in 0..width {
					match self.store(v_address.wrapping_add(i), ((value >> (i * 8)) & 0xff) as u8) {
						Ok(()) => {},
						Err(e) => return Err(e)
					}
				}
			}
		}
		Ok(())
	}

	pub fn store_halfword(&mut self, v_address: u64, value: u16) -> Result<(), Trap> {
		self.store_bytes(v_address, value as u64, 2)
	}

	pub fn store_word(&mut self, v_address: u64, value: u32) -> Result<(), Trap> {
		self.store_bytes(v_address, value as u64, 4)
	}

	pub fn store_doubleword(&mut self, v_address: u64, value: u64) -> Result<(), Trap> {
		self.store_bytes(v_address, value as u64, 8)
	}

	pub fn load_raw(&mut self, address: u64) -> u8 {
		let effective_address = self.get_effective_address(address);
		// @TODO: Mapping should be configurable with dtb
		match address {
			// I don't know why but dtb data seems to be stored from 0x1020 on Linux.
			// It might be from self.x[0xb] initialization?
			0x00001020..=0x00001ea2 => self.dtb[address as usize - 0x1020],
			0x02000000..=0x0200ffff => self.clint.load(effective_address),
			0x0C000000..=0x0fffffff => self.plic.load(effective_address),
			0x10000000..=0x100000ff => self.uart.load(effective_address),
			0x10001000..=0x10001FFF => self.disk.load(effective_address),
			_ => {
				self.memory.read_byte(effective_address)
			}
		}
	}

	pub fn load_word_raw(&mut self, address: u64) -> u32 {
		let mut data = 0 as u32;
		for i in 0..4 {
			data |= (self.load_raw(address.wrapping_add(i)) as u32) << (i * 8)
		}
		data
	}

	pub fn load_doubleword_raw(&mut self, address: u64) -> u64 {
		let mut data = 0 as u64;
		for i in 0..8 {
			data |= (self.load_raw(address.wrapping_add(i)) as u64) << (i * 8)
		}
		data
	}

	pub fn store_raw(&mut self, address: u64, value: u8) {
		let effective_address = self.get_effective_address(address);
		// @TODO: Mapping should be configurable with dtb
		match address {
			0x02000000..=0x0200ffff => self.clint.store(effective_address, value),
			0x0c000000..=0x0fffffff => self.plic.store(effective_address, value),
			0x10000000..=0x100000ff => self.uart.store(effective_address, value),
			0x10001000..=0x10001FFF => self.disk.store(effective_address, value),
			_ => self.memory.write_byte(effective_address, value)
		};
	}

	pub fn store_word_raw(&mut self, address: u64, value: u32) {
		for i in 0..4 {
			self.store_raw(address.wrapping_add(i), ((value >> (i * 8)) & 0xff) as u8);
		}
	}

	pub fn store_doubleword_raw(&mut self, address: u64, value: u64) {
		for i in 0..8 {
			self.store_raw(address.wrapping_add(i), ((value >> (i * 8)) & 0xff) as u8);
		}
	}

	pub fn validate_address(&mut self, v_address: u64) -> bool {
		// @TODO: Support other access types?
		let p_address = match self.translate_address(v_address, MemoryAccessType::DontCare) {
			Ok(address) => address,
			Err(()) => return false
		};
		let effective_address = self.get_effective_address(p_address);
		match effective_address {
			0x00001020..=0x00001fff => true,
			0x02000000..=0x0200ffff => true,
			0x0C000000..=0x0fffffff => true,
			0x10000000..=0x100000ff => true,
			0x10001000..=0x10001FFF => true,
			_ => self.memory.validate_address(effective_address)
		}
	}

	fn translate_address(&mut self, address: u64, access_type: MemoryAccessType) -> Result<u64, ()> {
		match self.addressing_mode {
			AddressingMode::None => Ok(address),
			AddressingMode::SV32 => match self.privilege_mode {
				PrivilegeMode::User | PrivilegeMode::Supervisor => {
					let vpns = [(address >> 12) & 0x3ff, (address >> 22) & 0x3ff];
					self.traverse_page(address, 2 - 1, self.ppn, &vpns, access_type)
				},
				_ => Ok(address)
			},
			AddressingMode::SV39 => match self.privilege_mode {
				PrivilegeMode::User | PrivilegeMode::Supervisor => {
					let vpns = [(address >> 12) & 0x1ff, (address >> 21) & 0x1ff, (address >> 30) & 0x1ff];
					self.traverse_page(address, 3 - 1, self.ppn, &vpns, access_type)
				},
				_ => Ok(address)
			},
			AddressingMode::SV48 => {
				panic!("AddressingMode SV48 is not supported yet.");
			}
		}
	}

	fn traverse_page(&mut self, v_address: u64, level: u8, parent_ppn: u64,
		vpns: &[u64], access_type: MemoryAccessType) -> Result<u64, ()> {
		let pagesize = 4096;
		let ptesize = match self.addressing_mode {
			AddressingMode::SV32 => 4,
			_ => 8
		};
		let pte_address = parent_ppn * pagesize + vpns[level as usize] * ptesize;
		let pte = match self.addressing_mode {
			AddressingMode::SV32 => self.load_word_raw(pte_address) as u64,
			_ => self.load_doubleword_raw(pte_address)
		};
		let ppn = match self.addressing_mode {
			AddressingMode::SV32 => (pte >> 10) & 0x3fffff,
			_ => (pte >> 10) & 0xfffffffffff
		};
		let ppns = match self.addressing_mode {
			AddressingMode::SV32 => [(pte >> 10) & 0x3ff, (pte >> 20) & 0xfff, 0 /*dummy*/],
			AddressingMode::SV39 => [(pte >> 10) & 0x1ff, (pte >> 19) & 0x1ff, (pte >> 28) & 0x3ffffff],
			_ => panic!() // Shouldn't happen
		};
		let _rsw = (pte >> 8) & 0x3;
		let d = (pte >> 7) & 1;
		let a = (pte >> 6) & 1;
		let _g = (pte >> 5) & 1;
		let _u = (pte >> 4) & 1;
		let x = (pte >> 3) & 1;
		let w = (pte >> 2) & 1;
		let r = (pte >> 1) & 1;
		let v = pte & 1;

		// println!("VA:{:X} Level:{:X} PTE_AD:{:X} PTE:{:X} PPPN:{:X} PPN:{:X} PPN1:{:X} PPN0:{:X}", v_address, level, pte_address, pte, parent_ppn, ppn, ppns[1], ppns[0]);

		if v == 0 || (r == 0 && w == 1) {
			return Err(());
		}

		if r == 0 && x == 0 {
			return match level {
				0 => Err(()),
				_ => self.traverse_page(v_address, level - 1, ppn, vpns, access_type)
			};
		}

		// Leaf page found

		if a == 0 || (match access_type { MemoryAccessType::Write => d == 0, _ => false }) {
			let new_pte = pte | (1 << 6) | (match access_type {
				MemoryAccessType::Write => 1 << 7,
				_ => 0
			});
			match self.addressing_mode {
				AddressingMode::SV32 => self.store_word_raw(pte_address, new_pte as u32),
				_ => self.store_doubleword_raw(pte_address, new_pte)
			};
		}

		match access_type {
			MemoryAccessType::Execute => {
				if x == 0 {
					return Err(());
				}
			},
			MemoryAccessType::Read => {
				if r == 0 {
					return Err(());
				}
			},
			MemoryAccessType::Write => {
				if w == 0 {
					return Err(());
				}
			},
			_ => {}
		};

		let offset = v_address & 0xfff; // [11:0]
		// @TODO: Optimize
		let p_address = match self.addressing_mode {
			AddressingMode::SV32 => match level {
				1 => {
					if ppns[0] != 0 {
						return Err(());
					}
					(ppns[1] << 22) | (vpns[0] << 12) | offset
				},
				0 => (ppn << 12) | offset,
				_ => panic!() // Shouldn't happen
			},
			_ => match level {
				2 => {
					if ppns[1] != 0 || ppns[0] != 0 {
						return Err(());
					}
					(ppns[2] << 30) | (vpns[1] << 21) | (vpns[0] << 12) | offset
				},
				1 => {
					if ppns[0] != 0 {
						return Err(());
					}
					(ppns[2] << 30) | (ppns[1] << 21) | (vpns[0] << 12) | offset
				},
				0 => (ppn << 12) | offset,
				_ => panic!() // Shouldn't happen
			},
		};
		// println!("PA:{:X}", p_address);
		Ok(p_address)
	}

	pub fn get_clint(&self) -> &Clint {
		&self.clint
	}

	pub fn get_mut_clint(&mut self) -> &mut Clint {
		&mut self.clint
	}

	pub fn get_mut_uart(&mut self) -> &mut Uart {
		&mut self.uart
	}
}