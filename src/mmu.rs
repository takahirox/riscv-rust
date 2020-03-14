use cpu::{PrivilegeMode, Trap, TrapType, Xlen};
use virtio_block_disk::VirtioBlockDisk;
use plic::{InterruptType, Plic};
use clint::Clint;
use uart::Uart;
use terminal::Terminal;

const DRAM_BASE: usize = 0x80000000;

pub struct Mmu {
	clock: u64,
	xlen: Xlen,
	ppn: u64,
	addressing_mode: AddressingMode,
	privilege_mode: PrivilegeMode,
	interrupt: InterruptType,
	memory: Vec<u8>,
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
	Write
}

impl Mmu {
	pub fn new(xlen: Xlen, terminal: Box<dyn Terminal>) -> Self {
		Mmu {
			clock: 0,
			xlen: xlen,
			ppn: 0,
			addressing_mode: AddressingMode::None,
			privilege_mode: PrivilegeMode::Machine,
			interrupt: InterruptType::None,
			memory: vec![],
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
		for _i in 0..capacity {
			self.memory.push(0);
		}
	}
	
	pub fn init_disk(&mut self, data: Vec<u8>) {
		self.disk.init(data);
	}

	pub fn tick(&mut self) {
		self.disk.tick();
		self.plic.tick();
		self.clint.tick();
		self.uart.tick();
		self.clock = self.clock.wrapping_add(1);
	}

	pub fn detect_interrupt(&mut self) -> &InterruptType {
		// @TODO: Implement properly
		match self.interrupt {
			InterruptType::None => {
				let mut interrupt = InterruptType::None;
				if self.is_disk_interrupting() {
					interrupt = InterruptType::Virtio;
				} else if self.is_uart_interrupting() {
					interrupt = InterruptType::KeyInput;
				} else if self.is_clint_interrupting() {
					interrupt = InterruptType::Timer;
				}
				match interrupt {
					InterruptType::None => {},
					_ => {
						self.update_plic(&interrupt);
					}
				};
				self.interrupt = interrupt;
			},
			_ => {}
		};
		&self.interrupt
	}

	pub fn reset_interrupt(&mut self) {
		self.interrupt = InterruptType::None;
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

	pub fn fetch_word(&mut self, v_address: u64) -> Result<u32, Trap> {
		let mut data = 0 as u32;
		for i in 0..4 {
			match self.fetch(v_address.wrapping_add(i)) {
				Ok(byte) => {
					data |= (byte as u32) << (i * 8)
				},
				Err(e) => return Err(e)
			};
		}
		Ok(data)
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

	pub fn load_halfword(&mut self, v_address: u64) -> Result<u16, Trap> {
		let mut data = 0 as u16;
		for i in 0..2 {
			match self.load(v_address.wrapping_add(i)) {
				Ok(byte) => {
					data |= (byte as u16) << (i * 8)
				},
				Err(e) => return Err(e)
			};
		}
		Ok(data)
	}

	pub fn load_word(&mut self, v_address: u64) -> Result<u32, Trap> {
		let mut data = 0 as u32;
		for i in 0..4 {
			match self.load(v_address.wrapping_add(i)) {
				Ok(byte) => {
					data |= (byte as u32) << (i * 8)
				},
				Err(e) => return Err(e)
			};
		}
		Ok(data)
	}

	pub fn load_doubleword(&mut self, v_address: u64) -> Result<u64, Trap> {
		let mut data = 0 as u64;
		for i in 0..8 {
			match self.load(v_address.wrapping_add(i)) {
				Ok(byte) => {
					data |= (byte as u64) << (i * 8)
				},
				Err(e) => return Err(e)
			};
		}
		Ok(data)
	}

	pub fn store(&mut self, v_address: u64, value: u8) -> Result<(), Trap> {
		let effective_address = self.get_effective_address(v_address);
		let p_address = match self.translate_address(effective_address, MemoryAccessType::Write) {
			Ok(address) => address,
			Err(()) => return Err(Trap {
				trap_type: TrapType::LoadPageFault,
				value: v_address
			})
		};
		self.store_raw(p_address, value);
		Ok(())
	}

	pub fn store_halfword(&mut self, v_address: u64, value: u16) -> Result<(), Trap> {
		for i in 0..2 {
			match self.store(v_address.wrapping_add(i), ((value >> (i * 8)) & 0xff) as u8) {
				Ok(()) => {},
				Err(e) => return Err(e)
			}
		}
		Ok(())
	}

	pub fn store_word(&mut self, v_address: u64, value: u32) -> Result<(), Trap> {
		for i in 0..4 {
			match self.store(v_address.wrapping_add(i), ((value >> (i * 8)) & 0xff) as u8) {
				Ok(()) => {},
				Err(e) => return Err(e)
			}
		}
		Ok(())
	}

	pub fn store_doubleword(&mut self, v_address: u64, value: u64) -> Result<(), Trap> {
		for i in 0..8 {
			match self.store(v_address.wrapping_add(i), ((value >> (i * 8)) & 0xff) as u8) {
				Ok(()) => {},
				Err(e) => return Err(e)
			}
		}
		Ok(())
	}

	pub fn load_raw(&mut self, address: u64) -> u8 {
		let effective_address = self.get_effective_address(address);
		// @TODO: Check valid memory map
		match address {
			0x0200bff8..=0x0200bfff => self.clint.load(effective_address) as u8,
			0x0c201004..=0x0c201007 => self.plic.load(effective_address) as u8,
			0x10000000..=0x10000005 => self.uart.load(effective_address),
			0x10001000..=0x10001FFF => self.disk.load(effective_address),
			_ => {
				if effective_address < DRAM_BASE as u64 {
					panic!("No memory map support yet to load AD:{:X}", effective_address);
				}
				self.memory[effective_address as usize - DRAM_BASE]
			}
		}
	}

	pub fn load_halfword_raw(&mut self, address: u64) -> u16 {
		let mut data = 0 as u16;
		for i in 0..2 {
			data |= (self.load_raw(address.wrapping_add(i)) as u16) << (i * 8)
		}
		data
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
		// @TODO: Check memory map
		match address {
			0x0c000004..=0x0c000007 => {}, // @TODO: Where to?
			0x0c000028..=0x0c00002b => {}, // @TODO: Where to?
			0x0c002080..=0x0c002083 => { // PLIC_SENABLE(hart) (PLIC + 0x2080 + (hart)*0x100)
				self.plic.store(effective_address, value);
			},
			0x0c201000..=0x0c201007 => {}, // @TODO: Where to?
			0x02004000..=0x02004007 => {
				self.clint.store(effective_address, value);
			},
			0x10000000..=0x10000005 => {
				self.uart.store(effective_address, value);
			},
			0x10001000..=0x10001FFF => { // @TODO: Check a valid range
				self.disk.store(effective_address, value);
			},
			_ => {
				if effective_address < DRAM_BASE as u64 {
					panic!("No memory map support yet to store AD:{:X}", effective_address);
				}
				self.memory[effective_address as usize - DRAM_BASE] = value;
			}
		};
	}

	pub fn store_halfword_raw(&mut self, address: u64, value: u16) {
		for i in 0..2 {
			self.store_raw(address.wrapping_add(i), ((value >> (i * 8)) & 0xff) as u8);
		}
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
			return Err({});
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
			}
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

	//

	pub fn handle_disk_access(&mut self) {
		let avail_address = self.disk.get_avail_address();
		let base_desc_address = self.disk.get_desc_address() as u64;
		let base_used_address = self.disk.get_used_address();

		let _flag = self.load_halfword_raw(avail_address);
		let offset = self.load_halfword_raw(avail_address.wrapping_add(1));
		let index = self.load_halfword_raw(avail_address.wrapping_add(offset as u64 % 8).wrapping_add(2));
		let desc_size = 16;

		let desc_address0 = base_desc_address + desc_size * index as u64;
		let addr0 = self.load_doubleword_raw(desc_address0);
		let _len0 = self.load_word_raw(desc_address0.wrapping_add(8));
		let _flags0 = self.load_halfword_raw(desc_address0.wrapping_add(12));
		let next0 = self.load_halfword_raw(desc_address0.wrapping_add(14));

		let desc_address1 = base_desc_address + desc_size * next0 as u64;
		let addr1 = self.load_doubleword_raw(desc_address1);
		let len1 = self.load_word_raw(desc_address1.wrapping_add(8));
		let flags1 = self.load_halfword_raw(desc_address1.wrapping_add(12));
		let next1 = self.load_halfword_raw(desc_address1.wrapping_add(14));

		let desc_address2 = base_desc_address + desc_size * next1 as u64;
		let _addr2 = self.load_doubleword_raw(desc_address2);
		let _len2 = self.load_word_raw(desc_address2.wrapping_add(8));
		let _flags2 = self.load_halfword_raw(desc_address2.wrapping_add(12));
		let _next2 = self.load_halfword_raw(desc_address2.wrapping_add(14));

		/*
		println!("Avail AD:{:X}", avail_address);
		println!("Flag:{:X}", flag);
		println!("Offset:{:X}", offset);
		println!("Index:{:X}", index);
		println!("addr0:{:X}", addr0);
		println!("len0:{:X}", len0);
		println!("flags0:{:X}", flags0);
		println!("next0:{:X}", next0);
		println!("addr1:{:X}", addr1);
		println!("len1:{:X}", len1);
		println!("flags1:{:X}", flags1);
		println!("next1:{:X}", next1);
		println!("addr2:{:X}", addr2);
		println!("len2:{:X}", len2);
		println!("flags2:{:X}", flags2);
		println!("next2:{:X}", next2);
		*/
		
		let _blk_type = self.load_word_raw(addr0);
		let _blk_reserved = self.load_word_raw(addr0.wrapping_add(4));
		let blk_sector = self.load_doubleword_raw(addr0.wrapping_add(8));

		/*
		println!("Blk type:{:X}", blk_type);
		println!("Blk reserved:{:X}", blk_reserved);
		println!("Blk sector:{:X}", blk_sector);
		*/

		match (flags1 & 2) == 0 {
			true => { // write to disk
				// println!("Write to disk");
				for i in 0..len1 as u64 {
					let data = self.load_raw(addr1 + i);
					self.disk.write_to_disk(blk_sector * 512 + i, data);
					// print!("{:02X} ", data);
				}
				// println!();
			},
			false => { // read from disk
				// println!("Read from disk");
				for i in 0..len1 as u64 {
					let data = self.disk.read_from_disk(blk_sector * 512 + i);
					self.store_raw(addr1 + i, data);
					// print!("{:02X} ", data);
				}
				// println!();
			}
		};
		
		let new_id = self.disk.get_new_id() as u16;
		self.store_halfword_raw(base_used_address.wrapping_add(2), new_id % 8);
	}

	//

	pub fn is_disk_interrupting(&mut self) -> bool {
		self.disk.is_interrupting()
	}

	pub fn reset_disk_interrupting(&mut self) {
		self.disk.reset_interrupting();
	}

	pub fn is_clint_interrupting(&self) -> bool {
		self.clint.is_interrupting()
	}

	pub fn reset_clint_interrupting(&mut self) {
		self.clint.reset_interrupting();
	}

	pub fn is_uart_interrupting(&mut self) -> bool {
		self.uart.is_interrupting()
	}

	pub fn reset_uart_interrupting(&mut self) {
		self.uart.reset_interrupting();
	}

	pub fn update_plic(&mut self, interrupt_type: &InterruptType) {
		self.plic.update(interrupt_type);
	}

	// Wasm specific
	pub fn get_uart_output(&mut self) -> u8 {
		self.uart.get_output()
	}

	pub fn put_uart_input(&mut self, data: u8) {
		self.uart.put_input(data);
	}
}