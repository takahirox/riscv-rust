/// DRAM base address. Offset from this base address
/// is the address in main memory.
const DRAM_BASE: u64 = 0x80000000;

const DTB_SIZE: usize = 0xfe0;

extern crate fnv;

use self::fnv::FnvHashMap;

use memory::Memory;
use cpu::{PrivilegeMode, Trap, TrapType, Xlen};
use device::virtio_block_disk::VirtioBlockDisk;
use device::plic::Plic;
use device::clint::Clint;
use device::uart::Uart;
use terminal::Terminal;

/// Emulates Memory Management Unit. It holds the Main memory and peripheral
/// devices, maps address to them, and accesses them depending on address.
/// It also manages virtual-physical address translation and memoty protection.
/// It may also be said Bus.
/// @TODO: Memory protection is not implemented yet. We should support.
pub struct Mmu {
	clock: u64,
	xlen: Xlen,
	ppn: u64,
	addressing_mode: AddressingMode,
	privilege_mode: PrivilegeMode,
	memory: MemoryWrapper,
	dtb: Vec<u8>,
	disk: VirtioBlockDisk,
	plic: Plic,
	clint: Clint,
	uart: Uart,

	/// Address translation page cache. Experimental feature.
	/// The cache is cleared when translation mapping can be changed;
	/// xlen, ppn, privilege_mode, or addressing_mode is updated.
	/// Precisely it isn't good enough because page table entries
	/// can be updated anytime with store instructions, of course
	/// very depending on how pages are mapped tho.
	/// But observing all page table entries is high cost so
	/// ignoring so far. Then this cache optimization can cause a bug
	/// due to unexpected (meaning not in page fault handler)
	/// page table entry update. So this is experimental feature and
	/// disabled by default. If you want to enable, use `enable_page_cache()`.
	page_cache_enabled: bool,
	fetch_page_cache: FnvHashMap<u64, u64>,
	load_page_cache: FnvHashMap<u64, u64>,
	store_page_cache: FnvHashMap<u64, u64>
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
	/// Creates a new `Mmu`.
	///
	/// # Arguments
	/// * `xlen`
	/// * `terminal`
	pub fn new(xlen: Xlen, terminal: Box<dyn Terminal>) -> Self {
		let mut dtb = vec![0; DTB_SIZE];
		load_default_dtb_content(&mut dtb);
		Mmu {
			clock: 0,
			xlen: xlen,
			ppn: 0,
			addressing_mode: AddressingMode::None,
			privilege_mode: PrivilegeMode::Machine,
			memory: MemoryWrapper::new(),
			dtb: dtb,
			disk: VirtioBlockDisk::new(),
			plic: Plic::new(),
			clint: Clint::new(),
			uart: Uart::new(terminal),
			page_cache_enabled: false,
			fetch_page_cache: FnvHashMap::default(),
			load_page_cache: FnvHashMap::default(),
			store_page_cache: FnvHashMap::default()
		}
	}

	/// Updates XLEN, 32-bit or 64-bit
	///
	/// # Arguments
	/// * `xlen`
	pub fn update_xlen(&mut self, xlen: Xlen) {
		self.xlen = xlen;
		self.clear_page_cache();
	}

	/// Initializes Main memory. This method is expected to be called only once.
	///
	/// # Arguments
	/// * `capacity`
	pub fn init_memory(&mut self, capacity: u64) {
		self.memory.init(capacity);
	}
	
	/// Initializes Virtio block disk. This method is expected to be called only once.
	///
	/// # Arguments
	/// * `data` Filesystem binary content
	pub fn init_disk(&mut self, data: Vec<u8>) {
		self.disk.init(data);
	}

	/// Overrides defalut Device tree configuration.
	///
	/// # Arguments
	/// * `data` DTB binary content
	pub fn init_dtb(&mut self, data: Vec<u8>) {
		for i in 0..data.len() {
			self.dtb[i] = data[i];
		}
		for i in data.len()..self.dtb.len() {
			self.dtb[i] = 0;
		}
	}

	/// Enables or disables page cache optimization.
	///
	/// # Arguments
	/// * `enabled`
	pub fn enable_page_cache(&mut self, enabled: bool) {
		self.page_cache_enabled = enabled;
		self.clear_page_cache();
	}

	/// Clears page cache entries
	fn clear_page_cache(&mut self) {
		self.fetch_page_cache.clear();
		self.load_page_cache.clear();
		self.store_page_cache.clear();
	}

	/// Runs one cycle of MMU and peripheral devices.
	pub fn tick(&mut self, mip: &mut u64) {
		self.clint.tick(mip);
		self.disk.tick(&mut self.memory);
		self.uart.tick();
		self.plic.tick(self.disk.is_interrupting(), self.uart.is_interrupting(), mip);
		self.clock = self.clock.wrapping_add(1);
	}

	/// Updates addressing mode
	///
	/// # Arguments
	/// * `new_addressing_mode`
	pub fn update_addressing_mode(&mut self, new_addressing_mode: AddressingMode) {
		self.addressing_mode = new_addressing_mode;
		self.clear_page_cache();
	}

	/// Updates privilege mode
	///
	/// # Arguments
	/// * `mode`
	pub fn update_privilege_mode(&mut self, mode: PrivilegeMode) {
		self.privilege_mode = mode;
		self.clear_page_cache();
	}

	/// Updates PPN used for address translation
	///
	/// # Arguments
	/// * `ppn`
	pub fn update_ppn(&mut self, ppn: u64) {
		self.ppn = ppn;
		self.clear_page_cache();
	}

	fn get_effective_address(&self, address: u64) -> u64 {
		match self.xlen {
			Xlen::Bit32 => address & 0xffffffff,
			Xlen::Bit64 => address
		}
	}

	/// Fetches an instruction byte. This method takes virtual address
	/// and translates into physical address inside.
	///
	/// # Arguments
	/// * `v_address` Virtual address
	fn fetch(&mut self, v_address: u64) -> Result<u8, Trap> {
		match self.translate_address(v_address, MemoryAccessType::Execute) {
			Ok(p_address) => Ok(self.load_raw(p_address)),
			Err(()) => return Err(Trap {
				trap_type: TrapType::InstructionPageFault,
				value: v_address
			})
		}
	}

	/// Fetches instruction four bytes. This method takes virtual address
	/// and translates into physical address inside.
	///
	/// # Arguments
	/// * `v_address` Virtual address
	pub fn fetch_word(&mut self, v_address: u64) -> Result<u32, Trap> {
		let width = 4;
		match (v_address & 0xfff) <= (0x1000 - width) {
			true => {
				// Fast path. All bytes fetched are in the same page so
				// translating an address only once.
				let effective_address = self.get_effective_address(v_address);
				match self.translate_address(effective_address, MemoryAccessType::Execute) {
					Ok(p_address) => Ok(self.load_word_raw(p_address)),
					Err(()) => Err(Trap {
						trap_type: TrapType::InstructionPageFault,
						value: effective_address
					})
				}
			},
			false => {
				let mut data = 0 as u32;
				for i in 0..width {
					match self.fetch(v_address.wrapping_add(i)) {
						Ok(byte) => {
							data |= (byte as u32) << (i * 8)
						},
						Err(e) => return Err(e)
					};
				}
				Ok(data)
			}
		}
	}

	/// Loads an byte. This method takes virtual address and translates
	/// into physical address inside.
	///
	/// # Arguments
	/// * `v_address` Virtual address
	pub fn load(&mut self, v_address: u64) -> Result<u8, Trap> {
		let effective_address = self.get_effective_address(v_address);
		match self.translate_address(effective_address, MemoryAccessType::Read) {
			Ok(p_address) => Ok(self.load_raw(p_address)),
			Err(()) => Err(Trap {
				trap_type: TrapType::LoadPageFault,
				value: v_address
			})
		}
	}

	/// Loads multiple bytes. This method takes virtual address and translates
	/// into physical address inside.
	///
	/// # Arguments
	/// * `v_address` Virtual address
	/// * `width` Must be 1, 2, 4, or 8
	fn load_bytes(&mut self, v_address: u64, width: u64) -> Result<u64, Trap> {
		debug_assert!(width == 1 || width == 2 || width == 4 || width == 8,
			"Width must be 1, 2, 4, or 8. {:X}", width);
		match (v_address & 0xfff) <= (0x1000 - width) {
			true => match self.translate_address(v_address, MemoryAccessType::Read) {
				Ok(p_address) => {
					// Fast path. All bytes fetched are in the same page so
					// translating an address only once.
					match width {
						1 => Ok(self.load_raw(p_address) as u64),
						2 => Ok(self.load_halfword_raw(p_address) as u64),
						4 => Ok(self.load_word_raw(p_address) as u64),
						8 => Ok(self.load_doubleword_raw(p_address)),
						_ => panic!("Width must be 1, 2, 4, or 8. {:X}", width)
					}
				},
				Err(()) => Err(Trap {
					trap_type: TrapType::LoadPageFault,
					value: v_address
				})
			},
			false => {
				let mut data = 0 as u64;
				for i in 0..width {
					match self.load(v_address.wrapping_add(i)) {
						Ok(byte) => {
							data |= (byte as u64) << (i * 8)
						},
						Err(e) => return Err(e)
					};
				}
				Ok(data)
			}
		}
	}

	/// Loads two bytes. This method takes virtual address and translates
	/// into physical address inside.
	///
	/// # Arguments
	/// * `v_address` Virtual address
	pub fn load_halfword(&mut self, v_address: u64) -> Result<u16, Trap> {
		match self.load_bytes(v_address, 2) {
			Ok(data) => Ok(data as u16),
			Err(e) => Err(e)
		}
	}

	/// Loads four bytes. This method takes virtual address and translates
	/// into physical address inside.
	///
	/// # Arguments
	/// * `v_address` Virtual address
	pub fn load_word(&mut self, v_address: u64) -> Result<u32, Trap> {
		match self.load_bytes(v_address, 4) {
			Ok(data) => Ok(data as u32),
			Err(e) => Err(e)
		}
	}

	/// Loads eight bytes. This method takes virtual address and translates
	/// into physical address inside.
	///
	/// # Arguments
	/// * `v_address` Virtual address
	pub fn load_doubleword(&mut self, v_address: u64) -> Result<u64, Trap> {
		match self.load_bytes(v_address, 8) {
			Ok(data) => Ok(data as u64),
			Err(e) => Err(e)
		}
	}

	/// Store an byte. This method takes virtual address and translates
	/// into physical address inside.
	///
	/// # Arguments
	/// * `v_address` Virtual address
	/// * `value`
	pub fn store(&mut self, v_address: u64, value: u8) -> Result<(), Trap> {
		match self.translate_address(v_address, MemoryAccessType::Write) {
			Ok(p_address) => {
				self.store_raw(p_address, value);
				Ok(())
			},
			Err(()) => Err(Trap {
				trap_type: TrapType::StorePageFault,
				value: v_address
			})
		}
	}

	/// Stores multiple bytes. This method takes virtual address and translates
	/// into physical address inside.
	///
	/// # Arguments
	/// * `v_address` Virtual address
	/// * `value` data written
	/// * `width` Must be 1, 2, 4, or 8
	fn store_bytes(&mut self, v_address: u64, value: u64, width: u64) -> Result<(), Trap> {
		debug_assert!(width == 1 || width == 2 || width == 4 || width == 8,
			"Width must be 1, 2, 4, or 8. {:X}", width);
		match (v_address & 0xfff) <= (0x1000 - width) {
			true => match self.translate_address(v_address, MemoryAccessType::Write) {
				Ok(p_address) => {
					// Fast path. All bytes fetched are in the same page so
					// translating an address only once.
					match width {
						1 => self.store_raw(p_address, value as u8),
						2 => self.store_halfword_raw(p_address, value as u16),
						4 => self.store_word_raw(p_address, value as u32),
						8 => self.store_doubleword_raw(p_address, value),
						_ => panic!("Width must be 1, 2, 4, or 8. {:X}", width)
					}
					Ok(())
				},
				Err(()) => Err(Trap {
					trap_type: TrapType::StorePageFault,
					value: v_address
				})
			},
			false => {
				for i in 0..width {
					match self.store(v_address.wrapping_add(i), ((value >> (i * 8)) & 0xff) as u8) {
						Ok(()) => {},
						Err(e) => return Err(e)
					}
				}
				Ok(())
			}
		}
	}

	/// Stores two bytes. This method takes virtual address and translates
	/// into physical address inside.
	///
	/// # Arguments
	/// * `v_address` Virtual address
	/// * `value` data written
	pub fn store_halfword(&mut self, v_address: u64, value: u16) -> Result<(), Trap> {
		self.store_bytes(v_address, value as u64, 2)
	}

	/// Stores four bytes. This method takes virtual address and translates
	/// into physical address inside.
	///
	/// # Arguments
	/// * `v_address` Virtual address
	/// * `value` data written
	pub fn store_word(&mut self, v_address: u64, value: u32) -> Result<(), Trap> {
		self.store_bytes(v_address, value as u64, 4)
	}

	/// Stores eight bytes. This method takes virtual address and translates
	/// into physical address inside.
	///
	/// # Arguments
	/// * `v_address` Virtual address
	/// * `value` data written
	pub fn store_doubleword(&mut self, v_address: u64, value: u64) -> Result<(), Trap> {
		self.store_bytes(v_address, value as u64, 8)
	}

	/// Loads a byte from main memory or peripheral devices depending on
	/// physical address.
	///
	/// # Arguments
	/// * `p_address` Physical address
	fn load_raw(&mut self, p_address: u64) -> u8 {
		let effective_address = self.get_effective_address(p_address);
		// @TODO: Mapping should be configurable with dtb
		match effective_address >= DRAM_BASE {
			true => self.memory.read_byte(effective_address),
			false => match effective_address {
				// I don't know why but dtb data seems to be stored from 0x1020 on Linux.
				// It might be from self.x[0xb] initialization?
				// And DTB size is arbitray.
				0x00001020..=0x00001fff => self.dtb[effective_address as usize - 0x1020],
				0x02000000..=0x0200ffff => self.clint.load(effective_address),
				0x0C000000..=0x0fffffff => self.plic.load(effective_address),
				0x10000000..=0x100000ff => self.uart.load(effective_address),
				0x10001000..=0x10001FFF => self.disk.load(effective_address),
				_ => panic!("Unknown memory mapping {:X}.", effective_address)
			}
		}
	}

	/// Loads two bytes from main memory or peripheral devices depending on
	/// physical address.
	///
	/// # Arguments
	/// * `p_address` Physical address
	fn load_halfword_raw(&mut self, p_address: u64) -> u16 {
		let effective_address = self.get_effective_address(p_address);
		match effective_address >= DRAM_BASE && effective_address.wrapping_add(1) > effective_address {
			// Fast path. Directly load main memory at a time.
			true => self.memory.read_halfword(effective_address),
			false => {
				let mut data = 0 as u16;
				for i in 0..2 {
					data |= (self.load_raw(effective_address.wrapping_add(i)) as u16) << (i * 8)
				}
				data
			}
		}
	}

	/// Loads four bytes from main memory or peripheral devices depending on
	/// physical address.
	///
	/// # Arguments
	/// * `p_address` Physical address
	pub fn load_word_raw(&mut self, p_address: u64) -> u32 {
		let effective_address = self.get_effective_address(p_address);
		match effective_address >= DRAM_BASE && effective_address.wrapping_add(3) > effective_address {
			// Fast path. Directly load main memory at a time.
			true => self.memory.read_word(effective_address),
			false => {
				let mut data = 0 as u32;
				for i in 0..4 {
					data |= (self.load_raw(effective_address.wrapping_add(i)) as u32) << (i * 8)
				}
				data
			}
		}
	}

	/// Loads eight bytes from main memory or peripheral devices depending on
	/// physical address.
	///
	/// # Arguments
	/// * `p_address` Physical address
	fn load_doubleword_raw(&mut self, p_address: u64) -> u64 {
		let effective_address = self.get_effective_address(p_address);
		match effective_address >= DRAM_BASE && effective_address.wrapping_add(7) > effective_address {
			// Fast path. Directly load main memory at a time.
			true => self.memory.read_doubleword(effective_address),
			false => {
				let mut data = 0 as u64;
				for i in 0..8 {
					data |= (self.load_raw(effective_address.wrapping_add(i)) as u64) << (i * 8)
				}
				data
			}
		}
	}

	/// Stores a byte to main memory or peripheral devices depending on
	/// physical address.
	///
	/// # Arguments
	/// * `p_address` Physical address
	/// * `value` data written
	pub fn store_raw(&mut self, p_address: u64, value: u8) {
		let effective_address = self.get_effective_address(p_address);
		// @TODO: Mapping should be configurable with dtb
		match effective_address >= DRAM_BASE {
			true => self.memory.write_byte(effective_address, value),
			false => match effective_address {
				0x02000000..=0x0200ffff => self.clint.store(effective_address, value),
				0x0c000000..=0x0fffffff => self.plic.store(effective_address, value),
				0x10000000..=0x100000ff => self.uart.store(effective_address, value),
				0x10001000..=0x10001FFF => self.disk.store(effective_address, value),
				_ => panic!("Unknown memory mapping {:X}.", effective_address)
			}
		};
	}

	/// Stores two bytes to main memory or peripheral devices depending on
	/// physical address.
	///
	/// # Arguments
	/// * `p_address` Physical address
	/// * `value` data written
	fn store_halfword_raw(&mut self, p_address: u64, value: u16) {
		let effective_address = self.get_effective_address(p_address);
		match effective_address >= DRAM_BASE && effective_address.wrapping_add(1) > effective_address {
			// Fast path. Directly store to main memory at a time.
			true => self.memory.write_halfword(effective_address, value),
			false => {
				for i in 0..2 {
					self.store_raw(effective_address.wrapping_add(i), ((value >> (i * 8)) & 0xff) as u8);
				}
			}
		}
	}

	/// Stores four bytes to main memory or peripheral devices depending on
	/// physical address.
	///
	/// # Arguments
	/// * `p_address` Physical address
	/// * `value` data written
	fn store_word_raw(&mut self, p_address: u64, value: u32) {
		let effective_address = self.get_effective_address(p_address);
		match effective_address >= DRAM_BASE && effective_address.wrapping_add(3) > effective_address {
			// Fast path. Directly store to main memory at a time.
			true => self.memory.write_word(effective_address, value),
			false => {
				for i in 0..4 {
					self.store_raw(effective_address.wrapping_add(i), ((value >> (i * 8)) & 0xff) as u8);
				}
			}
		}
	}

	/// Stores eight bytes to main memory or peripheral devices depending on
	/// physical address.
	///
	/// # Arguments
	/// * `p_address` Physical address
	/// * `value` data written
	fn store_doubleword_raw(&mut self, p_address: u64, value: u64) {
		let effective_address = self.get_effective_address(p_address);
		match effective_address >= DRAM_BASE && effective_address.wrapping_add(7) > effective_address {
			// Fast path. Directly store to main memory at a time.
			true => self.memory.write_doubleword(effective_address, value),
			false => {
				for i in 0..8 {
					self.store_raw(effective_address.wrapping_add(i), ((value >> (i * 8)) & 0xff) as u8);
				}
			}
		}
	}

	/// Checks if passed virtual address is valid (pointing a certain device) or not.
	/// This method can return page fault trap.
	///
	/// # Arguments
	/// * `v_address` Virtual address
	pub fn validate_address(&mut self, v_address: u64) -> Result<bool, ()> {
		// @TODO: Support other access types?
		let p_address = match self.translate_address(v_address, MemoryAccessType::DontCare) {
			Ok(address) => address,
			Err(()) => return Err(())
		};
		let effective_address = self.get_effective_address(p_address);
		let valid = match effective_address >= DRAM_BASE {
			true => self.memory.validate_address(effective_address),
			false => match effective_address {
				0x00001020..=0x00001fff => true,
				0x02000000..=0x0200ffff => true,
				0x0C000000..=0x0fffffff => true,
				0x10000000..=0x100000ff => true,
				0x10001000..=0x10001FFF => true,
				_ => false
			}
		};
		Ok(valid)
	}

	fn translate_address(&mut self, v_address: u64, access_type: MemoryAccessType) -> Result<u64, ()> {
		let address = self.get_effective_address(v_address);
		let v_page = address & !0xfff;
		let cache = match self.page_cache_enabled {
			true => match access_type {
				MemoryAccessType::Execute => self.fetch_page_cache.get(&v_page),
				MemoryAccessType::Read => self.load_page_cache.get(&v_page),
				MemoryAccessType::Write => self.store_page_cache.get(&v_page),
				MemoryAccessType::DontCare => None,
			},
			false => None
		};
		match cache {
			Some(p_page) => Ok(p_page | (address & 0xfff)),
			None => {
				let p_address = match self.addressing_mode {
					AddressingMode::None => Ok(address),
					AddressingMode::SV32 => match self.privilege_mode {
						PrivilegeMode::User | PrivilegeMode::Supervisor => {
							let vpns = [(address >> 12) & 0x3ff, (address >> 22) & 0x3ff];
							self.traverse_page(address, 2 - 1, self.ppn, &vpns, &access_type)
						},
						_ => Ok(address)
					},
					AddressingMode::SV39 => match self.privilege_mode {
						PrivilegeMode::User | PrivilegeMode::Supervisor => {
							let vpns = [(address >> 12) & 0x1ff, (address >> 21) & 0x1ff, (address >> 30) & 0x1ff];
							self.traverse_page(address, 3 - 1, self.ppn, &vpns, &access_type)
						},
						_ => Ok(address)
					},
					AddressingMode::SV48 => {
						panic!("AddressingMode SV48 is not supported yet.");
					}
				};
				match self.page_cache_enabled {
					true => match p_address {
						Ok(p_address) => {
							let p_page = p_address & !0xfff;
							match access_type {
								MemoryAccessType::Execute => self.fetch_page_cache.insert(v_page, p_page),
								MemoryAccessType::Read => self.load_page_cache.insert(v_page, p_page),
								MemoryAccessType::Write => self.store_page_cache.insert(v_page, p_page),
								MemoryAccessType::DontCare => None,
							};
							Ok(p_address)
						},
						Err(()) => Err(())
					},
					false => p_address
				}
			}
		}
	}

	fn traverse_page(&mut self, v_address: u64, level: u8, parent_ppn: u64,
		vpns: &[u64], access_type: &MemoryAccessType) -> Result<u64, ()> {
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

	/// Returns immutable reference to `Clint`.
	pub fn get_clint(&self) -> &Clint {
		&self.clint
	}

	/// Returns mutable reference to `Clint`.
	pub fn get_mut_clint(&mut self) -> &mut Clint {
		&mut self.clint
	}

	/// Returns mutable reference to `Uart`.
	pub fn get_mut_uart(&mut self) -> &mut Uart {
		&mut self.uart
	}
}

/// [`Memory`](../memory/struct.Memory.html) wrapper. Converts physical address to the one in memory
/// using [`DRAM_BASE`](constant.DRAM_BASE.html) and accesses [`Memory`](../memory/struct.Memory.html).
pub struct MemoryWrapper {
	memory: Memory
}

impl MemoryWrapper {
	fn new() -> Self {
		MemoryWrapper {
			memory: Memory::new()
		}
	}

	fn init(&mut self, capacity: u64) {
		self.memory.init(capacity);
	}

	pub fn read_byte(&mut self, p_address: u64) -> u8 {
		debug_assert!(p_address >= DRAM_BASE, "Memory address must equals to or bigger than DRAM_BASE. {:X}", p_address);
		self.memory.read_byte(p_address - DRAM_BASE)
	}

	pub fn read_halfword(&mut self, p_address: u64) -> u16 {
		debug_assert!(p_address >= DRAM_BASE && p_address.wrapping_add(1) >= DRAM_BASE,
			"Memory address must equals to or bigger than DRAM_BASE. {:X}", p_address);
		self.memory.read_halfword(p_address - DRAM_BASE)
	}

	pub fn read_word(&mut self, p_address: u64) -> u32 {
		debug_assert!(p_address >= DRAM_BASE && p_address.wrapping_add(3) >= DRAM_BASE,
			"Memory address must equals to or bigger than DRAM_BASE. {:X}", p_address);
		self.memory.read_word(p_address - DRAM_BASE)
	}

	pub fn read_doubleword(&mut self, p_address: u64) -> u64 {
		debug_assert!(p_address >= DRAM_BASE && p_address.wrapping_add(7) >= DRAM_BASE,
			"Memory address must equals to or bigger than DRAM_BASE. {:X}", p_address);
		self.memory.read_doubleword(p_address - DRAM_BASE)
	}

	pub fn write_byte(&mut self, p_address: u64, value: u8) {
		debug_assert!(p_address >= DRAM_BASE, "Memory address must equals to or bigger than DRAM_BASE. {:X}", p_address);
		self.memory.write_byte(p_address - DRAM_BASE, value)
	}

	pub fn write_halfword(&mut self, p_address: u64, value: u16) {
		debug_assert!(p_address >= DRAM_BASE && p_address.wrapping_add(1) >= DRAM_BASE,
			"Memory address must equals to or bigger than DRAM_BASE. {:X}", p_address);
		self.memory.write_halfword(p_address - DRAM_BASE, value)
	}

	pub fn write_word(&mut self, p_address: u64, value: u32) {
		debug_assert!(p_address >= DRAM_BASE && p_address.wrapping_add(3) >= DRAM_BASE,
			"Memory address must equals to or bigger than DRAM_BASE. {:X}", p_address);
		self.memory.write_word(p_address - DRAM_BASE, value)
	}

	pub fn write_doubleword(&mut self, p_address: u64, value: u64) {
		debug_assert!(p_address >= DRAM_BASE && p_address.wrapping_add(7) >= DRAM_BASE,
			"Memory address must equals to or bigger than DRAM_BASE. {:X}", p_address);
		self.memory.write_doubleword(p_address - DRAM_BASE, value)
	}

	pub fn validate_address(&self, address: u64) -> bool {
		self.memory.validate_address(address - DRAM_BASE)
	}
}

const DTB_CONTENT_SIZE: usize = 400;
const DTB_CONTENT: [u32; DTB_CONTENT_SIZE] = [
  0xedfe0dd0, 0x3e060000, 0x38000000, 0x3c050000,
  0x28000000, 0x11000000, 0x10000000, 0x00000000,
  0x02010000, 0x04050000, 0x00000000, 0x00000000,
  0x00000000, 0x00000000, 0x01000000, 0x00000000,
  0x03000000, 0x04000000, 0x00000000, 0x02000000,
  0x03000000, 0x04000000, 0x0f000000, 0x02000000,
  0x03000000, 0x0d000000, 0x1b000000, 0x63736972,
  0x69762d76, 0x6f697472, 0x00000000, 0x03000000,
  0x12000000, 0x26000000, 0x63736972, 0x69762d76,
  0x6f697472, 0x6d65712c, 0x00000075, 0x01000000,
  0x736f6863, 0x00006e65, 0x03000000, 0x1f000000,
  0x2c000000, 0x746f6f72, 0x65642f3d, 0x64762f76,
  0x6f722061, 0x6e6f6320, 0x656c6f73, 0x7974743d,
  0x00003035, 0x03000000, 0x0f000000, 0x35000000,
  0x7261752f, 0x30314074, 0x30303030, 0x00003030,
  0x02000000, 0x01000000, 0x74726175, 0x30303140,
  0x30303030, 0x00000030, 0x03000000, 0x04000000,
  0x41000000, 0x0a000000, 0x03000000, 0x04000000,
  0x4c000000, 0x03000000, 0x03000000, 0x04000000,
  0x5d000000, 0x00403800, 0x03000000, 0x10000000,
  0x6d000000, 0x00000000, 0x00000010, 0x00000000,
  0x00010000, 0x03000000, 0x09000000, 0x1b000000,
  0x3631736e, 0x61303535, 0x00000000, 0x02000000,
  0x01000000, 0x74726976, 0x6d5f6f69, 0x406f696d,
  0x30303031, 0x30303031, 0x00000000, 0x03000000,
  0x04000000, 0x41000000, 0x01000000, 0x03000000,
  0x04000000, 0x4c000000, 0x03000000, 0x03000000,
  0x10000000, 0x6d000000, 0x00000000, 0x00100010,
  0x00000000, 0x00100000, 0x03000000, 0x0c000000,
  0x1b000000, 0x74726976, 0x6d2c6f69, 0x006f696d,
  0x02000000, 0x01000000, 0x73757063, 0x00000000,
  0x03000000, 0x04000000, 0x00000000, 0x01000000,
  0x03000000, 0x04000000, 0x0f000000, 0x00000000,
  0x03000000, 0x04000000, 0x71000000, 0x80969800,
  0x01000000, 0x2d757063, 0x0070616d, 0x01000000,
  0x73756c63, 0x30726574, 0x00000000, 0x01000000,
  0x65726f63, 0x00000030, 0x03000000, 0x04000000,
  0x84000000, 0x01000000, 0x02000000, 0x02000000,
  0x02000000, 0x01000000, 0x40757063, 0x00000030,
  0x03000000, 0x04000000, 0x88000000, 0x01000000,
  0x03000000, 0x04000000, 0x90000000, 0x00757063,
  0x03000000, 0x04000000, 0x6d000000, 0x00000000,
  0x03000000, 0x05000000, 0x9c000000, 0x79616b6f,
  0x00000000, 0x03000000, 0x06000000, 0x1b000000,
  0x63736972, 0x00000076, 0x03000000, 0x0d000000,
  0xa3000000, 0x34367672, 0x66616d69, 0x75736364,
  0x00000000, 0x03000000, 0x0b000000, 0xad000000,
  0x63736972, 0x76732c76, 0x00003933, 0x01000000,
  0x65746e69, 0x70757272, 0x6f632d74, 0x6f72746e,
  0x72656c6c, 0x00000000, 0x03000000, 0x04000000,
  0xb6000000, 0x01000000, 0x03000000, 0x00000000,
  0xc7000000, 0x03000000, 0x0f000000, 0x1b000000,
  0x63736972, 0x70632c76, 0x6e692d75, 0x00006374,
  0x03000000, 0x04000000, 0x88000000, 0x02000000,
  0x02000000, 0x02000000, 0x02000000, 0x01000000,
  0x6f6d656d, 0x38407972, 0x30303030, 0x00303030,
  0x03000000, 0x07000000, 0x90000000, 0x6f6d656d,
  0x00007972, 0x03000000, 0x10000000, 0x6d000000,
  0x00000000, 0x00000080, 0x00000000, 0x00000008,
  0x02000000, 0x01000000, 0x00636f73, 0x03000000,
  0x04000000, 0x00000000, 0x02000000, 0x03000000,
  0x04000000, 0x0f000000, 0x02000000, 0x03000000,
  0x0b000000, 0x1b000000, 0x706d6973, 0x622d656c,
  0x00007375, 0x03000000, 0x00000000, 0xdc000000,
  0x01000000, 0x65746e69, 0x70757272, 0x6f632d74,
  0x6f72746e, 0x72656c6c, 0x30306340, 0x30303030,
  0x00000000, 0x03000000, 0x04000000, 0x88000000,
  0x03000000, 0x03000000, 0x04000000, 0xe3000000,
  0x35000000, 0x03000000, 0x10000000, 0x6d000000,
  0x00000000, 0x0000000c, 0x00000000, 0x00000004,
  0x03000000, 0x10000000, 0xee000000, 0x02000000,
  0x0b000000, 0x02000000, 0x09000000, 0x03000000,
  0x00000000, 0xc7000000, 0x03000000, 0x0c000000,
  0x1b000000, 0x63736972, 0x6c702c76, 0x00306369,
  0x03000000, 0x04000000, 0xb6000000, 0x01000000,
  0x03000000, 0x04000000, 0x00000000, 0x00000000,
  0x02000000, 0x01000000, 0x6e696c63, 0x30324074,
  0x30303030, 0x00000030, 0x03000000, 0x10000000,
  0xee000000, 0x02000000, 0x03000000, 0x02000000,
  0x07000000, 0x03000000, 0x10000000, 0x6d000000,
  0x00000000, 0x00000002, 0x00000000, 0x00000100,
  0x03000000, 0x0d000000, 0x1b000000, 0x63736972,
  0x6c632c76, 0x30746e69, 0x00000000, 0x02000000,
  0x02000000, 0x02000000, 0x09000000, 0x64646123,
  0x73736572, 0x6c65632d, 0x2300736c, 0x657a6973,
  0x6c65632d, 0x6300736c, 0x61706d6f, 0x6c626974,
  0x6f6d0065, 0x006c6564, 0x746f6f62, 0x73677261,
  0x64747300, 0x2d74756f, 0x68746170, 0x746e6900,
  0x75727265, 0x00737470, 0x65746e69, 0x70757272,
  0x61702d74, 0x746e6572, 0x6f6c6300, 0x662d6b63,
  0x75716572, 0x79636e65, 0x67657200, 0x6d697400,
  0x73616265, 0x72662d65, 0x65757165, 0x0079636e,
  0x00757063, 0x6e616870, 0x00656c64, 0x69766564,
  0x745f6563, 0x00657079, 0x74617473, 0x72007375,
  0x76637369, 0x6173692c, 0x756d6d00, 0x7079742d,
  0x69230065, 0x7265746e, 0x74707572, 0x6c65632d,
  0x6900736c, 0x7265746e, 0x74707572, 0x6e6f632d,
  0x6c6f7274, 0x0072656c, 0x676e6172, 0x72007365,
  0x76637369, 0x65646e2c, 0x6e690076, 0x72726574,
  0x73747075, 0x7478652d, 0x65646e65, 0x00000064,
];

fn load_default_dtb_content(dtb: &mut Vec<u8>) {
	for i in 0..DTB_CONTENT_SIZE {
		dtb[i * 4] = DTB_CONTENT[i] as u8;
		dtb[i * 4 + 1] = (DTB_CONTENT[i] >> 8) as u8;
		dtb[i * 4 + 2] = (DTB_CONTENT[i] >> 16) as u8;
		dtb[i * 4 + 3] = (DTB_CONTENT[i] >> 24) as u8;
	}
}

// Corresponding DTS content. The above binary is made with the
// following command.
// $ dtc -I dtb -O dts dts.dts
/*
/dts-v1/;

/ {
	#address-cells = <0x2>;
	#size-cells = <0x2>;
	compatible = "riscv-virtio";
	model = "riscv-virtio,qemu";

	chosen {
		bootargs = "root=/dev/vda ro console=tty50";
		stdout-path = "/uart@10000000";
	};

	uart@10000000 {
		interrupts = <0xa>;
		interrupt-parent = <0x3>;
		clock-frequency = <0x384000>;
		reg = <0x0 0x10000000 0x0 0x100>;
		compatible = "ns16550a";
	};

	virtio_mmio@10001000 {
		interrupts = <0x1>;
		interrupt-parent = <0x3>;
		reg = <0x0 0x10001000 0x0 0x1000>;
		compatible = "virtio,mmio";
	};

	cpus {
		#address-cells = <0x1>;
		#size-cells = <0x0>;
		timebase-frequency = <0x989680>;

		cpu-map {

			cluster0 {

				core0 {
					cpu = <0x1>;
				};
			};
		};

		cpu@0 {
			phandle = <0x1>;
			device_type = "cpu";
			reg = <0x0>;
			status = "okay";
			compatible = "riscv";
			riscv,isa = "rv64imafdcsu";
			mmu-type = "riscv,sv39";

			interrupt-controller {
				#interrupt-cells = <0x1>;
				interrupt-controller;
				compatible = "riscv,cpu-intc";
				phandle = <0x2>;
			};
		};
	};

	memory@80000000 {
		device_type = "memory";
		reg = <0x0 0x80000000 0x0 0x8000000>;
	};

	soc {
		#address-cells = <0x2>;
		#size-cells = <0x2>;
		compatible = "simple-bus";
		ranges;

		interrupt-controller@c000000 {
			phandle = <0x3>;
			riscv,ndev = <0x35>;
			reg = <0x0 0xc000000 0x0 0x4000000>;
			interrupts-extended = <0x2 0xb 0x2 0x9>;
			interrupt-controller;
			compatible = "riscv,plic0";
			#interrupt-cells = <0x1>;
			#address-cells = <0x0>;
		};

		clint@2000000 {
			interrupts-extended = <0x2 0x3 0x2 0x7>;
			reg = <0x0 0x2000000 0x0 0x10000>;
			compatible = "riscv,clint0";
		};
	};
};
*/