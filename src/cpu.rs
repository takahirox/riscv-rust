use mmu::{AddressingMode, Mmu};
use plic::InterruptType;
use terminal::Terminal;

const CSR_CAPACITY: usize = 4096;

const CSR_USTATUS_ADDRESS: u16 = 0x000;
const _CSR_UIR_ADDRESS: u16 = 0x004;
const CSR_UTVEC_ADDRESS: u16 = 0x005;
const _CSR_USCRATCH_ADDRESS: u16 = 0x040;
const CSR_UEPC_ADDRESS: u16 = 0x041;
const CSR_UCAUSE_ADDRESS: u16 = 0x042;
const CSR_UTVAL_ADDRESS: u16 = 0x043;
const _CSR_UIP_ADDRESS: u16 = 0x044;
const CSR_SSTATUS_ADDRESS: u16 = 0x100;
const CSR_SEDELEG_ADDRESS: u16 = 0x102;
const CSR_SIDELEG_ADDRESS: u16 = 0x103;
const CSR_STVEC_ADDRESS: u16 = 0x105;
const _CSR_SSCRATCH_ADDRESS: u16 = 0x140;
const CSR_SEPC_ADDRESS: u16 = 0x141;
const CSR_SCAUSE_ADDRESS: u16 = 0x142;
const CSR_STVAL_ADDRESS: u16 = 0x143;
const CSR_SATP_ADDRESS: u16 = 0x180;
const CSR_MSTATUS_ADDRESS: u16 = 0x300;
const CSR_MEDELEG_ADDRESS: u16 = 0x302;
const CSR_MIDELEG_ADDRESS: u16 = 0x303;
const _CSR_MIE_ADDRESS: u16 = 0x304;
const CSR_MTVEC_ADDRESS: u16 = 0x305;
const _CSR_MSCRATCH_ADDRESS: u16 = 0x340;
const CSR_MEPC_ADDRESS: u16 = 0x341;
const CSR_MCAUSE_ADDRESS: u16 = 0x342;
const CSR_MTVAL_ADDRESS: u16 = 0x343;
const _CSR_PMPCFG0_ADDRESS: u16 = 0x3a0;
const _CSR_PMPADDR0_ADDRESS: u16 = 0x3b0;
const _CSR_MHARTID_ADDRESS: u16 = 0xf14;

pub struct Cpu {
	clock: u64,
	xlen: Xlen,
	privilege_mode: PrivilegeMode,
	// using only lower 32bits of x, pc, and csr registers
	// for 32-bit mode
	x: [i64; 32],
	pc: u64,
	csr: [u64; CSR_CAPACITY],
	mmu: Mmu
}

#[derive(Clone)]
pub enum Xlen {
	Bit32,
	Bit64
	// @TODO: Support Bit128
}

#[derive(Clone)]
#[allow(dead_code)]
pub enum PrivilegeMode {
	User,
	Supervisor,
	Reserved,
	Machine
}

pub struct Trap {
	pub trap_type: TrapType,
	pub value: u64 // Trap type specific value
}

#[allow(dead_code)]
pub enum TrapType {
	InstructionAddressMisaligned,
	InstructionAccessFault,
	IllegalInstruction,
	Breakpoint,
	LoadAddressMisaligned,
	LoadAccessFault,
	StoreAddressMisaligned,
	StoreAccessFault,
	EnvironmentCallFromUMode,
	EnvironmentCallFromSMode,
	EnvironmentCallFromMMode,
	InstructionPageFault,
	LoadPageFault,
	StorePageFault,
	UserSoftwareInterrupt,
	SupervisorSoftwareInterrupt,
	MachineSoftwareInterrupt,
	UserTimerInterrupt,
	SupervisorTimerInterrupt,
	MachineTimerInterrupt,
	UserExternalInterrupt,
	SupervisorExternalInterrupt,
	MachineExternalInterrupt
}

enum Instruction {
	ADD,
	ADDI,
	ADDIW,
	ADDW,
	AMOSWAPW,
	AND,
	ANDI,
	AUIPC,
	BEQ,
	BGE,
	BGEU,
	BLT,
	BLTU,
	BNE,
	CSRRC,
	CSRRCI,
	CSRRS,
	CSRRSI,
	CSRRW,
	CSRRWI,
	DIV,
	DIVU,
	DIVUW,
	DIVW,
	ECALL,
	FENCE,
	JAL,
	JALR,
	LB,
	LBU,
	LD,
	LH,
	LHU,
	LUI,
	LW,
	LWU,
	MUL,
	MULH,
	MULHU,
	MULHSU,
	MULW,
	MRET,
	OR,
	ORI,
	REM,
	REMU,
	REMUW,
	REMW,
	SB,
	SD,
	SFENCEVMA,
	SH,
	SLL,
	SLLI,
	SLLIW,
	SLLW,
	SLT,
	SLTI,
	SLTU,
	SLTIU,
	SRA,
	SRAI,
	SRAIW,
	SRAW,
	SRET,
	SRL,
	SRLI,
	SRLIW,
	SRLW,
	SUB,
	SUBW,
	SW,
	URET,
	XOR,
	XORI
}

enum InstructionFormat {
	B,
	C, // CSR
	I,
	J,
	O, // Other, temporal
	R,
	S,
	U
}

fn _get_privilege_mode_name(mode: &PrivilegeMode) -> &'static str {
	match mode {
		PrivilegeMode::User => "User",
		PrivilegeMode::Supervisor => "Supervisor",
		PrivilegeMode::Reserved => "Reserved",
		PrivilegeMode::Machine => "Machine"
	}
}

// bigger number is higher privilege level
fn get_privilege_encoding(mode: &PrivilegeMode) -> u8 {
	match mode {
		PrivilegeMode::User => 0,
		PrivilegeMode::Supervisor => 1,
		PrivilegeMode::Reserved => panic!(),
		PrivilegeMode::Machine => 3
	}
}

fn get_trap_type_name(trap_type: &TrapType) -> &'static str {
	match trap_type {
		TrapType::InstructionAddressMisaligned => "InstructionAddressMisaligned",
		TrapType::InstructionAccessFault => "InstructionAccessFault",
		TrapType::IllegalInstruction => "IllegalInstruction",
		TrapType::Breakpoint => "Breakpoint",
		TrapType::LoadAddressMisaligned => "LoadAddressMisaligned",
		TrapType::LoadAccessFault => "LoadAccessFault",
		TrapType::StoreAddressMisaligned => "StoreAddressMisaligned",
		TrapType::StoreAccessFault => "StoreAccessFault",
		TrapType::EnvironmentCallFromUMode => "EnvironmentCallFromUMode",
		TrapType::EnvironmentCallFromSMode => "EnvironmentCallFromSMode",
		TrapType::EnvironmentCallFromMMode => "EnvironmentCallFromMMode",
		TrapType::InstructionPageFault => "InstructionPageFault",
		TrapType::LoadPageFault => "LoadPageFault",
		TrapType::StorePageFault => "StorePageFault",
		TrapType::UserSoftwareInterrupt => "UserSoftwareInterrupt",
		TrapType::SupervisorSoftwareInterrupt => "SupervisorSoftwareInterrupt",
		TrapType::MachineSoftwareInterrupt => "MachineSoftwareInterrupt",
		TrapType::UserTimerInterrupt => "UserTimerInterrupt",
		TrapType::SupervisorTimerInterrupt => "SupervisorTimerInterrupt",
		TrapType::MachineTimerInterrupt => "MachineTimerInterrupt",
		TrapType::UserExternalInterrupt => "UserExternalInterrupt",
		TrapType::SupervisorExternalInterrupt => "SupervisorExternalInterrupt",
		TrapType::MachineExternalInterrupt => "MachineExternalInterrupt"
	}
}

fn get_trap_cause(trap: &Trap, xlen: &Xlen) -> u64 {
	let interrupt_bit = match xlen {
		Xlen::Bit32 => 0x80000000 as u64,
		Xlen::Bit64 => 0x8000000000000000 as u64,
	};
	match trap.trap_type {
		TrapType::InstructionAddressMisaligned => 0,
		TrapType::InstructionAccessFault => 1,
		TrapType::IllegalInstruction => 2,
		TrapType::Breakpoint => 3,
		TrapType::LoadAddressMisaligned => 4,
		TrapType::LoadAccessFault => 5,
		TrapType::StoreAddressMisaligned => 6,
		TrapType::StoreAccessFault => 7,
		TrapType::EnvironmentCallFromUMode => 8,
		TrapType::EnvironmentCallFromSMode => 9,
		TrapType::EnvironmentCallFromMMode => 11,
		TrapType::InstructionPageFault => 12,
		TrapType::LoadPageFault => 13,
		TrapType::StorePageFault => 15,
		TrapType::UserSoftwareInterrupt => interrupt_bit,
		TrapType::SupervisorSoftwareInterrupt => interrupt_bit + 1,
		TrapType::MachineSoftwareInterrupt => interrupt_bit + 3,
		TrapType::UserTimerInterrupt => interrupt_bit + 4,
		TrapType::SupervisorTimerInterrupt => interrupt_bit + 5,
		TrapType::MachineTimerInterrupt => interrupt_bit + 7,
		TrapType::UserExternalInterrupt => interrupt_bit + 8,
		TrapType::SupervisorExternalInterrupt => interrupt_bit + 9,
		TrapType::MachineExternalInterrupt => interrupt_bit + 11
	}
}

fn get_interrupt_privilege_mode(trap: &Trap) -> PrivilegeMode {
	match trap.trap_type {
		TrapType::MachineSoftwareInterrupt |
		TrapType::MachineTimerInterrupt |
		TrapType::MachineExternalInterrupt => PrivilegeMode::Machine,
		TrapType::SupervisorSoftwareInterrupt |
		TrapType::SupervisorTimerInterrupt |
		TrapType::SupervisorExternalInterrupt => PrivilegeMode::Supervisor,
		TrapType::UserSoftwareInterrupt |
		TrapType::UserTimerInterrupt |
		TrapType::UserExternalInterrupt => PrivilegeMode::User,
		_ => panic!("{} is not an interrupt", get_trap_type_name(&trap.trap_type))
	}
}

fn get_instruction_name(instruction: &Instruction) -> &'static str {
	match instruction {
		Instruction::ADD => "ADD",
		Instruction::ADDI => "ADDI",
		Instruction::ADDIW => "ADDIW",
		Instruction::ADDW => "ADDW",
		Instruction::AMOSWAPW => "AMOSWAP.W",
		Instruction::AND => "AND",
		Instruction::ANDI => "ANDI",
		Instruction::AUIPC => "AUIPC",
		Instruction::BEQ => "BEQ",
		Instruction::BGE => "BGE",
		Instruction::BGEU => "BGEU",
		Instruction::BLT => "BLT",
		Instruction::BLTU => "BLTU",
		Instruction::BNE => "BNE",
		Instruction::CSRRC => "CSRRC",
		Instruction::CSRRCI => "CSRRCI",
		Instruction::CSRRS => "CSRRS",
		Instruction::CSRRSI => "CSRRSI",
		Instruction::CSRRW => "CSRRW",
		Instruction::CSRRWI => "CSRRWI",
		Instruction::DIV => "DIV",
		Instruction::DIVU => "DIVU",
		Instruction::DIVUW => "DIVUW",
		Instruction::DIVW => "DIVW",
		Instruction::ECALL => "ECALL",
		Instruction::FENCE => "FENCE",
		Instruction::JAL => "JAL",
		Instruction::JALR => "JALR",
		Instruction::LB => "LB",
		Instruction::LBU => "LBU",
		Instruction::LD => "LD",
		Instruction::LH => "LH",
		Instruction::LHU => "LHU",
		Instruction::LUI => "LUI",
		Instruction::LW => "LW",
		Instruction::LWU => "LWU",
		Instruction::MRET => "MRET",
		Instruction::MUL => "MUL",
		Instruction::MULH => "MULH",
		Instruction::MULHU => "MULHU",
		Instruction::MULHSU => "MULHSU",
		Instruction::MULW => "MULW",
		Instruction::OR => "OR",
		Instruction::ORI => "ORI",
		Instruction::REM => "REM",
		Instruction::REMU => "REMU",
		Instruction::REMUW => "REMUW",
		Instruction::REMW => "REMW",
		Instruction::SB => "SB",
		Instruction::SD => "SD",
		Instruction::SFENCEVMA => "SFENCE_VMA",
		Instruction::SH => "SH",
		Instruction::SLL => "SLL",
		Instruction::SLLI => "SLLI",
		Instruction::SLLIW => "SLLIW",
		Instruction::SLLW => "SLLW",
		Instruction::SLT => "SLT",
		Instruction::SLTI => "SLTI",
		Instruction::SLTU => "SLTU",
		Instruction::SLTIU => "SLTIU",
		Instruction::SRA => "SRA",
		Instruction::SRAI => "SRAI",
		Instruction::SRAIW => "SRAIW",
		Instruction::SRAW => "SRAW",
		Instruction::SRET => "SRET",
		Instruction::SRL => "SRL",
		Instruction::SRLI => "SRLI",
		Instruction::SRLIW => "SRLIW",
		Instruction::SRLW => "SRLW",
		Instruction::SUB => "SUB",
		Instruction::SUBW => "SUBW",
		Instruction::SW => "SW",
		Instruction::URET => "URET",
		Instruction::XOR => "XOR",
		Instruction::XORI => "XORI"
	}
}

fn get_instruction_format(instruction: &Instruction) -> InstructionFormat {
	match instruction {
		Instruction::BEQ |
		Instruction::BGE |
		Instruction::BGEU |
		Instruction::BLT |
		Instruction::BLTU |
		Instruction::BNE => InstructionFormat::B,
		Instruction::CSRRC |
		Instruction::CSRRCI |
		Instruction::CSRRS |
		Instruction::CSRRSI |
		Instruction::CSRRW |
		Instruction::CSRRWI => InstructionFormat::C,
		Instruction::ADDI |
		Instruction::ADDIW |
		Instruction::ANDI |
		Instruction::JALR |
		Instruction::LB |
		Instruction::LBU |
		Instruction::LD |
		Instruction::LH |
		Instruction::LHU |
		Instruction::LW |
		Instruction::LWU |
		Instruction::ORI |
		Instruction::SLLI |
		Instruction::SLLIW |
		Instruction::SLTI |
		Instruction::SLTIU |
		Instruction::SRLI |
		Instruction::SRLIW |
		Instruction::SRAI |
		Instruction::SRAIW |
		Instruction::XORI => InstructionFormat::I,
		Instruction::JAL => InstructionFormat::J,
		Instruction::FENCE => InstructionFormat::O,
		Instruction::ADD |
		Instruction::ADDW |
		Instruction::AMOSWAPW |
		Instruction::AND |
		Instruction::DIV |
		Instruction::DIVU |
		Instruction::DIVUW |
		Instruction::DIVW |
		Instruction::ECALL |
		Instruction::MRET |
		Instruction::MUL |
		Instruction::MULH |
		Instruction::MULHU |
		Instruction::MULHSU |
		Instruction::MULW |
		Instruction::OR |
		Instruction::REM |
		Instruction::REMU |
		Instruction::REMUW |
		Instruction::REMW |
		Instruction::SUB |
		Instruction::SUBW |
		Instruction::SFENCEVMA |
		Instruction::SLL |
		Instruction::SLLW |
		Instruction::SLT |
		Instruction::SLTU |
		Instruction::SRA |
		Instruction::SRAW |
		Instruction::SRET |
		Instruction::SRL |
		Instruction::SRLW |
		Instruction::URET |
		Instruction::XOR => InstructionFormat::R,
		Instruction::SB |
		Instruction::SD |
		Instruction::SH |
		Instruction::SW => InstructionFormat::S,
		Instruction::AUIPC |
		Instruction::LUI => InstructionFormat::U
	}
}

impl Cpu {
	pub fn new(terminal: Box<dyn Terminal>) -> Self {
		let mut cpu = Cpu {
			clock: 0,
			xlen: Xlen::Bit64,
			privilege_mode: PrivilegeMode::Machine,
			x: [0; 32],
			pc: 0,
			csr: [0; CSR_CAPACITY],
			mmu: Mmu::new(Xlen::Bit64, terminal)
		};
		cpu.csr[CSR_SSTATUS_ADDRESS as usize] = 0x200000000;
		cpu
	}

	// Five public methods for setting up from outside

	pub fn store_raw(&mut self, address: u64, value: u8) {
		self.mmu.store_raw(address, value);
	}

	pub fn update_pc(&mut self, value: u64) {
		self.pc = value;
	}

	pub fn update_xlen(&mut self, xlen: Xlen) {
		self.xlen = xlen.clone();
		self.mmu.update_xlen(xlen.clone());
	}

	pub fn setup_memory(&mut self, capacity: u64) {
		self.mmu.init_memory(capacity);
	}

	pub fn setup_filesystem(&mut self, data: Vec<u8>) {
		self.mmu.init_disk(data);
	}

	// One public methods for running riscv-tests

	pub fn load_word_raw(&mut self, address: u64) -> u32 {
		self.mmu.load_word_raw(address)
	}

	//

	pub fn tick(&mut self) {
		match self.tick_operate() {
			Ok(()) => {},
			Err(e) => self.handle_exception(e)
		}
		self.mmu.tick();
		self.handle_interrupt();
		self.clock = self.clock.wrapping_add(1);
	}

	// @TODO: Rename
	fn tick_operate(&mut self) -> Result<(), Trap> {
		let word = match self.fetch() {
			Ok(word) => word,
			Err(e) => return Err(e)
		};
		let instruction = self.decode(word);
		self.operate(word, instruction)
	}

	fn handle_interrupt(&mut self) {
		match self.mmu.detect_interrupt() {
			InterruptType::None => {},
			InterruptType::KeyInput => {
				match self.handle_trap(Trap {
					trap_type: TrapType::SupervisorExternalInterrupt,
					value: self.pc // dummy
				}, true) {
					true => {
						self.mmu.reset_uart_interrupting();
						self.mmu.reset_interrupt();
					},
					false => {}
				};
			},
			InterruptType::Timer => {
				match self.handle_trap(Trap {
					trap_type: TrapType::SupervisorSoftwareInterrupt,
					value: self.pc // dummy
				}, true) {
					true => {
						self.mmu.reset_clint_interrupting();
						self.mmu.reset_interrupt();
					},
					false => {}
				};
			},
			InterruptType::Virtio => {
				match self.handle_trap(Trap {
					trap_type: TrapType::SupervisorExternalInterrupt,
					value: self.pc // dummy
				}, true) {
					true => {
						self.mmu.handle_disk_access();
						self.mmu.reset_disk_interrupting();
						self.mmu.reset_interrupt();
					},
					false => {}
				};
			}
		};
	}

	fn handle_exception(&mut self, exception: Trap) {
		self.handle_trap(exception, false);
	}

	fn handle_trap(&mut self, trap: Trap, is_interrupt: bool) -> bool{
		let current_privilege_encoding = get_privilege_encoding(&self.privilege_mode) as u64;
		let cause = get_trap_cause(&trap, &self.xlen);

		// @TODO: Check if this logic is correct
		let mdeleg = match is_interrupt {
			true => self.csr[CSR_MIDELEG_ADDRESS as usize],
			false => self.csr[CSR_MEDELEG_ADDRESS as usize]
		};
		let sdeleg = match is_interrupt {
			true => self.csr[CSR_SIDELEG_ADDRESS as usize],
			false => self.csr[CSR_SEDELEG_ADDRESS as usize]
		};
		let pos = cause & 0xffff;
		let new_privilege_mode = match ((mdeleg >> pos) & 1) == 0 {
			true => PrivilegeMode::Machine,
			false => match ((sdeleg >> pos) & 1) == 0 {
				true => PrivilegeMode::Supervisor,
				false => PrivilegeMode::User
			}
		};

		// @TODO: Which we should do, dispose or pend, if trap is disabled?
		// Disposing so far.

		let status = match new_privilege_mode {
			PrivilegeMode::Machine => self.csr[CSR_MSTATUS_ADDRESS as usize],
			PrivilegeMode::Supervisor => self.csr[CSR_SSTATUS_ADDRESS as usize],
			PrivilegeMode::User => self.csr[CSR_USTATUS_ADDRESS as usize],
			PrivilegeMode::Reserved => panic!(),
		};

		let mie = (status >> 3) & 1;
		let sie = (status >> 1) & 1;
		let uie = status & 1;

		if is_interrupt {
			let interrupt_privilege_mode = get_interrupt_privilege_mode(&trap);
			let interrupt_privilege_encoding = get_privilege_encoding(&interrupt_privilege_mode) as u64;
			match new_privilege_mode {
				PrivilegeMode::Machine => {
					if mie == 0 {
						return false;
					}
				},
				PrivilegeMode::Supervisor => {
					if sie == 0 {
						return false;
					}
				},
				PrivilegeMode::User => {
					if uie == 0 {
						return false;
					}
				},
				PrivilegeMode::Reserved => panic!()
			};
			if current_privilege_encoding > interrupt_privilege_encoding {
				return false;
			}
		}

		self.privilege_mode = new_privilege_mode;
		self.mmu.update_privilege_mode(self.privilege_mode.clone());
		let csr_epc_address = match self.privilege_mode {
			PrivilegeMode::Machine => CSR_MEPC_ADDRESS,
			PrivilegeMode::Supervisor => CSR_SEPC_ADDRESS,
			PrivilegeMode::User => CSR_UEPC_ADDRESS,
			PrivilegeMode::Reserved => panic!()
		};
		let csr_cause_address = match self.privilege_mode {
			PrivilegeMode::Machine => CSR_MCAUSE_ADDRESS,
			PrivilegeMode::Supervisor => CSR_SCAUSE_ADDRESS,
			PrivilegeMode::User => CSR_UCAUSE_ADDRESS,
			PrivilegeMode::Reserved => panic!()
		};
		let csr_tval_address = match self.privilege_mode {
			PrivilegeMode::Machine => CSR_MTVAL_ADDRESS,
			PrivilegeMode::Supervisor => CSR_STVAL_ADDRESS,
			PrivilegeMode::User => CSR_UTVAL_ADDRESS,
			PrivilegeMode::Reserved => panic!()
		};
		let csr_tvec_address = match self.privilege_mode {
			PrivilegeMode::Machine => CSR_MTVEC_ADDRESS,
			PrivilegeMode::Supervisor => CSR_STVEC_ADDRESS,
			PrivilegeMode::User => CSR_UTVEC_ADDRESS,
			PrivilegeMode::Reserved => panic!()
		};

		self.csr[csr_epc_address as usize] = match is_interrupt {
			true => self.pc, // @TODO: remove this hack
			false => self.pc.wrapping_sub(4)
		};
		self.csr[csr_cause_address as usize] = cause;
		self.csr[csr_tval_address as usize] = trap.value;
		self.pc = self.csr[csr_tvec_address as usize];

		match self.privilege_mode {
			PrivilegeMode::Machine => {
				let status = self.csr[CSR_MSTATUS_ADDRESS as usize];
				let mie = (status >> 3) & 1;
				// clear MIE[3], override MPIE[7] with MIE[3], override MPP[12:11] with current privilege encoding
				let new_status = (status & !0x1888) | (mie << 7) | (current_privilege_encoding << 11);
				self.csr[CSR_MSTATUS_ADDRESS as usize] = new_status;
			},
			PrivilegeMode::Supervisor => {
				let status = self.csr[CSR_SSTATUS_ADDRESS as usize];
				let sie = (status >> 1) & 1;
				// clear SIE[1], override SPIE[5] with SIE[1], override SPP[8] with current privilege encoding
				let new_status = (status & !0x122) | (sie << 5) | ((current_privilege_encoding & 1) << 8);
				self.csr[CSR_SSTATUS_ADDRESS as usize] = new_status;
			},
			PrivilegeMode::User => {
				panic!("Not implemenete yet");
			},
			PrivilegeMode::Reserved => panic!() // shouldn't happen
		};
		true
	}

	fn fetch(&mut self) -> Result<u32, Trap> {
		let word = match self.mmu.fetch_word(self.pc) {
			Ok(word) => word,
			Err(e) => {
				self.pc = self.pc.wrapping_add(4);
				return Err(e);
			}
		};
		// @TODO: Should I increment pc after operating an instruction because
		// some of the instruction operations need the address of the instruction?
		self.pc = self.pc.wrapping_add(4);
		Ok(word)
	}

	fn has_csr_access_privilege(&self, address: u16) -> bool {
		let privilege = (address >> 8) & 0x3; // the lowest privilege level that can access the CSR
		privilege as u8 <= get_privilege_encoding(&self.privilege_mode)
	}

	fn read_csr(&mut self, address: u16) -> Result<u64, Trap> {
		match self.has_csr_access_privilege(address) {
			true => Ok(self.csr[address as usize]),
			false => Err(Trap {
				trap_type: TrapType::IllegalInstruction,
				value: self.pc.wrapping_sub(4) // @TODO: Is this always correct?
			})
		}
	}

	fn write_csr(&mut self, address: u16, value: u64) -> Result<(), Trap> {
		// println!("PC:{:X} Privilege mode:{}", self.pc.wrapping_sub(4), _get_privilege_mode_name(&self.privilege_mode));
		// println!("CSR:{:X} Value:{:X}", address, value);
		match self.has_csr_access_privilege(address) {
			true => {
				/*
				// Checking writability fails some tests so disabling so far
				let read_only = ((address >> 10) & 0x3) == 0x3;
				if read_only {
					return Err(Exception::IllegalInstruction);
				}
				*/
				self.csr[address as usize] = value;
				if address == CSR_SATP_ADDRESS {
					self.update_addressing_mode(value);
				}
				Ok(())
			},
			false => Err(Trap {
				trap_type: TrapType::IllegalInstruction,
				value: self.pc.wrapping_sub(4) // @TODO: Is this always correct?
			})
		}
	}

	fn update_addressing_mode(&mut self, value: u64) {
		let addressing_mode = match self.xlen {
			Xlen::Bit32 => match value & 0x80000000 {
				0 => AddressingMode::None,
				_ => AddressingMode::SV32
			},
			Xlen::Bit64 => match value >> 60 {
				0 => AddressingMode::None,
				8 => AddressingMode::SV39,
				9 => AddressingMode::SV48,
				_ => {
					println!("Unknown addressing_mode {:X}", value >> 60);
					panic!();
				}
			}
		};
		let ppn = match self.xlen {
			Xlen::Bit32 => value & 0x3fffff,
			Xlen::Bit64 => value & 0xfffffffffff
		};
		self.mmu.update_addressing_mode(addressing_mode);
		self.mmu.update_ppn(ppn);
	}

	// @TODO: Rename to better name?
	fn sign_extend(&self, value: i64) -> i64 {
		match self.xlen {
			Xlen::Bit32 => (match value & 0x80000000 {
				0x80000000 => (value as u64) | 0xffffffff00000000,
				_ => (value as u64) & 0xffffffff
			}) as i64,
			Xlen::Bit64 => value
		}
	}

	// @TODO: Rename to better name?
	fn unsigned_data(&self, value: i64) -> u64 {
		match self.xlen {
			Xlen::Bit32 => (value as u64) & 0xffffffff,
			Xlen::Bit64 => value as u64
		}
	}

	fn decode(&mut self, word: u32) -> Instruction {
		let opcode = word & 0x7f; // [6:0]
		let funct3 = (word >> 12) & 0x7; // [14:12]
		let funct7 = (word >> 25) & 0x7f; // [31:25]

		match opcode {
			0x03 => match funct3 {
				0 => Instruction::LB,
				1 => Instruction::LH,
				2 => Instruction::LW,
				3 => Instruction::LD,
				4 => Instruction::LBU,
				5 => Instruction::LHU,
				6 => Instruction::LWU,
				_ => {
					println!("Unknown funct3: {:03b}", funct3);
					self.dump_instruction(self.pc.wrapping_sub(4));
					panic!();
				}
			},
			0x0f => Instruction::FENCE,
			0x13 => match funct3 {
				0 => Instruction::ADDI,
				1 => Instruction::SLLI,
				2 => Instruction::SLTI,
				3 => Instruction::SLTIU,
				4 => Instruction::XORI,
				5 => match funct7 {
					0 => Instruction::SRLI,
					1 => Instruction::SRLI, // temporal workaround for xv6
					0x20 => Instruction::SRAI,
					_ => {
						println!("Unknown funct7: {:07b}", funct7);
						self.dump_instruction(self.pc.wrapping_sub(4));
						panic!();
					}
				}
				6 => Instruction::ORI,
				7 => Instruction::ANDI,
				_ => {
					println!("Unknown funct3: {:03b}", funct3);
					self.dump_instruction(self.pc.wrapping_sub(4));
					panic!();
				}
			},
			0x17 => Instruction::AUIPC,
			0x1b => match funct3 {
				0 => Instruction::ADDIW,
				1 => Instruction::SLLIW,
				5 => match funct7 {
					0 => Instruction::SRLIW,
					0x20 => Instruction::SRAIW,
					_ => {
						println!("Unknown funct7: {:07b}", funct7);
						self.dump_instruction(self.pc.wrapping_sub(4));
						panic!();
					}
				},
				_ => {
					println!("Unknown funct3: {:03b}", funct3);
					self.dump_instruction(self.pc.wrapping_sub(4));
					panic!();
				}
			},
			0x23 => match funct3 {
				0 => Instruction::SB,
				1 => Instruction::SH,
				2 => Instruction::SW,
				3 => Instruction::SD,
				_ => {
					println!("Unknown funct3: {:03b}", funct3);
					self.dump_instruction(self.pc.wrapping_sub(4));
					panic!();
				}
			},
			0x2f => match funct3 {
				2 => {
					match funct7 >> 2 {
						1 => Instruction::AMOSWAPW,
						_ => {
							println!("Unknown funct7: {:07b}", funct7);
							self.dump_instruction(self.pc.wrapping_sub(4));
							panic!();
						}
					}
				},
				_ => {
					println!("Unknown funct3: {:03b}", funct3);
					self.dump_instruction(self.pc.wrapping_sub(4));
					panic!();
				}
			}
			0x33 => match funct3 {
				0 => match funct7 {
					0 => Instruction::ADD,
					1 => Instruction::MUL,
					0x20 => Instruction::SUB,
					_ => {
						println!("Unknown funct7: {:07b}", funct7);
						self.dump_instruction(self.pc.wrapping_sub(4));
						panic!();
					}
				},
				1 => match funct7 {
					0 => Instruction::SLL,
					1 => Instruction::MULH,
					_ => {
						println!("Unknown funct7: {:07b}", funct7);
						self.dump_instruction(self.pc.wrapping_sub(4));
						panic!();
					}
				},
				2 => match funct7 {
					0 => Instruction::SLT,
					1 => Instruction::MULHSU,
					_ => {
						println!("Unknown funct7: {:07b}", funct7);
						self.dump_instruction(self.pc.wrapping_sub(4));
						panic!();
					}
				},
				3 => match funct7 {
					0 => Instruction::SLTU,
					1 => Instruction::MULHU,
					_ => {
						println!("Unknown funct7: {:07b}", funct7);
						self.dump_instruction(self.pc.wrapping_sub(4));
						panic!();
					}
				},
				4 => match funct7 {
					0 => Instruction::XOR,
					1 => Instruction::DIV,
					_ => {
						println!("Unknown funct7: {:07b}", funct7);
						self.dump_instruction(self.pc.wrapping_sub(4));
						panic!();
					}
				},
				5 => match funct7 {
					0 => Instruction::SRL,
					1 => Instruction::DIVU,
					0x20 => Instruction::SRA,
					_ => {
						println!("Unknown funct7: {:07b}", funct7);
						self.dump_instruction(self.pc.wrapping_sub(4));
						panic!();
					}
				},
				6 => match funct7 {
					0 => Instruction::OR,
					1 => Instruction::REM,
					_ => {
						println!("Unknown funct7: {:07b}", funct7);
						self.dump_instruction(self.pc.wrapping_sub(4));
						panic!();
					}
				},
				7 => match funct7 {
					0 => Instruction::AND,
					1 => Instruction::REMU,
					_ => {
						println!("Unknown funct7: {:07b}", funct7);
						self.dump_instruction(self.pc.wrapping_sub(4));
						panic!();
					}
				},
				_ => {
					println!("Unknown funct3: {:03b}", funct3);
					self.dump_instruction(self.pc.wrapping_sub(4));
					panic!();
				}
			},
			0x37 => Instruction::LUI,
			0x3b => match funct3 {
				0 => match funct7 {
					0 => Instruction::ADDW,
					1 => Instruction::MULW,
					0x20 => Instruction::SUBW,
					_ => {
						println!("Unknown funct7: {:07b}", funct7);
						self.dump_instruction(self.pc.wrapping_sub(4));
						panic!();
					}
				},
				1 => Instruction::SLLW,
				4 => Instruction::DIVW,
				5 => match funct7 {
					0 => Instruction::SRLW,
					1 => Instruction::DIVUW,
					0x20 => Instruction::SRAW,
					_ => {
						println!("Unknown funct7: {:07b}", funct7);
						self.dump_instruction(self.pc.wrapping_sub(4));
						panic!();
					}
				},
				6 => Instruction::REMW,
				7 => Instruction::REMUW,
				_ => {
					println!("funct3: {:03b} is not supported yet", funct3);
					self.dump_instruction(self.pc.wrapping_sub(4));
					panic!();
				}
			},
			0x63 => match funct3 {
				0 => Instruction::BEQ,
				1 => Instruction::BNE,
				4 => Instruction::BLT,
				5 => Instruction::BGE,
				6 => Instruction::BLTU,
				7 => Instruction::BGEU,
				_ => {
					println!("Branch funct3: {:03b} is not supported yet", funct3);
					self.dump_instruction(self.pc.wrapping_sub(4));
					panic!();
				}
			},
			0x67 => Instruction::JALR,
			0x6f => Instruction::JAL,
			0x73 => match funct3 {
				0 => {
					match funct7 {
						9 => Instruction::SFENCEVMA,
						_ => match word {
							0x00000073 => Instruction::ECALL,
							0x00200073 => Instruction::URET,
							0x10200073 => Instruction::SRET,
							0x30200073 => Instruction::MRET,
							_ => {
								println!("Priviledged instruction 0x{:08x} is not supported yet", word);
								self.dump_instruction(self.pc.wrapping_sub(4));
								panic!();
							}
						}
					}
				}
				1 => Instruction::CSRRW,
				2 => Instruction::CSRRS,
				3 => Instruction::CSRRC,
				5 => Instruction::CSRRWI,
				6 => Instruction::CSRRSI,
				7 => Instruction::CSRRCI,
				_ => {
					println!("CSR funct3: {:03b} is not supported yet", funct3);
					self.dump_instruction(self.pc.wrapping_sub(4));
					panic!();
				}
			},
			_ => {
				println!("Unknown Instruction type.");
				self.dump_instruction(self.pc.wrapping_sub(4));
				panic!();
			}
		}
	}

	fn operate(&mut self, word: u32, instruction: Instruction) -> Result<(), Trap> {
		let instruction_format = get_instruction_format(&instruction);
		match instruction_format {
			InstructionFormat::B => {
				let rs1 = (word & 0x000f8000) >> 15; // [19:15]
				let rs2 = (word & 0x01f00000) >> 20; // [24:20]
				let imm = (
					match word & 0x80000000 { // imm[31:12] = [31]
						0x80000000 => 0xfffff800,
						_ => 0
					} |
					((word & 0x00000080) << 4) | // imm[11] = [7]
					((word & 0x7e000000) >> 20) | // imm[10:5] = [30:25]
					((word & 0x00000f00) >> 7) // imm[4:1] = [11:8]
				) as i32 as i64 as u64;
				// println!("Compare {:X} {:X}", self.x[rs1 as usize], self.x[rs2 as usize]);
				match instruction {
					Instruction::BEQ => {
						if self.sign_extend(self.x[rs1 as usize]) == self.sign_extend(self.x[rs2 as usize]) {
							self.pc = self.pc.wrapping_sub(4).wrapping_add(imm);
						}
					},
					Instruction::BGE => {
						if self.sign_extend(self.x[rs1 as usize]) >= self.sign_extend(self.x[rs2 as usize]) {
							self.pc = self.pc.wrapping_sub(4).wrapping_add(imm);
						}
					},
					Instruction::BGEU => {
						if self.unsigned_data(self.x[rs1 as usize]) >= self.unsigned_data(self.x[rs2 as usize]) {
							self.pc = self.pc.wrapping_sub(4).wrapping_add(imm);
						}
					},
					Instruction::BLT => {
						if self.sign_extend(self.x[rs1 as usize]) < self.sign_extend(self.x[rs2 as usize]) {
							self.pc = self.pc.wrapping_sub(4).wrapping_add(imm);
						}
					},
					Instruction::BLTU => {
						if self.unsigned_data(self.x[rs1 as usize]) < self.unsigned_data(self.x[rs2 as usize]) {
							self.pc = self.pc.wrapping_sub(4).wrapping_add(imm);
						}
					},
					Instruction::BNE => {
						if self.sign_extend(self.x[rs1 as usize]) != self.sign_extend(self.x[rs2 as usize]) {
							self.pc = self.pc.wrapping_sub(4).wrapping_add(imm);
						}
					},
					_ => {
						println!("{}", get_instruction_name(&instruction).to_owned() + " instruction is not supported yet.");
						self.dump_instruction(self.pc.wrapping_sub(4));
						panic!();
					}
				};
			},
			InstructionFormat::C => {
				let csr = ((word >> 20) & 0xfff) as u16; // [31:20];
				let rs = (word >> 15) & 0x1f; // [19:15];
				let rd = (word >> 7) & 0x1f; // [11:7];
				// @TODO: Don't write if csr bits aren't writable
				match instruction {
					Instruction::CSRRC => {
						let data = match self.read_csr(csr) {
							Ok(data) => data,
							Err(e) => return Err(e)
						};
						let tmp = self.x[rs as usize];
						self.x[rd as usize] = self.sign_extend(data as i64);
						self.x[0] = 0; // hard-wired zero
						match self.write_csr(csr, (self.x[rd as usize] & !tmp) as u64) {
							Ok(()) => {},
							Err(e) => return Err(e)
						};
					},
					Instruction::CSRRCI => {
						let data = match self.read_csr(csr) {
							Ok(data) => data,
							Err(e) => return Err(e)
						};
						self.x[rd as usize] = self.sign_extend(data as i64);
						self.x[0] = 0; // hard-wired zero
						match self.write_csr(csr, (self.x[rd as usize] as u64) & !(rs as u64)) {
							Ok(()) => {},
							Err(e) => return Err(e)
						};
					},
					Instruction::CSRRS => {
						let data = match self.read_csr(csr) {
							Ok(data) => data,
							Err(e) => return Err(e)
						};
						let tmp = self.x[rs as usize];
						self.x[rd as usize] = self.sign_extend(data as i64);
						self.x[0] = 0; // hard-wired zero
						match self.write_csr(csr, self.unsigned_data(self.x[rd as usize] | tmp)) {
							Ok(()) => {},
							Err(e) => return Err(e)
						};
					},
					Instruction::CSRRSI => {
						let data = match self.read_csr(csr) {
							Ok(data) => data,
							Err(e) => return Err(e)
						};
						self.x[rd as usize] = self.sign_extend(data as i64);
						self.x[0] = 0; // hard-wired zero
						match self.write_csr(csr, self.unsigned_data((self.x[rd as usize] as u64 | rs as u64) as i64)) {
							Ok(()) => {},
							Err(e) => return Err(e)
						};
					},
					Instruction::CSRRW => {
						let data = match self.read_csr(csr) {
							Ok(data) => data,
							Err(e) => return Err(e)
						};
						let tmp = self.x[rs as usize];
						self.x[rd as usize] = self.sign_extend(data as i64);
						self.x[0] = 0; // hard-wired zero
						match self.write_csr(csr, self.unsigned_data(tmp)) {
							Ok(()) => {},
							Err(e) => return Err(e)
						};
					},
					Instruction::CSRRWI => {
						let data = match self.read_csr(csr) {
							Ok(data) => data,
							Err(e) => return Err(e)
						};
						self.x[rd as usize] = self.sign_extend(data as i64);
						self.x[0] = 0; // hard-wired zero
						match self.write_csr(csr, rs as u64) {
							Ok(()) => {},
							Err(e) => return Err(e)
						};
					},
					_ => {
						println!("{}", get_instruction_name(&instruction).to_owned() + " instruction is not supported yet.");
						self.dump_instruction(self.pc.wrapping_sub(4));
						panic!();
					}
				};
			},
			InstructionFormat::I => {
				let rd = (word >> 7) & 0x1f; // [11:7]
				let rs1 = (word >> 15) & 0x1f; // [19:15]
				let imm = (
					match word & 0x80000000 { // imm[31:11] = [31]
						0x80000000 => 0xfffff800,
						_ => 0
					} |
					((word >> 20) & 0x000007ff) // imm[10:0] = [30:20]
				) as i32 as i64;
				match instruction {
					Instruction::ADDI => {
						self.x[rd as usize] = self.sign_extend(self.x[rs1 as usize].wrapping_add(imm));
					},
					Instruction::ADDIW => {
						self.x[rd as usize] = self.x[rs1 as usize].wrapping_add(imm) as i32 as i64;
					},
					Instruction::ANDI => {
						self.x[rd as usize] = self.sign_extend(self.x[rs1 as usize] & imm);
					},
					Instruction::JALR => {
						let tmp = self.sign_extend(self.pc as i64);
						self.pc = (self.x[rs1 as usize] as u64).wrapping_add(imm as u64);
						self.x[rd as usize] = tmp;
					},
					Instruction::LB => {
						self.x[rd as usize] = match self.mmu.load(self.x[rs1 as usize].wrapping_add(imm) as u64) {
							Ok(data) => data as i8 as i64,
							Err(e) => return Err(e)
						};
					},
					Instruction::LBU => {
						self.x[rd as usize] = match self.mmu.load(self.x[rs1 as usize].wrapping_add(imm) as u64) {
							Ok(data) => data as i64,
							Err(e) => return Err(e)
						};
					},
					Instruction::LD => {
						self.x[rd as usize] = match self.mmu.load_doubleword(self.x[rs1 as usize].wrapping_add(imm) as u64) {
							Ok(data) => data as i64,
							Err(e) => return Err(e)
						};
					},
					Instruction::LH => {
						self.x[rd as usize] = match self.mmu.load_halfword(self.x[rs1 as usize].wrapping_add(imm) as u64) {
							Ok(data) => data as i16 as i64,
							Err(e) => return Err(e)
						};
					},
					Instruction::LHU => {
						self.x[rd as usize] = match self.mmu.load_halfword(self.x[rs1 as usize].wrapping_add(imm) as u64) {
							Ok(data) => data as i64,
							Err(e) => return Err(e)
						};
					},
					Instruction::LW => {
						self.x[rd as usize] = match self.mmu.load_word(self.x[rs1 as usize].wrapping_add(imm) as u64) {
							Ok(data) => data as i32 as i64,
							Err(e) => return Err(e)
						};
					},
					Instruction::LWU => {
						self.x[rd as usize] = match self.mmu.load_word(self.x[rs1 as usize].wrapping_add(imm) as u64) {
							Ok(data) => data as i64,
							Err(e) => return Err(e)
						};
					},
					Instruction::ORI => {
						self.x[rd as usize] = self.sign_extend(self.x[rs1 as usize] | imm);
					},
					Instruction::SLLI => {
						let shamt = (imm & match self.xlen {
							Xlen::Bit32 => 0x1f,
							Xlen::Bit64 => 0x3f
						}) as u32;
						self.x[rd as usize] = self.sign_extend(self.x[rs1 as usize] << shamt);
					},
					Instruction::SLLIW => {
						let shamt = (imm as u32) & 0x1f;
						self.x[rd as usize] = (self.x[rs1 as usize] << shamt) as i32 as i64;
					},
					Instruction::SLTI => {
						self.x[rd as usize] = match self.x[rs1 as usize] < imm {
							true => 1,
							false => 0
						}
					},
					Instruction::SLTIU => {
						self.x[rd as usize] = match self.unsigned_data(self.x[rs1 as usize]) < self.unsigned_data(imm) {
							true => 1,
							false => 0
						}
					},
					Instruction::SRAI => {
						let shamt = (imm & match self.xlen {
							Xlen::Bit32 => 0x1f,
							Xlen::Bit64 => 0x3f
						}) as u32;
						self.x[rd as usize] = self.sign_extend(self.x[rs1 as usize] >> shamt);
					},
					Instruction::SRAIW => {
						let shamt = (imm as u32) & 0x1f;
						self.x[rd as usize] = ((self.x[rs1 as usize] as i32) >> shamt) as i32 as i64;
					},
					Instruction::SRLI => {
						let shamt = (imm & match self.xlen {
							Xlen::Bit32 => 0x1f,
							Xlen::Bit64 => 0x3f
						}) as u32;
						self.x[rd as usize] = self.sign_extend((self.unsigned_data(self.x[rs1 as usize]) >> shamt) as i64);
					},
					Instruction::SRLIW => {
						let shamt = (imm as u32) & 0x1f;
						self.x[rd as usize] = ((self.x[rs1 as usize] as u32) >> shamt) as i32 as i64;
					},
					Instruction::XORI => {
						self.x[rd as usize] = self.sign_extend(self.x[rs1 as usize] ^ imm);
					},
					_ => {
						println!("{}", get_instruction_name(&instruction).to_owned() + " instruction is not supported yet.");
						self.dump_instruction(self.pc.wrapping_sub(4));
						panic!();
					}
				};
			},
			InstructionFormat::J => {
				let rd = (word >> 7) & 0x1f; // [11:7]
				let imm = (
					match word & 0x80000000 { // imm[31:20] = [31]
						0x80000000 => 0xfff00000,
						_ => 0
					} |
					(word & 0x000ff000) | // imm[19:12] = [19:12]
					((word & 0x00100000) >> 9) | // imm[11] = [20]
					((word & 0x7fe00000) >> 20) // imm[10:1] = [30:21]
				) as i32 as i64 as u64;
				match instruction {
					Instruction::JAL => {
						self.x[rd as usize] = self.sign_extend(self.pc as i64);
						self.pc = self.pc.wrapping_sub(4).wrapping_add(imm);
					},
					_ => {
						println!("{}", get_instruction_name(&instruction).to_owned() + " instruction is not supported yet.");
						self.dump_instruction(self.pc.wrapping_sub(4));
						panic!();
					}
				};
			},
			InstructionFormat::O => {
				match instruction {
					Instruction::FENCE => {
						// @TODO: Implement
					},
					_ => {
						println!("{}", get_instruction_name(&instruction).to_owned() + " instruction is not supported yet.");
						self.dump_instruction(self.pc.wrapping_sub(4));
						panic!();
					}
				};
			},
			InstructionFormat::R => {
				let rd = (word >> 7) & 0x1f; // [11:7]
				let rs1 = (word >> 15) & 0x1f; // [19:15]
				let rs2 = (word >> 20) & 0x1f; // [24:20]
				match instruction {
					Instruction::ADD => {
						self.x[rd as usize] = self.sign_extend(self.x[rs1 as usize].wrapping_add(self.x[rs2 as usize]));
					},
					Instruction::ADDW => {
						self.x[rd as usize] = self.x[rs1 as usize].wrapping_add(self.x[rs2 as usize]) as i32 as i64;
					},
					Instruction::AMOSWAPW => {
						let tmp = match self.mmu.load_word(self.unsigned_data(self.x[rs1 as usize])) {
							Ok(data) => data,
							Err(e) => return Err(e)
						};
						match self.mmu.store_word(self.unsigned_data(self.x[rs1 as usize]), self.x[rs2 as usize] as u32) {
							Ok(()) => {},
							Err(e) => return Err(e)
						};
						self.x[rd as usize] = tmp as i32 as i64;
					},
					Instruction::AND => {
						self.x[rd as usize] = self.sign_extend(self.x[rs1 as usize] & self.x[rs2 as usize]);
					},
					Instruction::DIV => {
						self.x[rd as usize] = match self.x[rs2 as usize] {
							0 => -1,
							_ => self.sign_extend(self.x[rs1 as usize].wrapping_div(self.x[rs2 as usize]))
						};
					},
					Instruction::DIVU => {
						self.x[rd as usize] = match self.x[rs2 as usize] {
							0 => -1,
							_ => self.sign_extend(self.unsigned_data(self.x[rs1 as usize]).wrapping_div(self.unsigned_data(self.x[rs2 as usize])) as i64)
						};
					},
					Instruction::DIVUW => {
						self.x[rd as usize] = match self.x[rs2 as usize] {
							0 => -1,
							_ => (self.x[rs1 as usize] as u32).wrapping_div(self.x[rs2 as usize] as u32) as i32 as i64
						};
					},
					Instruction::DIVW => {
						self.x[rd as usize] = match self.x[rs2 as usize] {
							0 => -1,
							_ => self.sign_extend((self.x[rs1 as usize] as i32).wrapping_div(self.x[rs2 as usize] as i32) as i64)
						};
					},
					Instruction::ECALL => {
						let csr_epc_address = match self.privilege_mode {
							PrivilegeMode::User => CSR_UEPC_ADDRESS,
							PrivilegeMode::Supervisor => CSR_SEPC_ADDRESS,
							PrivilegeMode::Machine => CSR_MEPC_ADDRESS,
							PrivilegeMode::Reserved => panic!()
						};
						self.csr[csr_epc_address as usize] = self.pc.wrapping_sub(4);
						let exception_type = match self.privilege_mode {
							PrivilegeMode::User => TrapType::EnvironmentCallFromUMode,
							PrivilegeMode::Supervisor => TrapType::EnvironmentCallFromSMode,
							PrivilegeMode::Machine => TrapType::EnvironmentCallFromMMode,
							PrivilegeMode::Reserved => panic!()
						};
						return Err(Trap {
							trap_type: exception_type,
							value: self.pc.wrapping_sub(4)
						});
					},
					Instruction::MRET |
					Instruction::SRET |
					Instruction::URET => {
						// @TODO: Throw error if higher privilege return instruction is executed
						// @TODO: Implement propertly
						let csr_epc_address = match instruction {
							Instruction::MRET => CSR_MEPC_ADDRESS,
							Instruction::SRET => CSR_SEPC_ADDRESS,
							Instruction::URET => CSR_UEPC_ADDRESS,
							_ => panic!() // shouldn't happen
						};
						self.pc = match self.read_csr(csr_epc_address) {
							Ok(data) => data,
							Err(e) => return Err(e)
						};
						match instruction {
							Instruction::MRET => {
								let status = self.csr[CSR_MSTATUS_ADDRESS as usize];
								let mpie = (status >> 7) & 1;
								let mpp = (status >> 11) & 0x3;
								// Override MIE[3] with MPIE[7], set MPIE[7] to 1, set MPP[12:11] to 0
								let new_status = (status & !0x1888) | (mpie << 3) | (1 << 7);
								self.csr[CSR_MSTATUS_ADDRESS as usize] = new_status;
								self.privilege_mode = match mpp {
									0 => PrivilegeMode::User,
									1 => PrivilegeMode::Supervisor,
									3 => PrivilegeMode::Machine,
									_ => panic!() // Shouldn't happen
								};
							},
							Instruction::SRET => {
								let status = self.csr[CSR_SSTATUS_ADDRESS as usize];
								let spie = (status >> 5) & 1;
								let spp = (status >> 8) & 1;
								// Override SIE[1] with SPIE[5], set SPIE[5] to 1, set SPP[8] to 0
								let new_status = (status & !0x122) | (spie << 1) | (1 << 5);
								self.csr[CSR_SSTATUS_ADDRESS as usize] = new_status;
								self.privilege_mode = match spp {
									0 => PrivilegeMode::User,
									1 => PrivilegeMode::Supervisor,
									_ => panic!() // Shouldn't happen
								};
							},
							Instruction::URET => {
								panic!("Not implemented yet.");
							},
							_ => panic!() // shouldn't happen
						};
						self.mmu.update_privilege_mode(self.privilege_mode.clone());
					},
					Instruction::MUL => {
						self.x[rd as usize] = self.sign_extend(self.x[rs1 as usize].wrapping_mul(self.x[rs2 as usize]));
					},
					Instruction::MULH => {
						self.x[rd as usize] = match self.xlen {
							Xlen::Bit32 => {
								self.sign_extend((self.x[rs1 as usize] * self.x[rs2 as usize]) >> 32)
							},
							Xlen::Bit64 => {
								((self.x[rs1 as usize] as i128) * (self.x[rs2 as usize] as i128) >> 64) as i64
							}
						};
					},
					Instruction::MULHU => {
						self.x[rd as usize] = match self.xlen {
							Xlen::Bit32 => {
								self.sign_extend((((self.x[rs1 as usize] as u32 as u64) * (self.x[rs2 as usize] as u32 as u64)) >> 32) as i64)
							},
							Xlen::Bit64 => {
								((self.x[rs1 as usize] as u64 as u128).wrapping_mul(self.x[rs2 as usize] as u64 as u128) >> 64) as i64
							}
						};
					},
					Instruction::MULHSU => {
						self.x[rd as usize] = match self.xlen {
							Xlen::Bit32 => {
								self.sign_extend(((self.x[rs1 as usize] as i64).wrapping_mul(self.x[rs2 as usize] as u32 as i64) >> 32) as i64)
							},
							Xlen::Bit64 => {
								((self.x[rs1 as usize] as u128).wrapping_mul(self.x[rs2 as usize] as u64 as u128) >> 64) as i64
							}
						};
					},
					Instruction::MULW => {
						self.x[rd as usize] = self.sign_extend((self.x[rs1 as usize] as i32).wrapping_mul(self.x[rs2 as usize] as i32) as i64);
					},
					Instruction::OR => {
						self.x[rd as usize] = self.sign_extend(self.x[rs1 as usize] | self.x[rs2 as usize]);
					},
					Instruction::REM => {
						self.x[rd as usize] = match self.x[rs2 as usize] {
							0 => self.x[rs1 as usize],
							_ => self.sign_extend(self.x[rs1 as usize].wrapping_rem(self.x[rs2 as usize]))
						};
					},
					Instruction::REMU => {
						self.x[rd as usize] = match self.x[rs2 as usize] {
							0 => self.x[rs1 as usize],
							_ => self.sign_extend(self.unsigned_data(self.x[rs1 as usize]).wrapping_rem(self.unsigned_data(self.x[rs2 as usize])) as i64)
						};
					},
					Instruction::REMUW => {
						self.x[rd as usize] = match self.x[rs2 as usize] {
							0 => self.x[rs1 as usize],
							_ => self.sign_extend((self.x[rs1 as usize] as u32).wrapping_rem(self.x[rs2 as usize] as u32) as i32 as i64)
						};
					},
					Instruction::REMW => {
						self.x[rd as usize] = match self.x[rs2 as usize] {
							0 => self.x[rs1 as usize],
							_ => self.sign_extend((self.x[rs1 as usize] as i32).wrapping_rem((self.x[rs2 as usize]) as i32) as i64)
						};
					},
					Instruction::SFENCEVMA => {
						// @TODO: Implement
					},
					Instruction::SUB => {
						self.x[rd as usize] = self.sign_extend(self.x[rs1 as usize].wrapping_sub(self.x[rs2 as usize]));
					},
					Instruction::SUBW => {
						self.x[rd as usize] = self.x[rs1 as usize].wrapping_sub(self.x[rs2 as usize]) as i32 as i64;
					},
					Instruction::SLL => {
						self.x[rd as usize] = self.sign_extend(self.x[rs1 as usize].wrapping_shl(self.x[rs2 as usize] as u32));
					},
					Instruction::SLLW => {
						self.x[rd as usize] = (self.x[rs1 as usize] as u32).wrapping_shl(self.x[rs2 as usize] as u32) as i32 as i64;
					},
					Instruction::SLT => {
						self.x[rd as usize] = match self.x[rs1 as usize] < self.x[rs2 as usize] {
							true => 1,
							false => 0
						}
					},
					Instruction::SLTU => {
						self.x[rd as usize] = match self.unsigned_data(self.x[rs1 as usize]) < self.unsigned_data(self.x[rs2 as usize]) {
							true => 1,
							false => 0
						}
					},
					Instruction::SRA => {
						self.x[rd as usize] = self.sign_extend(self.x[rs1 as usize].wrapping_shr(self.x[rs2 as usize] as u32));
					},
					Instruction::SRAW => {
						self.x[rd as usize] = (self.x[rs1 as usize] as i32).wrapping_shr(self.x[rs2 as usize] as u32) as i32 as i64;
					},
					Instruction::SRL => {
						self.x[rd as usize] = self.sign_extend(self.unsigned_data(self.x[rs1 as usize]).wrapping_shr(self.x[rs2 as usize] as u32) as i64);
					},
					Instruction::SRLW => {
						self.x[rd as usize] = (self.x[rs1 as usize] as u32).wrapping_shr(self.x[rs2 as usize] as u32) as i32 as i64;
					},
					Instruction::XOR => {
						self.x[rd as usize] = self.sign_extend(self.x[rs1 as usize] ^ self.x[rs2 as usize]);
					},
					_ => {
						println!("{}", get_instruction_name(&instruction).to_owned() + " instruction is not supported yet.");
						self.dump_instruction(self.pc.wrapping_sub(4));
						panic!();
					}
				};
			},
			InstructionFormat::S => {
				let rs1 = (word >> 15) & 0x1f; // [19:15]
				let rs2 = (word >> 20) & 0x1f; // [24:20]
				let imm = (
					match word & 0x80000000 {
						0x80000000 => 0xfffff000,
						_ => 0
					} | // imm[31:12] = [31]
					((word & 0xfe000000) >> 20) | // imm[11:5] = [31:25],
					((word & 0x00000f80) >> 7) // imm[4:0] = [11:7]
				) as i32 as i64;
				match instruction {
					Instruction::SB => {
						match self.mmu.store(self.x[rs1 as usize].wrapping_add(imm) as u64, self.x[rs2 as usize] as u8) {
							Ok(()) => {},
							Err(e) => return Err(e)
						};
					},
					Instruction::SH => {
						match self.mmu.store_halfword(self.x[rs1 as usize].wrapping_add(imm) as u64, self.x[rs2 as usize] as u16) {
							Ok(()) => {},
							Err(e) => return Err(e)
						};
					},
					Instruction::SW => {
						match self.mmu.store_word(self.x[rs1 as usize].wrapping_add(imm) as u64, self.x[rs2 as usize] as u32) {
							Ok(()) => {},
							Err(e) => return Err(e)
						};
					},
					Instruction::SD => {
						match self.mmu.store_doubleword(self.x[rs1 as usize].wrapping_add(imm) as u64, self.x[rs2 as usize] as u64) {
							Ok(()) => {},
							Err(e) => return Err(e)
						};
					},
					_ => {
						println!("{}", get_instruction_name(&instruction).to_owned() + " instruction is not supported yet.");
						self.dump_instruction(self.pc.wrapping_sub(4));
						panic!();
					}
				};
			},
			InstructionFormat::U => {
				let rd = (word >> 7) & 0x1f; // [11:7]
				let imm = (
					match word & 0x80000000 {
						0x80000000 => 0xffffffff00000000,
						_ => 0
					} | // imm[63:32] = [31]
					((word as u64) & 0xfffff000) // imm[31:12] = [31:12]
				) as u64;
				match instruction {
					Instruction::AUIPC => {
						self.x[rd as usize] = self.sign_extend(self.pc.wrapping_sub(4).wrapping_add(imm) as i64);
					},
					Instruction::LUI => {
						self.x[rd as usize] = imm as i64;
					}
					_ => {
						println!("{}", get_instruction_name(&instruction).to_owned() + " instruction is not supported yet.");
						self.dump_instruction(self.pc.wrapping_sub(4));
						panic!();
					}
				};
			}
		}
		self.x[0] = 0; // hard-wired zero
		Ok(())
	}

	fn dump_instruction(&mut self, address: u64) {
		let word = match self.mmu.load_word(address) {
			Ok(word) => word,
			Err(_e) => return // @TODO: What should we do if trap happens?
		};
		let pc = self.unsigned_data(address as i64);
		let opcode = word & 0x7f; // [6:0]
		println!("Pc:{:016x}, Opcode:{:07b}, Word:{:016x}", pc, opcode, word);
	}

	// For riscv-tests

	pub fn dump_current_instruction_to_terminal(&mut self) {
		// @TODO: Fetching can make a side effect,
		// for example updating page table entry or update peripheral hardware registers
		// by accessing them. How can we avoid it?
		let v_address = self.pc;
		let word = match self.mmu.fetch_word(v_address) {
			Ok(data) => data,
			Err(_e) => {
				let s = format!("PC:{:016x}, InstructionPageFault Trap!\n", v_address);
				self.put_bytes_to_terminal(s.as_bytes());
				return;
			}
		};
		let instruction = self.decode(word);
		let s = format!("PC:{:016x}, Word:{:016x}, Inst:{}\n",
			self.unsigned_data(v_address as i64),
			word, get_instruction_name(&instruction));
		self.put_bytes_to_terminal(s.as_bytes());
	}

	pub fn put_bytes_to_terminal(&mut self, bytes: &[u8]) {
		for i in 0..bytes.len() {
			self.mmu.put_uart_output(bytes[i]);
		}
	}
	
	// Wasm specific
	pub fn get_output(&mut self) -> u8 {
		self.mmu.get_uart_output()
	}

	pub fn put_input(&mut self, data: u8) {
		self.mmu.put_uart_input(data);
	}
}
