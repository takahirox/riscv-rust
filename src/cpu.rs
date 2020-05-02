use mmu::{AddressingMode, Mmu};
use terminal::Terminal;

const CSR_CAPACITY: usize = 4096;

const CSR_USTATUS_ADDRESS: u16 = 0x000;
const CSR_UIE_ADDRESS: u16 = 0x004;
const CSR_UTVEC_ADDRESS: u16 = 0x005;
const _CSR_USCRATCH_ADDRESS: u16 = 0x040;
const CSR_UEPC_ADDRESS: u16 = 0x041;
const CSR_UCAUSE_ADDRESS: u16 = 0x042;
const CSR_UTVAL_ADDRESS: u16 = 0x043;
const _CSR_UIP_ADDRESS: u16 = 0x044;
const CSR_SSTATUS_ADDRESS: u16 = 0x100;
const CSR_SEDELEG_ADDRESS: u16 = 0x102;
const CSR_SIDELEG_ADDRESS: u16 = 0x103;
const CSR_SIE_ADDRESS: u16 = 0x104;
const CSR_STVEC_ADDRESS: u16 = 0x105;
const _CSR_SSCRATCH_ADDRESS: u16 = 0x140;
const CSR_SEPC_ADDRESS: u16 = 0x141;
const CSR_SCAUSE_ADDRESS: u16 = 0x142;
const CSR_STVAL_ADDRESS: u16 = 0x143;
const CSR_SIP_ADDRESS: u16 = 0x144;
const CSR_SATP_ADDRESS: u16 = 0x180;
const CSR_MSTATUS_ADDRESS: u16 = 0x300;
const CSR_MISA_ADDRESS: u16 = 0x301;
const CSR_MEDELEG_ADDRESS: u16 = 0x302;
const CSR_MIDELEG_ADDRESS: u16 = 0x303;
const CSR_MIE_ADDRESS: u16 = 0x304;
const CSR_MTVEC_ADDRESS: u16 = 0x305;
const _CSR_MSCRATCH_ADDRESS: u16 = 0x340;
const CSR_MEPC_ADDRESS: u16 = 0x341;
const CSR_MCAUSE_ADDRESS: u16 = 0x342;
const CSR_MTVAL_ADDRESS: u16 = 0x343;
const CSR_MIP_ADDRESS: u16 = 0x344;
const _CSR_PMPCFG0_ADDRESS: u16 = 0x3a0;
const _CSR_PMPADDR0_ADDRESS: u16 = 0x3b0;
const _CSR_MCYCLE_ADDRESS: u16 = 0xb00;
const CSR_CYCLE_ADDRESS: u16 = 0xc00;
const CSR_TIME_ADDRESS: u16 = 0xc01;
const _CSR_INSERT_ADDRESS: u16 = 0xc02;
const _CSR_MHARTID_ADDRESS: u16 = 0xf14;

const MIP_MEIP: u64 = 0x800;
pub const MIP_MTIP: u64 = 0x080;
pub const MIP_MSIP: u64 = 0x008;
pub const MIP_SEIP: u64 = 0x200;
const MIP_STIP: u64 = 0x020;
const MIP_SSIP: u64 = 0x002;

pub struct Cpu {
	clock: u64,
	xlen: Xlen,
	privilege_mode: PrivilegeMode,
	wfi: bool,
	// using only lower 32bits of x, pc, and csr registers
	// for 32-bit mode
	x: [i64; 32],
	f: [f64; 32],
	pc: u64,
	csr: [u64; CSR_CAPACITY],
	mmu: Mmu,
	reservation: u64, // @TODO: Should support multiple address reservations
	is_reservation_set: bool,
	_dump_flag: bool
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
	ADDW,
	AMOADDD,
	AMOADDW,
	AMOANDD,
	AMOANDW,
	AMOMAXUW,
	AMOORD,
	AMOORW,
	AMOSWAPD,
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
	EBREAK,
	ECALL,
	FADDD,
	FCVTDL,
	FCVTDW,
	FCVTDWU,
	FCVTDS,
	FCVTSD,
	FCVTWD,
	FDIVD,
	FENCE,
	FEQD,
	FLD,
	FLED,
	FLTD,
	FMULD,
	FLW,
	FMADDD,
	FMVDX,
	FMVXD,
	FMVXW,
	FMVWX,
	FNMSUBD,
	FSD,
	FSW,
	FSGNJD,
	FSGNJXD,
	FSUBD,
	JAL,
	JALR,
	LB,
	LBU,
	LD,
	LH,
	LHU,
	LRD,
	LRW,
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
	SCD,
	SCW,
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
	SUBW,
	SW,
	URET,
	WFI,
	XOR,
	XORI
}

enum InstructionFormat {
	B,
	C, // CSR
	I,
	J,
	O, // Other, temporal format name
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

fn _get_trap_type_name(trap_type: &TrapType) -> &'static str {
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

fn get_instruction_name(instruction: &Instruction) -> &'static str {
	match instruction {
		Instruction::ADDW => "ADDW",
		Instruction::AMOADDD => "AMOADDD",
		Instruction::AMOADDW => "AMOADD.W",
		Instruction::AMOANDD => "AMOAND.D",
		Instruction::AMOANDW => "AMOAND.W",
		Instruction::AMOMAXUW => "AMOMAXU.W",
		Instruction::AMOORD => "AMOOR.D",
		Instruction::AMOORW => "AMOOR.W",
		Instruction::AMOSWAPD => "AMOSWAP.D",
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
		Instruction::EBREAK => "EBREAK",
		Instruction::ECALL => "ECALL",
		Instruction::FADDD => "FADD.D",
		Instruction::FCVTDL => "FCVT.D.L",
		Instruction::FCVTDS => "FCVT.D.S",
		Instruction::FCVTDW => "FCVT.D.W",
		Instruction::FCVTDWU => "FCVT.D.WU",
		Instruction::FCVTSD => "FCVT.S.D",
		Instruction::FCVTWD => "FCVT.W.D",
		Instruction::FDIVD => "FDIV.D",
		Instruction::FENCE => "FENCE",
		Instruction::FEQD => "FEQ.D",
		Instruction::FLD => "FLD",
		Instruction::FLED => "FLE.D",
		Instruction::FLTD => "FLT.D",
		Instruction::FLW => "FLW",
		Instruction::FMADDD => "FMADD.D",
		Instruction::FMULD => "FMUL.D",
		Instruction::FMVDX => "FMV.D.X",
		Instruction::FMVXD => "FMV.X.D",
		Instruction::FMVXW => "FMV.X.W",
		Instruction::FMVWX => "FMV.W.X",
		Instruction::FNMSUBD => "FNMSUB.D",
		Instruction::FSD => "FSD",
		Instruction::FSW => "FSW",
		Instruction::FSGNJD => "FSGNJD",
		Instruction::FSGNJXD => "FSGNJXD",
		Instruction::FSUBD => "FSUBD",
		Instruction::JAL => "JAL",
		Instruction::JALR => "JALR",
		Instruction::LB => "LB",
		Instruction::LBU => "LBU",
		Instruction::LD => "LD",
		Instruction::LH => "LH",
		Instruction::LHU => "LHU",
		Instruction::LRD => "LR.D",
		Instruction::LRW => "LR.W",
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
		Instruction::SCD => "SC.D",
		Instruction::SCW => "SC.W",
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
		Instruction::SUBW => "SUBW",
		Instruction::SW => "SW",
		Instruction::URET => "URET",
		Instruction::WFI => "WFI",
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
		Instruction::ANDI |
		Instruction::FLD |
		Instruction::FLW |
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
		Instruction::ADDW |
		Instruction::AMOADDD |
		Instruction::AMOADDW |
		Instruction::AMOANDD |
		Instruction::AMOANDW |
		Instruction::AMOMAXUW |
		Instruction::AMOORD |
		Instruction::AMOORW |
		Instruction::AMOSWAPD |
		Instruction::AMOSWAPW |
		Instruction::AND |
		Instruction::DIV |
		Instruction::DIVU |
		Instruction::DIVUW |
		Instruction::DIVW |
		Instruction::ECALL |
		Instruction::EBREAK |
		Instruction::FADDD |
		Instruction::FCVTDL |
		Instruction::FCVTDS |
		Instruction::FCVTDW |
		Instruction::FCVTDWU |
		Instruction::FCVTSD |
		Instruction::FCVTWD |
		Instruction::FDIVD |
		Instruction::FEQD |
		Instruction::FLED |
		Instruction::FLTD |
		Instruction::FMADDD |
		Instruction::FMULD |
		Instruction::FMVDX |
		Instruction::FMVXD |
		Instruction::FMVXW |
		Instruction::FMVWX |
		Instruction::FNMSUBD |
		Instruction::FSGNJD |
		Instruction::FSGNJXD |
		Instruction::FSUBD |
		Instruction::LRD |
		Instruction::LRW |
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
		Instruction::SCD |
		Instruction::SCW |
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
		Instruction::WFI |
		Instruction::XOR => InstructionFormat::R,
		Instruction::FSD |
		Instruction::FSW |
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
			wfi: false,
			x: [0; 32],
			f: [0.0; 32],
			pc: 0,
			csr: [0; CSR_CAPACITY],
			mmu: Mmu::new(Xlen::Bit64, terminal),
			reservation: 0,
			is_reservation_set: false,
			_dump_flag: false
		};
		cpu.x[0xb] = 0x1020; // I don't know why but Linux boot seems to require this initialization
		cpu.write_csr_raw(CSR_MISA_ADDRESS, 0x800000008014312f);
		cpu
	}

	// Six public methods for setting up from outside

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

	pub fn setup_dtb(&mut self, data: Vec<u8>) {
		self.mmu.init_dtb(data);
	}

	// One public method for running riscv-tests

	pub fn load_word_raw(&mut self, address: u64) -> u32 {
		self.mmu.load_word_raw(address)
	}

	//

	pub fn tick(&mut self) {
		let instruction_address = self.pc;
		match self.tick_operate() {
			Ok(()) => {},
			Err(e) => self.handle_exception(e, instruction_address)
		}
		self.mmu.tick(&mut self.csr[CSR_MIP_ADDRESS as usize]);
		self.handle_interrupt(self.pc);
		self.clock = self.clock.wrapping_add(1);
		self.write_csr_raw(CSR_CYCLE_ADDRESS, self.clock);
	}

	// @TODO: Rename
	fn tick_operate(&mut self) -> Result<(), Trap> {
		if self.wfi {
			return Ok(());
		}
		let mut word = match self.fetch() {
			Ok(word) => word,
			Err(e) => return Err(e)
		};
		let instruction_address = self.pc;
		if (word & 0x3) == 0x3 {
			self.pc = self.pc.wrapping_add(4); // 32-bit length non-compressed instruction
		} else {
			self.pc = self.pc.wrapping_add(2); // 16-bit length compressed instruction
			word = self.uncompress(word & 0xffff);
		}

		// New decode and operate system
		for i in 0..INSTRUCTION_NUM {
			let inst = &INSTRUCTIONS[i];
			if (word & inst.mask) == inst.data {
				return (inst.operation)(self, word);
			}
		}

		// Old decode and operate system
		// @TODO: Move all the instructions to the new system
		match self.decode(word) {
			Ok(instruction) => self.operate(word, instruction, instruction_address),
			Err(()) => panic!("Unknown instruction PC:{:X} WORD:{:X}", instruction_address, word)
		}
	}

	fn handle_interrupt(&mut self, instruction_address: u64) {
		// @TODO: Optimize
		let minterrupt = self.read_csr_raw(CSR_MIP_ADDRESS) & self.read_csr_raw(CSR_MIE_ADDRESS);

		if (minterrupt & MIP_MEIP) != 0 {
			if self.handle_trap(Trap {
				trap_type: TrapType::MachineExternalInterrupt,
				value: self.pc // dummy
			}, instruction_address, true) {
				// Who should fall mip bit?
				self.write_csr_raw(CSR_MIP_ADDRESS, self.read_csr_raw(CSR_MIP_ADDRESS) & !MIP_MEIP);
				self.wfi = false;
				return;
			}
		}
		if (minterrupt & MIP_MSIP) != 0 {
			if self.handle_trap(Trap {
				trap_type: TrapType::MachineSoftwareInterrupt,
				value: self.pc // dummy
			}, instruction_address, true) {
				self.write_csr_raw(CSR_MIP_ADDRESS, self.read_csr_raw(CSR_MIP_ADDRESS) & !MIP_MSIP);
				self.wfi = false;
				return;
			}
		}
		if (minterrupt & MIP_MTIP) != 0 {
			if self.handle_trap(Trap {
				trap_type: TrapType::MachineTimerInterrupt,
				value: self.pc // dummy
			}, instruction_address, true) {
				self.write_csr_raw(CSR_MIP_ADDRESS, self.read_csr_raw(CSR_MIP_ADDRESS) & !MIP_MTIP);
				self.wfi = false;
				return;
			}
		}
		if (minterrupt & MIP_SEIP) != 0 {
			if self.handle_trap(Trap {
				trap_type: TrapType::SupervisorExternalInterrupt,
				value: self.pc // dummy
			}, instruction_address, true) {
				self.write_csr_raw(CSR_MIP_ADDRESS, self.read_csr_raw(CSR_MIP_ADDRESS) & !MIP_SEIP);
				self.wfi = false;
				return;
			}
		}
		if (minterrupt & MIP_SSIP) != 0 {
			if self.handle_trap(Trap {
				trap_type: TrapType::SupervisorSoftwareInterrupt,
				value: self.pc // dummy
			}, instruction_address, true) {
				self.write_csr_raw(CSR_MIP_ADDRESS, self.read_csr_raw(CSR_MIP_ADDRESS) & !MIP_SEIP);
				self.wfi = false;
				return;
			}
		}
		if (minterrupt & MIP_STIP) != 0 {
			if self.handle_trap(Trap {
				trap_type: TrapType::SupervisorTimerInterrupt,
				value: self.pc // dummy
			}, instruction_address, true) {
				self.write_csr_raw(CSR_MIP_ADDRESS, self.read_csr_raw(CSR_MIP_ADDRESS) & !MIP_STIP);
				self.wfi = false;
				return;
			}
		}
	}

	fn handle_exception(&mut self, exception: Trap, instruction_address: u64) {
		self.handle_trap(exception, instruction_address, false);
	}

	fn handle_trap(&mut self, trap: Trap, instruction_address: u64, is_interrupt: bool) -> bool{
		let current_privilege_encoding = get_privilege_encoding(&self.privilege_mode) as u64;
		let cause = get_trap_cause(&trap, &self.xlen);

		// First, determine which privilege mode should handle the trap.
		// @TODO: Check if this logic is correct
		let mdeleg = match is_interrupt {
			true => self.read_csr_raw(CSR_MIDELEG_ADDRESS),
			false => self.read_csr_raw(CSR_MEDELEG_ADDRESS)
		};
		let sdeleg = match is_interrupt {
			true => self.read_csr_raw(CSR_SIDELEG_ADDRESS),
			false => self.read_csr_raw(CSR_SEDELEG_ADDRESS)
		};
		let pos = cause & 0xffff;
		let new_privilege_mode = match ((mdeleg >> pos) & 1) == 0 {
			true => PrivilegeMode::Machine,
			false => match ((sdeleg >> pos) & 1) == 0 {
				true => PrivilegeMode::Supervisor,
				false => PrivilegeMode::User
			}
		};
		let new_privilege_encoding = get_privilege_encoding(&new_privilege_mode) as u64;

		let current_status = match self.privilege_mode {
			PrivilegeMode::Machine => self.read_csr_raw(CSR_MSTATUS_ADDRESS),
			PrivilegeMode::Supervisor => self.read_csr_raw(CSR_SSTATUS_ADDRESS),
			PrivilegeMode::User => self.read_csr_raw(CSR_USTATUS_ADDRESS),
			PrivilegeMode::Reserved => panic!(),
		};

		// Second, ignore the interrupt if it's disabled by some conditions

		if is_interrupt {
			let ie = match new_privilege_mode {
				PrivilegeMode::Machine => self.read_csr_raw(CSR_MIE_ADDRESS),
				PrivilegeMode::Supervisor => self.read_csr_raw(CSR_SIE_ADDRESS),
				PrivilegeMode::User => self.read_csr_raw(CSR_UIE_ADDRESS),
				PrivilegeMode::Reserved => panic!(),
			};

			let current_mie = (current_status >> 3) & 1;
			let current_sie = (current_status >> 1) & 1;
			let current_uie = current_status & 1;

			let msie = (ie >> 3) & 1;
			let ssie = (ie >> 1) & 1;
			let usie = ie & 1;

			let mtie = (ie >> 7) & 1;
			let stie = (ie >> 5) & 1;
			let utie = (ie >> 4) & 1;

			let meie = (ie >> 11) & 1;
			let seie = (ie >> 9) & 1;
			let ueie = (ie >> 8) & 1;

			// 1. Interrupt is always enabled if new privilege level is higher
			// than current privilege level
			// 2. Interrupt is always disabled if new privilege level is lower
			// than current privilege level
			// 3. Interrupt is enabled if xIE in xstatus is 1 where x is privilege level
			// and new privilege level equals to current privilege level

			if new_privilege_encoding < current_privilege_encoding {
				return false;
			} else if current_privilege_encoding == new_privilege_encoding {
				match self.privilege_mode {
					PrivilegeMode::Machine => {
						if current_mie == 0 {
							return false;
						}
					},
					PrivilegeMode::Supervisor => {
						if current_sie == 0 {
							return false;
						}
					},
					PrivilegeMode::User => {
						if current_uie == 0 {
							return false;
						}
					},
					PrivilegeMode::Reserved => panic!()
				};
			}

			// Interrupt can be maskable by xie csr register
			// where x is a new privilege mode.

			match trap.trap_type {
				TrapType::UserSoftwareInterrupt => {
					if usie == 0 {
						return false;
					}
				},
				TrapType::SupervisorSoftwareInterrupt => {
					if ssie == 0 {
						return false;
					}
				},
				TrapType::MachineSoftwareInterrupt => {
					if msie == 0 {
						return false;
					}
				},
				TrapType::UserTimerInterrupt => {
					if utie == 0 {
						return false;
					}
				},
				TrapType::SupervisorTimerInterrupt => {
					if stie == 0 {
						return false;
					}
				},
				TrapType::MachineTimerInterrupt => {
					if mtie == 0 {
						return false;
					}
				},
				TrapType::UserExternalInterrupt => {
					if ueie == 0 {
						return false;
					}
				},
				TrapType::SupervisorExternalInterrupt => {
					if seie == 0 {
						return false;
					}
				},
				TrapType::MachineExternalInterrupt => {
					if meie == 0 {
						return false;
					}
				},
				_ => {}
			};
		}

		// So, this trap should be taken

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

		self.write_csr_raw(csr_epc_address, instruction_address);
		self.write_csr_raw(csr_cause_address, cause);
		self.write_csr_raw(csr_tval_address, trap.value);
		self.pc = self.read_csr_raw(csr_tvec_address);

		// Add 4 * cause if tvec has vector type address
		if (self.pc & 0x3) != 0 {
			self.pc = (self.pc & !0x3) + 4 * (cause & 0xffff);
		}

		match self.privilege_mode {
			PrivilegeMode::Machine => {
				let status = self.read_csr_raw(CSR_MSTATUS_ADDRESS);
				let mie = (status >> 3) & 1;
				// clear MIE[3], override MPIE[7] with MIE[3], override MPP[12:11] with current privilege encoding
				let new_status = (status & !0x1888) | (mie << 7) | (current_privilege_encoding << 11);
				self.write_csr_raw(CSR_MSTATUS_ADDRESS, new_status);
			},
			PrivilegeMode::Supervisor => {
				let status = self.read_csr_raw(CSR_SSTATUS_ADDRESS);
				let sie = (status >> 1) & 1;
				// clear SIE[1], override SPIE[5] with SIE[1], override SPP[8] with current privilege encoding
				let new_status = (status & !0x122) | (sie << 5) | ((current_privilege_encoding & 1) << 8);
				self.write_csr_raw(CSR_SSTATUS_ADDRESS, new_status);
			},
			PrivilegeMode::User => {
				panic!("Not implemented yet");
			},
			PrivilegeMode::Reserved => panic!() // shouldn't happen
		};
		//println!("Trap! {:X} Clock:{:X}", cause, self.clock);
		true
	}

	fn fetch(&mut self) -> Result<u32, Trap> {
		let word = match self.mmu.fetch_word(self.pc) {
			Ok(word) => word,
			Err(e) => {
				self.pc = self.pc.wrapping_add(4); // @TODO: What if instruction is compressed?
				return Err(e);
			}
		};
		Ok(word)
	}

	fn has_csr_access_privilege(&self, address: u16) -> bool {
		let privilege = (address >> 8) & 0x3; // the lowest privilege level that can access the CSR
		privilege as u8 <= get_privilege_encoding(&self.privilege_mode)
	}

	fn read_csr(&mut self, address: u16) -> Result<u64, Trap> {
		match self.has_csr_access_privilege(address) {
			true => Ok(self.read_csr_raw(address)),
			false => Err(Trap {
				trap_type: TrapType::IllegalInstruction,
				value: self.pc.wrapping_sub(4) // @TODO: Is this always correct?
			})
		}
	}

	fn write_csr(&mut self, address: u16, value: u64) -> Result<(), Trap> {
		match self.has_csr_access_privilege(address) {
			true => {
				/*
				// Checking writability fails some tests so disabling so far
				let read_only = ((address >> 10) & 0x3) == 0x3;
				if read_only {
					return Err(Exception::IllegalInstruction);
				}
				*/
				self.write_csr_raw(address, value);
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

	// SSTATUS, SIE, and SIP are subsets of MSTATUS, MIE, and MIP
	fn read_csr_raw(&self, address: u16) -> u64 {
		match address {
			// @TODO: Mask shuld consider of 32-bit mode
			CSR_SSTATUS_ADDRESS => self.csr[CSR_MSTATUS_ADDRESS as usize] & 0x80000003000de162,
			CSR_SIE_ADDRESS => self.csr[CSR_MIE_ADDRESS as usize] & 0x222,
			CSR_SIP_ADDRESS => self.csr[CSR_MIP_ADDRESS as usize] & 0x222,
			CSR_TIME_ADDRESS => self.mmu.get_clint().read_mtime(),
			_ => self.csr[address as usize]
		}
	}

	fn write_csr_raw(&mut self, address: u16, value: u64) {
		match address {
			CSR_SSTATUS_ADDRESS => {
				self.csr[CSR_MSTATUS_ADDRESS as usize] &= !0x80000003000de162;
				self.csr[CSR_MSTATUS_ADDRESS as usize] |= value & 0x80000003000de162;
			},
			CSR_SIE_ADDRESS => {
				self.csr[CSR_MIE_ADDRESS as usize] &= !0x222;
				self.csr[CSR_MIE_ADDRESS as usize] |= value & 0x222;
			},
			CSR_SIP_ADDRESS => {
				self.csr[CSR_MIP_ADDRESS as usize] &= !0x222;
				self.csr[CSR_MIP_ADDRESS as usize] |= value & 0x222;
			},
			CSR_MIDELEG_ADDRESS => {
				self.csr[address as usize] = value & 0x666; // from qemu
			},
			CSR_TIME_ADDRESS => {
				self.mmu.get_mut_clint().write_mtime(value);
			},
			_ => {
				self.csr[address as usize] = value;
			}
		};
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

	// @TODO: Optimize
	fn uncompress(&self, halfword: u32) -> u32 {
		let op = halfword & 0x3; // [1:0]
		let funct3 = (halfword >> 13) & 0x7; // [15:13]

		match op {
			0 => match funct3 {
				0 => {
					// C.ADDI4SPN
					// addi rd+8, x2, nzuimm
					let rd = (halfword >> 2) & 0x7; // [4:2]
					let nzuimm =
						((halfword >> 7) & 0x30) | // nzuimm[5:4] <= [12:11]
						((halfword >> 1) & 0x3c0) | // nzuimm{9:6] <= [10:7]
						((halfword >> 4) & 0x4) | // nzuimm[2] <= [6]
						((halfword >> 2) & 0x8); // nzuimm[3] <= [5]
					// nzuimm == 0 is reserved instruction
					if nzuimm != 0 {
						return (nzuimm << 20) | (2 << 15) | ((rd + 8) << 7) | 0x13;
					}
				},
				1 => {
					// @TODO: Support C.LQ for 128-bit
					// C.FLD for 32, 64-bit
					// fld rd+8, offset(rs1+8)
					let rd = (halfword >> 2) & 0x7; // [4:2]
					let rs1 = (halfword >> 7) & 0x7; // [9:7]
					let offset =
						((halfword >> 7) & 0x38) | // offset[5:3] <= [12:10]
						((halfword << 1) & 0xc0); // offset[7:6] <= [6:5]
					return (offset << 20) | ((rs1 + 8) << 15) | (3 << 12) | ((rd + 8) << 7) | 0x7;
				},
				2 => {
					// C.LW
					// lw rd+8, offset(rs1+8)
					let rs1 = (halfword >> 7) & 0x7; // [9:7]
					let rd = (halfword >> 2) & 0x7; // [4:2]
					let offset =
						((halfword >> 7) & 0x38) | // offset[5:3] <= [12:10]
						((halfword >> 4) & 0x4) | // offset[2] <= [6]
						((halfword << 1) & 0x40); // offset[6] <= [5]
					return (offset << 20) | ((rs1 + 8) << 15) | (2 << 12) | ((rd + 8) << 7) | 0x3;
				},
				3 => {
					// @TODO: Support C.FLW in 32-bit mode
					// C.LD in 64-bit mode
					// ld rd+8, offset(rs1+8)
					let rs1 = (halfword >> 7) & 0x7; // [9:7]
					let rd = (halfword >> 2) & 0x7; // [4:2]
					let offset =
						((halfword >> 7) & 0x38) | // offset[5:3] <= [12:10]
						((halfword << 1) & 0xc0); // offset[7:6] <= [6:5]
					return (offset << 20) | ((rs1 + 8) << 15) | (3 << 12) | ((rd + 8) << 7) | 0x3;
				},
				4 => {
					// Reserved
				},
				5 => {
					// C.FSD
					// fsd rs2+8, offset(rs1+8)
					let rs1 = (halfword >> 7) & 0x7; // [9:7]
					let rs2 = (halfword >> 2) & 0x7; // [4:2]
					let offset = 
						((halfword >> 7) & 0x38) | // uimm[5:3] <= [12:10]
						((halfword << 1) & 0xc0); // uimm[7:6] <= [6:5]
					let imm11_5 = (offset >> 5) & 0x7f;
					let imm4_0 = offset & 0x1f;
					return (imm11_5 << 25) | ((rs2 + 8) << 20) | ((rs1 + 8) << 15) | (3 << 12) | (imm4_0 << 7) | 0x27;
				},
				6 => {
					// C.SW
					// sw rs2+8, offset(rs1+8)
					let rs1 = (halfword >> 7) & 0x7; // [9:7]
					let rs2 = (halfword >> 2) & 0x7; // [4:2]
					let offset = 
						((halfword >> 7) & 0x38) | // offset[5:3] <= [12:10]
						((halfword << 1) & 0x40) | // offset[6] <= [5]
						((halfword >> 4) & 0x4); // offset[2] <= [6]
					let imm11_5 = (offset >> 5) & 0x7f;
					let imm4_0 = offset & 0x1f;
					return (imm11_5 << 25) | ((rs2 + 8) << 20) | ((rs1 + 8) << 15) | (2 << 12) | (imm4_0 << 7) | 0x23;
				},
				7 => {
					// @TODO: Support C.FSW in 32-bit mode
					// C.SD
					// sd rs2+8, offset(rs1+8)
					let rs1 = (halfword >> 7) & 0x7; // [9:7]
					let rs2 = (halfword >> 2) & 0x7; // [4:2]
					let offset = 
						((halfword >> 7) & 0x38) | // uimm[5:3] <= [12:10]
						((halfword << 1) & 0xc0); // uimm[7:6] <= [6:5]
					let imm11_5 = (offset >> 5) & 0x7f;
					let imm4_0 = offset & 0x1f;
					return (imm11_5 << 25) | ((rs2 + 8) << 20) | ((rs1 + 8) << 15) | (3 << 12) | (imm4_0 << 7) | 0x23;
				},
				_ => {} // Not happens
			},
			1 => {
				match funct3 {
					0 => {
						let r = (halfword >> 7) & 0x1f; // [11:7]
						let imm = match halfword & 0x1000 {
							0x1000 => 0xffffffc0,
							_ => 0
						} | // imm[31:6] <= [12]
						((halfword >> 7) & 0x20) | // imm[5] <= [12]
						((halfword >> 2) & 0x1f); // imm[4:0] <= [6:2]
						if r == 0 && imm == 0 {
							// C.NOP
							// addi x0, x0, 0
							return 0x13;
						} else if r != 0 {
							// C.ADDI
							// addi r, r, imm
							return (imm << 20) | (r << 15) | (r << 7) | 0x13;
						}
						// @TODO: Support HINTs
						// r == 0 and imm != 0 is HINTs
					},
					1 => {
						// @TODO: Support C.JAL in 32-bit mode
						// C.ADDIW
						// addiw r, r, imm
						let r = (halfword >> 7) & 0x1f;
						let imm = match halfword & 0x1000 {
							0x1000 => 0xffffffc0,
							_ => 0
						} | // imm[31:6] <= [12]
						((halfword >> 7) & 0x20) | // imm[5] <= [12]
						((halfword >> 2) & 0x1f); // imm[4:0] <= [6:2]
						if r != 0 {
							return (imm << 20) | (r << 15) | (r << 7) | 0x1b;
						}
						// r == 0 is reserved instruction
					},
					2 => {
						// C.LI
						// addi rd, x0, imm
						let r = (halfword >> 7) & 0x1f;
						let imm = match halfword & 0x1000 {
							0x1000 => 0xffffffc0,
							_ => 0
						} | // imm[31:6] <= [12]
						((halfword >> 7) & 0x20) | // imm[5] <= [12]
						((halfword >> 2) & 0x1f); // imm[4:0] <= [6:2]
						if r != 0 {
							return (imm << 20) | (r << 7) | 0x13;
						}
						// @TODO: Support HINTs
						// r == 0 is for HINTs
					},
					3 => {
						let r = (halfword >> 7) & 0x1f; // [11:7]
						if r == 2 {
							// C.ADDI16SP
							// addi r, r, nzimm
							let imm = match halfword & 0x1000 {
								0x1000 => 0xfffffc00,
								_ => 0
							} | // imm[31:10] <= [12]
							((halfword >> 3) & 0x200) | // imm[9] <= [12]
							((halfword >> 2) & 0x10) | // imm[4] <= [6]
							((halfword << 1) & 0x40) | // imm[6] <= [5]
							((halfword << 4) & 0x180) | // imm[8:7] <= [4:3]
							((halfword << 3) & 0x20); // imm[5] <= [2]
							if imm != 0 {
								return (imm << 20) | (r << 15) | (r << 7) | 0x13;
							}
							// imm == 0 is for reserved instruction
						}
						if r != 0 && r != 2 {
							// C.LUI
							// lui r, nzimm
							let nzimm = match halfword & 0x1000 {
								0x1000 => 0xfffc0000,
								_ => 0
							} | // nzimm[31:18] <= [12]
							((halfword << 5) & 0x20000) | // nzimm[17] <= [12]
							((halfword << 10) & 0x1f000); // nzimm[16:12] <= [6:2]
							if nzimm != 0 {
								return nzimm | (r << 7) | 0x37;
							}
							// nzimm == 0 is for reserved instruction
						}
					},
					4 => {
						let funct2 = (halfword >> 10) & 0x3; // [11:10]
						match funct2 {
							0 => {
								// C.SRLI
								// c.srli rs1+8, rs1+8, shamt
								let shamt = 
									((halfword >> 7) & 0x20) | // shamt[5] <= [12]
									((halfword >> 2) & 0x1f); // shamt[4:0] <= [6:2]
								let rs1 = (halfword >> 7) & 0x7; // [9:7]
								return (shamt << 20) | ((rs1 + 8) << 15) | (5 << 12) | ((rs1 + 8) << 7) | 0x13;
							},
							1 => {
								// C.SRAI
								// srai rs1+8, rs1+8, shamt
								let shamt = 
									((halfword >> 7) & 0x20) | // shamt[5] <= [12]
									((halfword >> 2) & 0x1f); // shamt[4:0] <= [6:2]
								let rs1 = (halfword >> 7) & 0x7; // [9:7]
								return (0x20 << 25) | (shamt << 20) | ((rs1 + 8) << 15) | (5 << 12) | ((rs1 + 8) << 7) | 0x13;
							},
							2 => {
								// C.ANDI
								// andi, r+8, r+8, imm
								let r = (halfword >> 7) & 0x7; // [9:7]
								let imm = match halfword & 0x1000 {
									0x1000 => 0xffffffc0,
									_ => 0
								} | // imm[31:6] <= [12]
								((halfword >> 7) & 0x20) | // imm[5] <= [12]
								((halfword >> 2) & 0x1f); // imm[4:0] <= [6:2]
								return (imm << 20) | ((r + 8) << 15) | (7 << 12) | ((r + 8) << 7) | 0x13;
							},
							3 => {
								let funct1 = (halfword >> 12) & 1; // [12]
								let funct2_2 = (halfword >> 5) & 0x3; // [6:5]
								let rs1 = (halfword >> 7) & 0x7;
								let rs2 = (halfword >> 2) & 0x7;
								match funct1 {
									0 => match funct2_2 {
										0 => {
											// C.SUB
											// sub rs1+8, rs1+8, rs2+8
											return (0x20 << 25) | ((rs2 + 8) << 20) | ((rs1 + 8) << 15) | ((rs1 + 8) << 7) | 0x33;
										},
										1 => {
											// C.XOR
											// xor rs1+8, rs1+8, rs2+8
											return ((rs2 + 8) << 20) | ((rs1 + 8) << 15) | (4 << 12) | ((rs1 + 8) << 7) | 0x33;
										},
										2 => {
											// C.OR
											// or rs1+8, rs1+8, rs2+8
											return ((rs2 + 8) << 20) | ((rs1 + 8) << 15) | (6 << 12) | ((rs1 + 8) << 7) | 0x33;
										},
										3 => {
											// C.AND
											// and rs1+8, rs1+8, rs2+8
											return ((rs2 + 8) << 20) | ((rs1 + 8) << 15) | (7 << 12) | ((rs1 + 8) << 7) | 0x33;
										},
										_ => {} // Not happens
									},
									1 => match funct2_2 {
										0 => {
											// C.SUBW
											// subw r1+8, r1+8, r2+8
											return (0x20 << 25) | ((rs2 + 8) << 20) | ((rs1 + 8) << 15) | ((rs1 + 8) << 7) | 0x3b;
										},
										1 => {
											// C.ADDW
											// addw r1+8, r1+8, r2+8
											return ((rs2 + 8) << 20) | ((rs1 + 8) << 15) | ((rs1 + 8) << 7) | 0x3b;
										},
										2 => {
											// Reserved
										},
										3 => {
											// Reserved
										},
										_ => {} // Not happens
									},
									_ => {} // No happens
								};
							},
							_ => {} // not happens
						};
					},
					5 => {
						// C.J
						// jal x0, imm
						let offset =
							match halfword & 0x1000 {
								0x1000 => 0xfffff000,
								_ => 0
							} | // offset[31:12] <= [12]
							((halfword >> 1) & 0x800) | // offset[11] <= [12]
							((halfword >> 7) & 0x10) | // offset[4] <= [11]
							((halfword >> 1) & 0x300) | // offset[9:8] <= [10:9]
							((halfword << 2) & 0x400) | // offset[10] <= [8]
							((halfword >> 1) & 0x40) | // offset[6] <= [7]
							((halfword << 1) & 0x80) | // offset[7] <= [6]
							((halfword >> 2) & 0xe) | // offset[3:1] <= [5:3]
							((halfword << 3) & 0x20); // offset[5] <= [2]
						let imm =
							((offset >> 1) & 0x80000) | // imm[19] <= offset[20]
							((offset << 8) & 0x7fe00) | // imm[18:9] <= offset[10:1]
							((offset >> 3) & 0x100) | // imm[8] <= offset[11]
							((offset >> 12) & 0xff); // imm[7:0] <= offset[19:12]
						return (imm << 12) | 0x6f;
					},
					6 => {
						// C.BEQZ
						// beq r+8, x0, offset
						let r = (halfword >> 7) & 0x7;
						let offset =
							match halfword & 0x1000 {
								0x1000 => 0xfffffe00,
								_ => 0
							} | // offset[31:9] <= [12]
							((halfword >> 4) & 0x100) | // offset[8] <= [12]
							((halfword >> 7) & 0x18) | // offset[4:3] <= [11:10]
							((halfword << 1) & 0xc0) | // offset[7:6] <= [6:5]
							((halfword >> 2) & 0x6) | // offset[2:1] <= [4:3]
							((halfword << 3) & 0x20); // offset[5] <= [2]
						let imm2 =
							((offset >> 6) & 0x40) | // imm2[6] <= [12]
							((offset >> 5) & 0x3f); // imm2[5:0] <= [10:5]
						let imm1 =
							(offset & 0x1e) | // imm1[4:1] <= [4:1]
							((offset >> 11) & 0x1); // imm1[0] <= [11]
						return (imm2 << 25) | ((r + 8) << 20) | (imm1 << 7) | 0x63;
					},
					7 => {
						// C.BNEZ
						// bne r+8, x0, offset
						let r = (halfword >> 7) & 0x7;
						let offset =
							match halfword & 0x1000 {
								0x1000 => 0xfffffe00,
								_ => 0
							} | // offset[31:9] <= [12]
							((halfword >> 4) & 0x100) | // offset[8] <= [12]
							((halfword >> 7) & 0x18) | // offset[4:3] <= [11:10]
							((halfword << 1) & 0xc0) | // offset[7:6] <= [6:5]
							((halfword >> 2) & 0x6) | // offset[2:1] <= [4:3]
							((halfword << 3) & 0x20); // offset[5] <= [2]
						let imm2 =
							((offset >> 6) & 0x40) | // imm2[6] <= [12]
							((offset >> 5) & 0x3f); // imm2[5:0] <= [10:5]
						let imm1 =
							(offset & 0x1e) | // imm1[4:1] <= [4:1]
							((offset >> 11) & 0x1); // imm1[0] <= [11]
						return (imm2 << 25) | ((r + 8) << 20) | (1 << 12) | (imm1 << 7) | 0x63;
					},
					_ => {} // No happens
				};
			},
			2 => {
				match funct3 {
					0 => {
						// C.SLLI
						// slli r, r, shamt
						let r = (halfword >> 7) & 0x1f;
						let shamt =
							((halfword >> 7) & 0x20) | // imm[5] <= [12]
							((halfword >> 2) & 0x1f); // imm[4:0] <= [6:2]
						if r != 0 {
							return (shamt << 20) | (r << 15) | (1 << 12) | (r << 7) | 0x13;
						}
						// r == 0 is reserved instruction?
					},
					1 => {
						// C.FLDSP
						// fld rd, offset(x2)
						let rd = (halfword >> 7) & 0x1f;
						let offset =
							((halfword >> 7) & 0x20) | // offset[5] <= [12]
							((halfword >> 2) & 0x18) | // offset[4:3] <= [6:5]
							((halfword << 4) & 0x1c0); // offset[8:6] <= [4:2]
						if rd != 0 {
							return (offset << 20) | (2 << 15) | (3 << 12) | (rd << 7) | 0x7;
						}
						// rd == 0 is reseved instruction
					},
					2 => {
						// C.LWSP
						// lw r, offset(x2)
						let r = (halfword >> 7) & 0x1f;
						let offset =
							((halfword >> 7) & 0x20) | // offset[5] <= [12]
							((halfword >> 2) & 0x1c) | // offset[4:2] <= [6:4]
							((halfword << 4) & 0xc0); // offset[7:6] <= [3:2]
						if r != 0 {
							return (offset << 20) | (2 << 15) | (2 << 12) | (r << 7) | 0x3;
						}
						// r == 0 is reseved instruction
					},
					3 => {
						// @TODO: Support C.FLWSP in 32-bit mode
						// C.LDSP
						// ld rd, offset(x2)
						let rd = (halfword >> 7) & 0x1f;
						let offset =
							((halfword >> 7) & 0x20) | // offset[5] <= [12]
							((halfword >> 2) & 0x18) | // offset[4:3] <= [6:5]
							((halfword << 4) & 0x1c0); // offset[8:6] <= [4:2]
						if rd != 0 {
							return (offset << 20) | (2 << 15) | (3 << 12) | (rd << 7) | 0x3;
						}
						// rd == 0 is reseved instruction
					},
					4 => {
						let funct1 = (halfword >> 12) & 1; // [12]
						let rs1 = (halfword >> 7) & 0x1f; // [11:7]
						let rs2 = (halfword >> 2) & 0x1f; // [6:2]
						match funct1 {
							0 => {
								if rs1 != 0 && rs2 == 0 {
									// C.JR
									// jalr x0, 0(rs1)
									return (rs1 << 15) | 0x67;
								}
								// rs1 == 0 is reserved instruction
								if rs1 != 0 && rs2 != 0 {
									// C.MV
									// add rs1, x0, rs2
									// println!("C.MV RS1:{:X} RS2:{:X}", rs1, rs2);
									return (rs2 << 20) | (rs1 << 7) | 0x33;
								}
								// rs1 == 0 && rs2 != 0 is Hints
								// @TODO: Support Hints
							},
							1 => {
								if rs1 == 0 && rs2 == 0 {
									// C.EBREAK
									// ebreak
									return 0x00100073;
								}
								if rs1 != 0 && rs2 == 0 {
									// C.JALR
									// jalr x1, 0(rs1)
									return (rs1 << 15) | (1 << 7) | 0x67;
								}
								if rs1 != 0 && rs2 != 0 {
									// C.ADD
									// add rs1, rs1, rs2
									return (rs2 << 20) | (rs1 << 15) | (rs1 << 7) | 0x33;
								}
								// rs1 == 0 && rs2 != 0 is Hists
								// @TODO: Supports Hinsts
							},
							_ => {} // Not happens
						};
					},
					5 => {
						// @TODO: Implement
						// C.FSDSP
						// fsd rs2, offset(x2)
						let rs2 = (halfword >> 2) & 0x1f; // [6:2]
						let offset =
							((halfword >> 7) & 0x38) | // offset[5:3] <= [12:10]
							((halfword >> 1) & 0x1c0); // offset[8:6] <= [9:7]
						let imm11_5 = (offset >> 5) & 0x3f;
						let imm4_0 = offset & 0x1f;
						return (imm11_5 << 25) | (rs2 << 20) | (2 << 15) | (3 << 12) | (imm4_0 << 7) | 0x27;
					},
					6 => {
						// C.SWSP
						// sw rs2, offset(x2)
						let rs2 = (halfword >> 2) & 0x1f; // [6:2]
						let offset =
							((halfword >> 7) & 0x3c) | // offset[5:2] <= [12:9]
							((halfword >> 1) & 0xc0); // offset[7:6] <= [8:7]
						let imm11_5 = (offset >> 5) & 0x3f;
						let imm4_0 = offset & 0x1f;
						return (imm11_5 << 25) | (rs2 << 20) | (2 << 15) | (2 << 12) | (imm4_0 << 7) | 0x23;
					},
					7 => {
						// @TODO: Support C.FSWSP in 32-bit mode
						// C.SDSP
						// sd rs, offset(x2)
						let rs2 = (halfword >> 2) & 0x1f; // [6:2]
						let offset =
							((halfword >> 7) & 0x38) | // offset[5:3] <= [12:10]
							((halfword >> 1) & 0x1c0); // offset[8:6] <= [9:7]
						let imm11_5 = (offset >> 5) & 0x3f;
						let imm4_0 = offset & 0x1f;
						return (imm11_5 << 25) | (rs2 << 20) | (2 << 15) | (3 << 12) | (imm4_0 << 7) | 0x23;
					},
					_ => {} // Not happens
				};
			},
			_ => {} // No happnes
		};
		0xffffffff // Return invalid value
	}

	// @TODO: Optimize
	fn decode(&mut self, word: u32) -> Result<Instruction, ()> {
		let opcode = word & 0x7f; // [6:0]
		let funct3 = (word >> 12) & 0x7; // [14:12]
		let funct5 = (word >> 20) & 0x1f; // [24:20]
		let funct7 = (word >> 25) & 0x7f; // [31:25]

		let instruction = match opcode {
			0x03 => match funct3 {
				0 => Instruction::LB,
				1 => Instruction::LH,
				2 => Instruction::LW,
				3 => Instruction::LD,
				4 => Instruction::LBU,
				5 => Instruction::LHU,
				6 => Instruction::LWU,
				_ => return Err(())
			},
			0x07 => match funct3 {
				2 => Instruction::FLW,
				3 => Instruction::FLD,
				_ => return Err(())
			},
			0x0f => Instruction::FENCE,
			0x13 => match funct3 {
				1 => Instruction::SLLI,
				2 => Instruction::SLTI,
				3 => Instruction::SLTIU,
				4 => Instruction::XORI,
				5 => match funct7 & !1 {
					0 => Instruction::SRLI,
					1 => Instruction::SRLI, // temporal workaround for xv6
					0x20 => Instruction::SRAI,
					_ => return Err(())
				}
				6 => Instruction::ORI,
				7 => Instruction::ANDI,
				_ => return Err(())
			},
			0x17 => Instruction::AUIPC,
			0x1b => match funct3 {
				1 => Instruction::SLLIW,
				5 => match funct7 {
					0 => Instruction::SRLIW,
					0x20 => Instruction::SRAIW,
					_ => return Err(())
				},
				_ => return Err(())
			},
			0x23 => match funct3 {
				0 => Instruction::SB,
				1 => Instruction::SH,
				2 => Instruction::SW,
				3 => Instruction::SD,
				_ => return Err(())
			},
			0x27 => match funct3 {
				2 => Instruction::FSW,
				3 => Instruction::FSD,
				_ => return Err(())
			},
			0x2f => match funct3 {
				2 => {
					match funct7 >> 2 {
						0 => Instruction::AMOADDW,
						1 => Instruction::AMOSWAPW,
						2 => Instruction::LRW,
						3 => Instruction::SCW,
						8 => Instruction::AMOORW,
						0xc => Instruction::AMOANDW,
						0x1c => Instruction::AMOMAXUW,
						_ => return Err(())
					}
				},
				3 => {
					match funct7 >> 2 {
						0 => Instruction::AMOADDD,
						1 => Instruction::AMOSWAPD,
						2 => Instruction::LRD,
						3 => Instruction::SCD,
						8 => Instruction::AMOORD,
						0xc => Instruction::AMOANDD,
						_ => return Err(())
					}
				},
				_ => return Err(())
			}
			0x33 => match funct3 {
				0 => match funct7 {
					1 => Instruction::MUL,
					_ => return Err(())
				},
				1 => match funct7 {
					0 => Instruction::SLL,
					1 => Instruction::MULH,
					_ => return Err(())
				},
				2 => match funct7 {
					0 => Instruction::SLT,
					1 => Instruction::MULHSU,
					_ => return Err(())
				},
				3 => match funct7 {
					0 => Instruction::SLTU,
					1 => Instruction::MULHU,
					_ => return Err(())
				},
				4 => match funct7 {
					0 => Instruction::XOR,
					1 => Instruction::DIV,
					_ => return Err(())
				},
				5 => match funct7 {
					0 => Instruction::SRL,
					1 => Instruction::DIVU,
					0x20 => Instruction::SRA,
					_ => return Err(())
				},
				6 => match funct7 {
					0 => Instruction::OR,
					1 => Instruction::REM,
					_ => return Err(())
				},
				7 => match funct7 {
					0 => Instruction::AND,
					1 => Instruction::REMU,
					_ => return Err(())
				},
				_ => return Err(())
			},
			0x37 => Instruction::LUI,
			0x3b => match funct3 {
				0 => match funct7 {
					0 => Instruction::ADDW,
					1 => Instruction::MULW,
					0x20 => Instruction::SUBW,
					_ => return Err(())
				},
				1 => Instruction::SLLW,
				4 => Instruction::DIVW,
				5 => match funct7 {
					0 => Instruction::SRLW,
					1 => Instruction::DIVUW,
					0x20 => Instruction::SRAW,
					_ => return Err(())
				},
				6 => Instruction::REMW,
				7 => Instruction::REMUW,
				_ => return Err(())
			},
			0x43 => match funct7 & 0x3 {
				1 => Instruction::FMADDD,
				_ => return Err(())
			},
			0x4b => match funct7 & 0x3 {
				1 => Instruction::FNMSUBD,
				_ => return Err(())
			},
			0x53 => match funct7 {
				0x1 => Instruction::FADDD,
				0x5 => Instruction::FSUBD,
				0x9 => Instruction::FMULD,
				0xd => Instruction::FDIVD,
				0x11 => match funct3 {
					0 => Instruction::FSGNJD,
					2 => Instruction::FSGNJXD,
					_ => return Err(())
				},
				0x20 => match funct5 {
					1 => Instruction::FCVTSD,
					_ => return Err(())
				},
				0x21 => match funct5 {
					0 => Instruction::FCVTDS,
					_ => return Err(())
				},
				0x51 => match funct3 {
					0 => Instruction::FLED,
					1 => Instruction::FLTD,
					2 => Instruction::FEQD,
					_ => return Err(())
				},
				0x61 => match funct5 {
					0 => Instruction::FCVTWD,
					_ => return Err(())				
				},
				0x69 => match funct5 {
					0 => Instruction::FCVTDW,
					1 => Instruction::FCVTDWU,
					2 => Instruction::FCVTDL,
					_ => return Err(())
				},
				0x70 => match funct5 {
					0 => match funct3 {
						0 => Instruction::FMVXW,
						_ => return Err(())
					},
					_ => return Err(())
				},
				0x71 => match funct5 {
					0 => match funct3 {
						0 => Instruction::FMVXD,
						_ => return Err(())
					},
					_ => return Err(())
				},
				0x78 => match funct5 {
					0 => match funct3 {
						0 => Instruction::FMVWX,
						_ => return Err(())
					},
					_ => return Err(())
				},
				0x79 => match funct5 {
					0 => Instruction::FMVDX,
					_ => return Err(())
				},
				_ => return Err(())
			},
			0x63 => match funct3 {
				0 => Instruction::BEQ,
				1 => Instruction::BNE,
				4 => Instruction::BLT,
				5 => Instruction::BGE,
				6 => Instruction::BLTU,
				7 => Instruction::BGEU,
				_ => return Err(())
			},
			0x67 => Instruction::JALR,
			0x6f => Instruction::JAL,
			0x73 => match funct3 {
				0 => {
					match funct7 {
						9 => Instruction::SFENCEVMA,
						_ => match word {
							0x00000073 => Instruction::ECALL,
							0x00100073 => Instruction::EBREAK,
							0x00200073 => Instruction::URET,
							0x10200073 => Instruction::SRET,
							0x10500073 => Instruction::WFI,
							0x30200073 => Instruction::MRET,
							_ => return Err(())
						}
					}
				}
				1 => Instruction::CSRRW,
				2 => Instruction::CSRRS,
				3 => Instruction::CSRRC,
				5 => Instruction::CSRRWI,
				6 => Instruction::CSRRSI,
				7 => Instruction::CSRRCI,
				_ => return Err(())
			},
			_ => return Err(())
		};
		Ok(instruction)
	}

	fn operate(&mut self, word: u32, instruction: Instruction, instruction_address: u64) -> Result<(), Trap> {
		let instruction_format = get_instruction_format(&instruction);
		match instruction_format {
			InstructionFormat::B => {
				let rs1 = (word & 0x000f8000) >> 15; // [19:15]
				let rs2 = (word & 0x01f00000) >> 20; // [24:20]
				let imm = (
					match word & 0x80000000 { // imm[31:12] = [31]
						0x80000000 => 0xfffff000,
						_ => 0
					} |
					((word & 0x00000080) << 4) | // imm[11] = [7]
					((word & 0x7e000000) >> 20) | // imm[10:5] = [30:25]
					((word & 0x00000f00) >> 7) // imm[4:1] = [11:8]
				) as i32 as i64 as u64;
				//if instruction_address == 0xffffffff802036da {
				//	println!("Compare RS1:{:X} RS2:{:X} RS1VAL:{:X} RS2VAL:{:X}", rs1, rs2, self.x[rs1 as usize], self.x[rs2 as usize]);
				//}
				match instruction {
					Instruction::BEQ => {
						if self.sign_extend(self.x[rs1 as usize]) == self.sign_extend(self.x[rs2 as usize]) {
							self.pc = instruction_address.wrapping_add(imm);
						}
					},
					Instruction::BGE => {
						if self.sign_extend(self.x[rs1 as usize]) >= self.sign_extend(self.x[rs2 as usize]) {
							self.pc = instruction_address.wrapping_add(imm);
						}
					},
					Instruction::BGEU => {
						if self.unsigned_data(self.x[rs1 as usize]) >= self.unsigned_data(self.x[rs2 as usize]) {
							self.pc = instruction_address.wrapping_add(imm);
						}
					},
					Instruction::BLT => {
						if self.sign_extend(self.x[rs1 as usize]) < self.sign_extend(self.x[rs2 as usize]) {
							self.pc = instruction_address.wrapping_add(imm);
						}
					},
					Instruction::BLTU => {
						if self.unsigned_data(self.x[rs1 as usize]) < self.unsigned_data(self.x[rs2 as usize]) {
							self.pc = instruction_address.wrapping_add(imm);
						}
					},
					Instruction::BNE => {
						if self.sign_extend(self.x[rs1 as usize]) != self.sign_extend(self.x[rs2 as usize]) {
							self.pc = instruction_address.wrapping_add(imm);
						}
					},
					_ => {
						println!("{}", get_instruction_name(&instruction).to_owned() + " instruction is not supported yet.");
						self.dump_instruction(instruction_address);
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
						//self.x[0] = 0; // hard-wired zero
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
						//self.x[0] = 0; // hard-wired zero
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
						//self.x[0] = 0; // hard-wired zero
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
						//self.x[0] = 0; // hard-wired zero
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
						//self.x[0] = 0; // hard-wired zero
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
						//self.x[0] = 0; // hard-wired zero
						match self.write_csr(csr, rs as u64) {
							Ok(()) => {},
							Err(e) => return Err(e)
						};
					},
					_ => {
						println!("{}", get_instruction_name(&instruction).to_owned() + " instruction is not supported yet.");
						self.dump_instruction(instruction_address);
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
					Instruction::ANDI => {
						self.x[rd as usize] = self.sign_extend(self.x[rs1 as usize] & imm);
					},
					Instruction::FLD => {
						self.f[rd as usize] = match self.mmu.load_doubleword(self.x[rs1 as usize].wrapping_add(imm) as u64) {
							Ok(data) => {
								f64::from_bits(data)
							},
							Err(e) => return Err(e)
						};
					},
					Instruction::FLW => {
						// @TODO: Implement properly
						self.f[rd as usize] = match self.mmu.load_word(self.x[rs1 as usize].wrapping_add(imm) as u64) {
							Ok(data) => {
								f64::from_bits(data as i32 as i64 as u64)
							},
							Err(e) => return Err(e)
						};
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
						//println!("RS1:{:X} RS1VAL:{:X}", rs1, self.x[rs1 as usize]);
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
						self.dump_instruction(instruction_address);
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
						self.pc = instruction_address.wrapping_add(imm);
					},
					_ => {
						println!("{}", get_instruction_name(&instruction).to_owned() + " instruction is not supported yet.");
						self.dump_instruction(instruction_address);
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
						self.dump_instruction(instruction_address);
						panic!();
					}
				};
			},
			InstructionFormat::R => {
				let rd = (word >> 7) & 0x1f; // [11:7]
				let rs1 = (word >> 15) & 0x1f; // [19:15]
				let rs2 = (word >> 20) & 0x1f; // [24:20]
				let rs3 = (word >> 27) & 0x1f; //[31:27]
				match instruction {
					Instruction::ADDW => {
						self.x[rd as usize] = self.x[rs1 as usize].wrapping_add(self.x[rs2 as usize]) as i32 as i64;
					},
					Instruction::AMOADDD => {
						let tmp = match self.mmu.load_doubleword(self.unsigned_data(self.x[rs1 as usize])) {
							Ok(data) => data,
							Err(e) => return Err(e)
						};
						match self.mmu.store_doubleword(self.unsigned_data(self.x[rs1 as usize]), self.x[rs2 as usize].wrapping_add(tmp as i64) as u64) {
							Ok(()) => {},
							Err(e) => return Err(e)
						};
						self.x[rd as usize] = tmp as i64;
					},
					Instruction::AMOADDW => {
						let tmp = match self.mmu.load_word(self.unsigned_data(self.x[rs1 as usize])) {
							Ok(data) => data,
							Err(e) => return Err(e)
						};
						match self.mmu.store_word(self.unsigned_data(self.x[rs1 as usize]), self.x[rs2 as usize].wrapping_add(tmp as i64) as u32) {
							Ok(()) => {},
							Err(e) => return Err(e)
						};
						self.x[rd as usize] = tmp as i32 as i64;
					},
					Instruction::AMOANDD => {
						let tmp = match self.mmu.load_doubleword(self.unsigned_data(self.x[rs1 as usize])) {
							Ok(data) => data,
							Err(e) => return Err(e)
						};
						match self.mmu.store_doubleword(self.unsigned_data(self.x[rs1 as usize]), (self.x[rs2 as usize] & (tmp as i64)) as u64) {
							Ok(()) => {},
							Err(e) => return Err(e)
						};
						self.x[rd as usize] = tmp as i32 as i64;
					},
					Instruction::AMOANDW => {
						let tmp = match self.mmu.load_word(self.unsigned_data(self.x[rs1 as usize])) {
							Ok(data) => data,
							Err(e) => return Err(e)
						};
						match self.mmu.store_word(self.unsigned_data(self.x[rs1 as usize]), (self.x[rs2 as usize] & (tmp as i64)) as u32) {
							Ok(()) => {},
							Err(e) => return Err(e)
						};
						self.x[rd as usize] = tmp as i32 as i64;
					},
					Instruction::AMOMAXUW => {
						let tmp = match self.mmu.load_word(self.unsigned_data(self.x[rs1 as usize])) {
							Ok(data) => data,
							Err(e) => return Err(e)
						};
						let max = match self.x[rs2 as usize] as u32 >= tmp {
							true => self.x[rs2 as usize] as u32,
							false => tmp
						};
						match self.mmu.store_word(self.unsigned_data(self.x[rs1 as usize]), max) {
							Ok(()) => {},
							Err(e) => return Err(e)
						};
						self.x[rd as usize] = tmp as i32 as i64;
					},
					Instruction::AMOORD => {
						let tmp = match self.mmu.load_doubleword(self.unsigned_data(self.x[rs1 as usize])) {
							Ok(data) => data,
							Err(e) => return Err(e)
						};
						match self.mmu.store_doubleword(self.unsigned_data(self.x[rs1 as usize]), (self.x[rs2 as usize] | (tmp as i64)) as u64) {
							Ok(()) => {},
							Err(e) => return Err(e)
						};
						self.x[rd as usize] = tmp as i64;
					},
					Instruction::AMOORW => {
						let tmp = match self.mmu.load_word(self.unsigned_data(self.x[rs1 as usize])) {
							Ok(data) => data,
							Err(e) => return Err(e)
						};
						match self.mmu.store_word(self.unsigned_data(self.x[rs1 as usize]), (self.x[rs2 as usize] | tmp as i64) as u32) {
							Ok(()) => {},
							Err(e) => return Err(e)
						};
						self.x[rd as usize] = tmp as i32 as i64;
					},
					Instruction::AMOSWAPD => {
						let tmp = match self.mmu.load_doubleword(self.unsigned_data(self.x[rs1 as usize])) {
							Ok(data) => data,
							Err(e) => return Err(e)
						};
						match self.mmu.store_doubleword(self.unsigned_data(self.x[rs1 as usize]), self.x[rs2 as usize] as u64) {
							Ok(()) => {},
							Err(e) => return Err(e)
						};
						self.x[rd as usize] = tmp as i64;
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
					Instruction::EBREAK => {
						// @TODO: Implement
					},
					Instruction::ECALL => {
						let exception_type = match self.privilege_mode {
							PrivilegeMode::User => TrapType::EnvironmentCallFromUMode,
							PrivilegeMode::Supervisor => TrapType::EnvironmentCallFromSMode,
							PrivilegeMode::Machine => TrapType::EnvironmentCallFromMMode,
							PrivilegeMode::Reserved => panic!()
						};
						return Err(Trap {
							trap_type: exception_type,
							value: instruction_address
						});
					},
					Instruction::FADDD => {
						self.f[rd as usize] = self.f[rs1 as usize] + self.f[rs2 as usize];
					},
					Instruction::FCVTDL => {
						self.f[rd as usize] = self.x[rs1 as usize] as f64;
					},
					Instruction::FCVTDS => {
						// @TODO: Implement properly
						self.f[rd as usize] = f32::from_bits(self.f[rs1 as usize].to_bits() as u32) as f64;
					},
					Instruction::FCVTDW => {
						self.f[rd as usize] = self.x[rs1 as usize] as i32 as f64;
					},
					Instruction::FCVTDWU => {
						self.f[rd as usize] = self.x[rs1 as usize] as u32 as f64;
					},
					Instruction::FCVTSD => {
						self.f[rd as usize] = f32::from_bits(self.f[rs1 as usize].to_bits() as u32) as f64;
					},
					Instruction::FCVTWD => {
						self.x[rd as usize] = self.f[rs1 as usize] as u32 as i32 as i64;
					},
					Instruction::FDIVD => {
						self.f[rd as usize] = match self.f[rs2 as usize] == 0.0 {
							true => 0.0, // @TODO: Implement properly
							false => self.f[rs1 as usize] / self.f[rs2 as usize]
						};
					},
					Instruction::FEQD => {
						self.x[rd as usize] = match self.f[rs1 as usize] == self.f[rs2 as usize] {
							true => 1,
							false => 0
						};
					},
					Instruction::FLED => {
						self.x[rd as usize] = match self.f[rs1 as usize] <= self.f[rs2 as usize] {
							true => 1,
							false => 0
						};
					},
					Instruction::FLTD => {
						self.x[rd as usize] = match self.f[rs1 as usize] < self.f[rs2 as usize] {
							true => 1,
							false => 0
						};
					},
					Instruction::FMADDD => {
						self.f[rd as usize] = self.f[rs1 as usize] * self.f[rs2 as usize] + self.f[rs3 as usize];
					},
					Instruction::FMULD => {
						self.f[rd as usize] = self.f[rs1 as usize] * self.f[rs2 as usize];
					},
					Instruction::FMVDX => {
						self.f[rd as usize] = f64::from_bits(self.x[rs1 as usize] as u64);
					},
					Instruction::FMVXD => {
						self.x[rd as usize] = self.f[rs1 as usize].to_bits() as i64;
					},
					Instruction::FMVXW => {
						self.x[rd as usize] = self.f[rs1 as usize].to_bits() as u32 as i32 as i64;
					},
					Instruction::FMVWX => {
						self.f[rd as usize] = f64::from_bits(self.x[rs1 as usize] as u32 as u64);
					},
					Instruction::FNMSUBD => {
						self.f[rd as usize] = -(self.f[rs1 as usize] * self.f[rs2 as usize]) + self.f[rs3 as usize];
					},
					Instruction::FSGNJD => {
						// @TODO: Confirm this logic is correct
						let rs1_bits = self.f[rs1 as usize].to_bits();
						let rs2_bits = self.f[rs2 as usize].to_bits();
						let sign_bit = rs2_bits & 0x8000000000000000;
						self.f[rd as usize] = f64::from_bits(sign_bit | (rs1_bits & 0x7fffffffffffffff));
					},
					Instruction::FSGNJXD => {
						// @TODO: Confirm this logic is correct
						let rs1_bits = self.f[rs1 as usize].to_bits();
						let rs2_bits = self.f[rs2 as usize].to_bits();
						let sign_bit = (rs1_bits ^ rs2_bits) & 0x8000000000000000;
						self.f[rd as usize] = f64::from_bits(sign_bit | (rs1_bits & 0x7fffffffffffffff));
					},
					Instruction::FSUBD => {
						self.f[rd as usize] = self.f[rs1 as usize] - self.f[rs2 as usize];
					},
					Instruction::LRD => {
						// @TODO: Implement properly
						self.x[rd as usize] = match self.mmu.load_doubleword(self.x[rs1 as usize] as u64) {
							Ok(data) => {
								self.is_reservation_set = true;
								self.reservation = self.x[rs1 as usize] as u64; // Is virtual address ok?
								data as i64
							},
							Err(e) => return Err(e)
						};
					},
					Instruction::LRW => {
						// @TODO: Implement properly
						self.x[rd as usize] = match self.mmu.load_word(self.x[rs1 as usize] as u64) {
							Ok(data) => {
								self.is_reservation_set = true;
								self.reservation = self.x[rs1 as usize] as u64; // Is virtual address ok?
								data as i32 as i64
							},
							Err(e) => return Err(e)
						};
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
								let status = self.read_csr_raw(CSR_MSTATUS_ADDRESS);
								let mpie = (status >> 7) & 1;
								let mpp = (status >> 11) & 0x3;
								// Override MIE[3] with MPIE[7], set MPIE[7] to 1, set MPP[12:11] to 0
								let new_status = (status & !0x1888) | (mpie << 3) | (1 << 7);
								self.write_csr_raw(CSR_MSTATUS_ADDRESS, new_status);
								self.privilege_mode = match mpp {
									0 => PrivilegeMode::User,
									1 => PrivilegeMode::Supervisor,
									3 => PrivilegeMode::Machine,
									_ => panic!() // Shouldn't happen
								};
							},
							Instruction::SRET => {
								let status = self.read_csr_raw(CSR_SSTATUS_ADDRESS);
								let spie = (status >> 5) & 1;
								let spp = (status >> 8) & 1;
								// Override SIE[1] with SPIE[5], set SPIE[5] to 1, set SPP[8] to 0
								let new_status = (status & !0x122) | (spie << 1) | (1 << 5);
								self.write_csr_raw(CSR_SSTATUS_ADDRESS, new_status);
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
					Instruction::SCD => {
						// @TODO: Implement properly
						match self.is_reservation_set && self.reservation == (self.x[rs1 as usize] as u64) {
							true => match self.mmu.store_doubleword(self.x[rs1 as usize] as u64, self.x[rs2 as usize] as u64) {
								Ok(()) => {
									self.x[rd as usize] = 0;
									self.is_reservation_set = false;
								},
								Err(e) => return Err(e)
							},
							false => {
								self.x[rd as usize] = 1;
							}
						};
					},
					Instruction::SCW => {
						// @TODO: Implement properly
						match self.is_reservation_set && self.reservation == (self.x[rs1 as usize] as u64) {
							true => match self.mmu.store_word(self.x[rs1 as usize] as u64, self.x[rs2 as usize] as u32) {
								Ok(()) => {
									self.x[rd as usize] = 0;
									self.is_reservation_set = false;
								},
								Err(e) => return Err(e)
							},
							false => {
								self.x[rd as usize] = 1;
							}
						};
					},
					Instruction::SFENCEVMA => {
						// @TODO: Implement
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
					Instruction::WFI => {
						self.wfi = true;
					},
					Instruction::XOR => {
						self.x[rd as usize] = self.sign_extend(self.x[rs1 as usize] ^ self.x[rs2 as usize]);
					},
					_ => {
						println!("{}", get_instruction_name(&instruction).to_owned() + " instruction is not supported yet.");
						self.dump_instruction(instruction_address);
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
					Instruction::FSD => {
						match self.mmu.store_doubleword(self.x[rs1 as usize].wrapping_add(imm) as u64, self.f[rs2 as usize].to_bits()) {
							Ok(()) => {},
							Err(e) => return Err(e)
						};
					},
					Instruction::FSW => {
						match self.mmu.store_word(self.x[rs1 as usize].wrapping_add(imm) as u64, self.f[rs2 as usize].to_bits() as u32) {
							Ok(()) => {},
							Err(e) => return Err(e)
						};
					},
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
						self.dump_instruction(instruction_address);
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
						self.x[rd as usize] = self.sign_extend(instruction_address.wrapping_add(imm) as i64);
					},
					Instruction::LUI => {
						self.x[rd as usize] = imm as i64;
					}
					_ => {
						println!("{}", get_instruction_name(&instruction).to_owned() + " instruction is not supported yet.");
						self.dump_instruction(instruction_address);
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
		let mut word = match self.mmu.fetch_word(v_address) {
			Ok(data) => data,
			Err(_e) => {
				let s = format!("PC:{:016x}, InstructionPageFault Trap!\n", v_address);
				self.put_bytes_to_terminal(s.as_bytes());
				return;
			}
		};
		let instruction = match self.decode(word) {
			Ok(instruction) => instruction,
			Err(()) => match self.decode(self.uncompress(word & 0xffff)) {
				Ok(instruction) => {
					word = word & 0xffff;
					instruction
				},
				Err(()) => {
					println!("Unknown instruction PC:{:x} WORD:{:x}", self.pc, word);
					self.dump_instruction(self.pc);
					panic!();
				}
			}
		};
		let s = format!("PC:{:016x}, Word:{:08x}, Inst:{}\n",
			self.unsigned_data(v_address as i64),
			word, get_instruction_name(&instruction));
		self.put_bytes_to_terminal(s.as_bytes());
	}

	pub fn put_bytes_to_terminal(&mut self, bytes: &[u8]) {
		for i in 0..bytes.len() {
			self.get_mut_terminal().put_byte(bytes[i]);
		}
	}

	pub fn get_mut_terminal(&mut self) -> &mut Box<dyn Terminal> {
		self.mmu.get_mut_uart().get_mut_terminal()
	}
}

// @TODO: Rename to Instruction
struct InstructionData {
	mask: u32,
	data: u32, // @TODO: rename
	name: &'static str,
	operation: fn(cpu: &mut Cpu, word: u32) -> Result<(), Trap>
}

struct FormatI {
	rd: usize,
	rs1: usize,
	imm: i64
}

fn parse_format_i (word: u32) -> FormatI {
	FormatI {
		rd: ((word >> 7) & 0x1f) as usize, // [11:7]
		rs1: ((word >> 15) & 0x1f) as usize, // [19:15]
		imm: (
			match word & 0x80000000 { // imm[31:11] = [31]
				0x80000000 => 0xfffff800,
				_ => 0
			} |
			((word >> 20) & 0x000007ff) // imm[10:0] = [30:20]
		) as i32 as i64
	}
}

struct FormatR {
	rd: usize,
	rs1: usize,
	rs2: usize
}

fn parse_format_r (word: u32) -> FormatR {
	FormatR {
		rd: ((word >> 7) & 0x1f) as usize, // [11:7]
		rs1: ((word >> 15) & 0x1f) as usize, // [19:15]
		rs2: ((word >> 20) & 0x1f) as usize // [24:20]
	}
}

const INSTRUCTION_NUM: usize = 4;

// @TODO: Reorder in often used order as 
// @TODO: Move all the instructions to INSTRUCTIONS from the current decode() and operate()
const INSTRUCTIONS: [InstructionData; INSTRUCTION_NUM] = [
	InstructionData {
		mask: 0xfe00707f,
		data: 0x00000033,
		name: "ADD",
		operation: |cpu, word| {
			let f = parse_format_r(word);
			cpu.x[f.rd] = cpu.sign_extend(cpu.x[f.rs1].wrapping_add(cpu.x[f.rs2]));
			Ok(())
		}
	},
	InstructionData {
		mask: 0x0000707f,
		data: 0x00000013,
		name: "ADDI",
		operation: |cpu, word| {
			let f = parse_format_i(word);
			cpu.x[f.rd] = cpu.sign_extend(cpu.x[f.rs1].wrapping_add(f.imm));
			Ok(())
		}
	},
	InstructionData {
		mask: 0x0000707f,
		data: 0x0000001b,
		name: "ADDIW",
		operation: |cpu, word| {
			let f = parse_format_i(word);
			cpu.x[f.rd] = cpu.x[f.rs1].wrapping_add(f.imm) as i32 as i64;
			Ok(())
		}
	},
	InstructionData {
		mask: 0xfe00707f,
		data: 0x40000033,
		name: "SUB",
		operation: |cpu, word| {
			let f = parse_format_r(word);
			cpu.x[f.rd] = cpu.sign_extend(cpu.x[f.rs1].wrapping_sub(cpu.x[f.rs2]));
			Ok(())
		}
	},
];