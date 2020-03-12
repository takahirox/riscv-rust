use display::Display;
use std::fs::File;
use std::io::Read;

const XV6: bool = true; // @TODO: Remove this flag
// @TODO: temporal
const TEST_MEMORY_CAPACITY: usize = 1024 * 512;
const XV6_MEMORY_CAPACITY: usize = 1024 * 1024 * 128;
const CSR_CAPACITY: usize = 4096;
const DRAM_BASE: usize = 0x80000000;
const TOHOST_ADDRESS: usize = 0x80001000;

const CSR_USTATUS_ADDRESS: u16 = 0x000;
const CSR_UIR_ADDRESS: u16 = 0x004;
const CSR_UTVEC_ADDRESS: u16 = 0x005;
const CSR_USCRATCH_ADDRESS: u16 = 0x040;
const CSR_UEPC_ADDRESS: u16 = 0x041;
const CSR_UCAUSE_ADDRESS: u16 = 0x042;
const CSR_UTVAL_ADDRESS: u16 = 0x043;
const CSR_UIP_ADDRESS: u16 = 0x044;
const CSR_SSTATUS_ADDRESS: u16 = 0x100;
const CSR_SEDELEG_ADDRESS: u16 = 0x102;
const CSR_SIDELEG_ADDRESS: u16 = 0x103;
const CSR_STVEC_ADDRESS: u16 = 0x105;
const CSR_SSCRATCH_ADDRESS: u16 = 0x140;
const CSR_SEPC_ADDRESS: u16 = 0x141;
const CSR_SCAUSE_ADDRESS: u16 = 0x142;
const CSR_STVAL_ADDRESS: u16 = 0x143;
const CSR_SATP_ADDRESS: u16 = 0x180;
const CSR_MSTATUS_ADDRESS: u16 = 0x300;
const CSR_MEDELEG_ADDRESS: u16 = 0x302;
const CSR_MIDELEG_ADDRESS: u16 = 0x303;
const CSR_MIE_ADDRESS: u16 = 0x304;
const CSR_MTVEC_ADDRESS: u16 = 0x305;
const CSR_MSCRATCH_ADDRESS: u16 = 0x340;
const CSR_MEPC_ADDRESS: u16 = 0x341;
const CSR_MCAUSE_ADDRESS: u16 = 0x342;
const CSR_MTVAL_ADDRESS: u16 = 0x343;
const CSR_PMPCFG0_ADDRESS: u16 = 0x3a0;
const CSR_PMPADDR0_ADDRESS: u16 = 0x3b0;
const CSR_MHARTID_ADDRESS: u16 = 0xf14;

pub struct Cpu {
	clock: u64,
	plic_enabled: bool,
	dump_flag: bool,
	xlen: Xlen,
	privilege_mode: PrivilegeMode,
	addressing_mode: AddressingMode,
	interrupt: InterruptType,
	ppn: u64,
	// using only lower 32bits of x, pc, and csr registers
	// for 32-bit mode
	x: [i64; 32],
	pc: u64,
	csr: [u64; CSR_CAPACITY],
	memory: Vec<u8>,
	disk: VirtioBlockDisk,
	uart: Uart,
	plic: Plic
}

pub enum Xlen {
	Bit32,
	Bit64
	// @TODO: Support Bit128
}

enum AddressingMode {
	None,
	SV32,
	SV39,
	SV48
}

enum PrivilegeMode {
	User,
	Supervisor,
	Reserved,
	Machine
}

enum ExceptionType {
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

struct Trap {
	exception_type: ExceptionType,
	address: u64
}

enum InterruptType {
	None,
	KeyInput,
	Timer,
	Virtio
}

enum MemoryAccessType {
	Execute,
	Read,
	Write
}

enum Instruction {
	ADD,
	ADDI,
	ADDIW,
	ADDW,
	AMOSWAP_W,
	AND,
	ANDI,
	AUIPC,
	BEQ,
	BGE,
	BGEU,
	BLT,
	BLTU,
	BNE,
	CSRRS,
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
	SFENCE_VMA,
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

fn get_xlen_width(xlen: &Xlen) -> u8 {
	match xlen {
		Xlen::Bit32 => 32,
		Xlen::Bit64 => 64
	}
}

fn get_addressing_mode_name(mode: &AddressingMode) -> &'static str {
	match mode {
		AddressingMode::None => "None",
		AddressingMode::SV32 => "SV32",
		AddressingMode::SV39 => "SV39",
		AddressingMode::SV48 => "SV48"
	}
}

fn get_privilege_mode_name(mode: &PrivilegeMode) -> &'static str {
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

fn get_trap_cause(trap: &Trap, xlen: &Xlen) -> u64 {
	let interrupt_bit = match xlen {
		Xlen::Bit32 => 0x80000000 as u64,
		Xlen::Bit64 => 0x8000000000000000 as u64,
	};
	match trap.exception_type {
		ExceptionType::InstructionAddressMisaligned => 0,
		ExceptionType::InstructionAccessFault => 1,
		ExceptionType::IllegalInstruction => 2,
		ExceptionType::Breakpoint => 3,
		ExceptionType::LoadAddressMisaligned => 4,
		ExceptionType::LoadAccessFault => 5,
		ExceptionType::StoreAddressMisaligned => 6,
		ExceptionType::StoreAccessFault => 7,
		ExceptionType::EnvironmentCallFromUMode => 8,
		ExceptionType::EnvironmentCallFromSMode => 9,
		ExceptionType::EnvironmentCallFromMMode => 11,
		ExceptionType::InstructionPageFault => 12,
		ExceptionType::LoadPageFault => 13,
		ExceptionType::StorePageFault => 15,
		ExceptionType::UserSoftwareInterrupt => interrupt_bit,
		ExceptionType::SupervisorSoftwareInterrupt => interrupt_bit + 1,
		ExceptionType::MachineSoftwareInterrupt => interrupt_bit + 3,
		ExceptionType::UserTimerInterrupt => interrupt_bit + 4,
		ExceptionType::SupervisorTimerInterrupt => interrupt_bit + 5,
		ExceptionType::MachineTimerInterrupt => interrupt_bit + 7,
		ExceptionType::UserExternalInterrupt => interrupt_bit + 8,
		ExceptionType::SupervisorExternalInterrupt => interrupt_bit + 9,
		ExceptionType::MachineExternalInterrupt => interrupt_bit + 11
	}
}

fn get_interrupt_privilege_mode(trap: &Trap) -> PrivilegeMode {
	match trap.exception_type {
		ExceptionType::MachineSoftwareInterrupt |
		ExceptionType::MachineTimerInterrupt |
		ExceptionType::MachineExternalInterrupt => PrivilegeMode::Machine,
		ExceptionType::SupervisorSoftwareInterrupt |
		ExceptionType::SupervisorTimerInterrupt |
		ExceptionType::SupervisorExternalInterrupt => PrivilegeMode::Supervisor,
		ExceptionType::UserSoftwareInterrupt |
		ExceptionType::UserTimerInterrupt |
		ExceptionType::UserExternalInterrupt => PrivilegeMode::User,
		_ => panic!() // other exception types are not interrupt
	}
}

fn get_instruction_name(instruction: &Instruction) -> &'static str {
	match instruction {
		Instruction::ADD => "ADD",
		Instruction::ADDI => "ADDI",
		Instruction::ADDIW => "ADDIW",
		Instruction::ADDW => "ADDW",
		Instruction::AMOSWAP_W => "AMOSWAP.W",
		Instruction::AND => "AND",
		Instruction::ANDI => "ANDI",
		Instruction::AUIPC => "AUIPC",
		Instruction::BEQ => "BEQ",
		Instruction::BGE => "BGE",
		Instruction::BGEU => "BGEU",
		Instruction::BLT => "BLT",
		Instruction::BLTU => "BLTU",
		Instruction::BNE => "BNE",
		Instruction::CSRRS => "CSRRS",
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
		Instruction::SFENCE_VMA => "SFENCE_VMA",
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
		Instruction::CSRRS |
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
		Instruction::AMOSWAP_W |
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
		Instruction::SFENCE_VMA |
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
	pub fn new(xlen: Xlen, display: Box<Display>) -> Self {
		let memory_capacity = match XV6 {
			true => XV6_MEMORY_CAPACITY,
			false => TEST_MEMORY_CAPACITY
		};
		let mut cpu = Cpu {
			clock: 0,
			plic_enabled: false,
			dump_flag: false,
			xlen: xlen,
			privilege_mode: PrivilegeMode::Machine,
			addressing_mode: AddressingMode::None,
			interrupt: InterruptType::None,
			ppn: 0,
			x: [0; 32],
			pc: 0,
			csr: [0; CSR_CAPACITY],
			memory: Vec::with_capacity(memory_capacity),
			disk: VirtioBlockDisk::new(),
			uart: Uart::new(display),
			plic: Plic::new()
		};
		for _i in 0..memory_capacity {
			cpu.memory.push(0);
		}
		cpu
	}

	// @TODO: Move out from cpu.rs
	pub fn init(&mut self, data: Vec<u8>, image_data: Vec<u8>) {
		// analyze elf header
		// check ELF magic number
		if data[0] != 0x7f || data[1] != 0x45 || data[2] != 0x4c || data[3] != 0x46 {
			panic!("This file does not seem ELF file");
		}

		let e_class = data[4];

		let e_width = match e_class {
			1 => 32,
			2 => 64,
			_ => panic!("Unknown e_class:{:X}", e_class)
		};

		let e_endian = data[5];
		let e_elf_version = data[6];
		let e_osabi = data[7];
		let e_abi_version = data[8];

		let mut offset = 0x10;

		let mut e_type = 0 as u64;
		for i in 0..2 {
			e_type |= (data[offset] as u64) << (8 * i);
			offset += 1;
		}

		let mut e_machine = 0 as u64;
		for i in 0..2 {
			e_machine |= (data[offset] as u64) << (8 * i);
			offset += 1;
		}

		let mut e_version = 0 as u64;
		for i in 0..4 {
			e_version |= (data[offset] as u64) << (8 * i);
			offset += 1;
		}

		let mut e_entry = 0 as u64;
		for i in 0..e_width / 8 {
			e_entry |= (data[offset] as u64) << (8 * i);
			offset += 1;
		}

		let mut e_phoff = 0 as u64;
		for i in 0..e_width / 8 {
			e_phoff |= (data[offset] as u64) << (8 * i);
			offset += 1;
		}

		let mut e_shoff = 0 as u64;
		for i in 0..e_width / 8 {
			e_shoff |= (data[offset] as u64) << (8 * i);
			offset += 1;
		}

		let mut e_flags = 0 as u64;
		for i in 0..4 {
			e_flags |= (data[offset] as u64) << (8 * i);
			offset += 1;
		}

		let mut e_ehsize = 0 as u64;
		for i in 0..2 {
			e_ehsize |= (data[offset] as u64) << (8 * i);
			offset += 1;
		}

		let mut e_phentsize = 0 as u64;
		for i in 0..2 {
			e_phentsize |= (data[offset] as u64) << (8 * i);
			offset += 1;
		}

		let mut e_phnum = 0 as u64;
		for i in 0..2 {
			e_phnum |= (data[offset] as u64) << (8 * i);
			offset += 1;
		}

		let mut e_shentsize = 0 as u64;
		for i in 0..2 {
			e_shentsize |= (data[offset] as u64) << (8 * i);
			offset += 1;
		}

		let mut e_shnum = 0 as u64;
		for i in 0..2 {
			e_shnum |= (data[offset] as u64) << (8 * i);
			offset += 1;
		}

		let mut e_shstrndx = 0 as u64;
		for i in 0..2 {
			e_shstrndx |= (data[offset] as u64) << (8 * i);
			offset += 1;
		}

		/*
		println!("ELF:{}", e_width);
		println!("e_endian:{:X}", e_endian);
		println!("e_elf_version:{:X}", e_elf_version);
		println!("e_osabi:{:X}", e_osabi);
		println!("e_abi_version:{:X}", e_abi_version);
		println!("e_type:{:X}", e_type);
		println!("e_machine:{:X}", e_machine);
		println!("e_version:{:X}", e_version);
		println!("e_entry:{:X}", e_entry);
		println!("e_phoff:{:X}", e_phoff);
		println!("e_shoff:{:X}", e_shoff);
		println!("e_flags:{:X}", e_flags);
		println!("e_ehsize:{:X}", e_ehsize);
		println!("e_phentsize:{:X}", e_phentsize);
		println!("e_phnum:{:X}", e_phnum);
		println!("e_shentsize:{:X}", e_shentsize);
		println!("e_shnum:{:X}", e_shnum);
		println!("e_shstrndx:{:X}", e_shstrndx);
		*/

		// analyze program headers
		offset = e_phoff as usize;
		for i in 0..e_phnum {
			let mut p_type = 0 as u64;
			for i in 0..4 {
				p_type |= (data[offset] as u64) << (8 * i);
				offset += 1;
			}

			let mut p_flags = 0 as u64;
			if (e_width == 64) {
				for i in 0..4 {
					p_flags |= (data[offset] as u64) << (8 * i);
					offset += 1;
				}
			}

			let mut p_offset = 0 as u64;
			for i in 0..e_width / 8 {
				p_offset |= (data[offset] as u64) << (8 * i);
				offset += 1;
			}

			let mut p_vaddr = 0 as u64;
			for i in 0..e_width / 8 {
				p_vaddr |= (data[offset] as u64) << (8 * i);
				offset += 1;
			}

			let mut p_paddr = 0 as u64;
			for i in 0..e_width / 8 {
				p_paddr |= (data[offset] as u64) << (8 * i);
				offset += 1;
			}

			let mut p_filesz = 0 as u64;
			for i in 0..e_width / 8 {
				p_filesz |= (data[offset] as u64) << (8 * i);
				offset += 1;
			}

			let mut p_memsz = 0 as u64;
			for i in 0..e_width / 8 {
				p_memsz |= (data[offset] as u64) << (8 * i);
				offset += 1;
			}

			if (e_width == 32) {
				for i in 0..4 {
					p_flags |= (data[offset] as u64) << (8 * i);
					offset += 1;
				}
			}

			let mut p_align = 0 as u64;
			for i in 0..e_width / 8 {
				p_align |= (data[offset] as u64) << (8 * i);
				offset += 1;
			}

			/*
			println!("");
			println!("Program:{:X}", i);
			println!("p_type:{:X}", p_type);
			println!("p_flags:{:X}", p_flags);
			println!("p_offset:{:X}", p_offset);
			println!("p_vaddr:{:X}", p_vaddr);
			println!("p_paddr:{:X}", p_paddr);
			println!("p_filesz:{:X}", p_filesz);
			println!("p_memsz:{:X}", p_memsz);
			println!("p_align:{:X}", p_align);
			println!("p_align:{:X}", p_align);
			*/

			for j in 0..p_filesz as usize {
				self.memory[p_paddr as usize + j - DRAM_BASE] = data[p_offset as usize + j];
			}
		}

		// analyze section headers

		offset = e_shoff as usize;
		for i in 0..e_shnum {
			let mut sh_name = 0 as u64;
			for i in 0..4 {
				sh_name |= (data[offset] as u64) << (8 * i);
				offset += 1;
			}

			let mut sh_type = 0 as u64;
			for i in 0..4 {
				sh_type |= (data[offset] as u64) << (8 * i);
				offset += 1;
			}

			let mut sh_flags = 0 as u64;
			for i in 0..e_width / 8 {
				sh_flags |= (data[offset] as u64) << (8 * i);
				offset += 1;
			}

			let mut sh_addr = 0 as u64;
			for i in 0..e_width / 8 {
				sh_addr |= (data[offset] as u64) << (8 * i);
				offset += 1;
			}

			let mut sh_offset = 0 as u64;
			for i in 0..e_width / 8 {
				sh_offset |= (data[offset] as u64) << (8 * i);
				offset += 1;
			}

			let mut sh_size = 0 as u64;
			for i in 0..e_width / 8 {
				sh_size |= (data[offset] as u64) << (8 * i);
				offset += 1;
			}

			let mut sh_link = 0 as u64;
			for i in 0..4 {
				sh_link |= (data[offset] as u64) << (8 * i);
				offset += 1;
			}

			let mut sh_info = 0 as u64;
			for i in 0..4 {
				sh_info |= (data[offset] as u64) << (8 * i);
				offset += 1;
			}

			let mut sh_addralign = 0 as u64;
			for i in 0..e_width / 8 {
				sh_addralign |= (data[offset] as u64) << (8 * i);
				offset += 1;
			}

			let mut sh_entsize = 0 as u64;
			for i in 0..e_width / 8 {
				sh_entsize |= (data[offset] as u64) << (8 * i);
				offset += 1;
			}

			/*
			println!("");
			println!("Section:{:X}", i);
			println!("sh_name:{:X}", sh_name);
			println!("sh_type:{:X}", sh_type);
			println!("sh_flags:{:X}", sh_flags);
			println!("sh_addr:{:X}", sh_addr);
			println!("sh_offset:{:X}", sh_offset);
			println!("sh_size:{:X}", sh_size);
			println!("sh_link:{:X}", sh_link);
			println!("sh_info:{:X}", sh_info);
			println!("sh_addralign:{:X}", sh_addralign);
			println!("sh_entsize:{:X}", sh_entsize);
			*/

			// @TODO: Implement properly
			if sh_type == 1 && sh_addr >= 0x80000000 && sh_offset > 0 && sh_size > 0 {
				for j in 0..sh_size as usize {
					self.memory[sh_addr as usize + j - DRAM_BASE] = data[sh_offset as usize + j];
				}
			}

		}

		//

		self.disk.init(image_data);

		self.pc = e_entry;
	}

	pub fn run(&mut self) {
		loop {
			self.tick();

			if !XV6 {
				// It seems in riscv-tests ends with end code
				// written to a certain physical memory address
				// (0x80001000 in mose test cases) so checking
				// the data in the address and terminating the test
				// if non-zero data is written.
				// End code 1 seems to mean pass.
				let endcode =
					(self.memory[TOHOST_ADDRESS - DRAM_BASE] as u32) |
					((self.memory[TOHOST_ADDRESS - DRAM_BASE + 1] as u32) << 8) |
					((self.memory[TOHOST_ADDRESS - DRAM_BASE + 2] as u32) << 16) |
					((self.memory[TOHOST_ADDRESS - DRAM_BASE + 3] as u32) << 24);
				if endcode != 0 {
					match endcode {
						1 => println!("Test Passed with {:X}", endcode),
						_ => println!("Test Failed with {:X}", endcode)
					};
					break;
				}
			}
		}
	}

	pub fn tick(&mut self) {
		match self.tick_operate() {
			Ok(()) => {},
			Err(e) => self.handle_exception(e)
		}
		self.disk.tick();
		self.uart.tick();
		self.handle_interrupt();
	}

	// @TODO: Rename
	fn tick_operate(&mut self) -> Result<(), Trap> {
		self.clock = self.clock.wrapping_add(1);
		let word = match self.fetch() {
			Ok(word) => word,
			Err(e) => return Err(e)
		};
		let instruction = self.decode(word);
		// @TODO: Remove if the emulator becomes stable
		if !XV6 {
			println!("PC:{:016x}, Word:{:016x}, Inst:{}",
				self.unsigned_data(self.pc.wrapping_sub(4) as i64),
				word, get_instruction_name(&instruction));
		}
		self.operate(word, instruction)
	}

	// XV6 specific for now
	fn handle_interrupt(&mut self) {
		if !XV6 {
			return;
		}

		// @TODO: Implement correctly
		if self.disk.interrupting {
			match self.interrupt {
				InterruptType::None => {
					self.interrupt = InterruptType::Virtio;
				},
				_ => {}
			}
		}

		if self.uart.interrupting {
			match self.interrupt {
				InterruptType::None => {
					self.interrupt = InterruptType::KeyInput;
				},
				_ => {}
			}
		}

		match self.interrupt {
			InterruptType::Virtio => {
				let result = self.handle_trap(Trap {
					exception_type: ExceptionType::SupervisorExternalInterrupt,
					address: self.pc // dummy
				}, true);
				if result {
					self.plic.update(&self.interrupt);
					self.interrupt = InterruptType::None;
					self.handle_disk_access();
					self.disk.reset_interrupt();
				} else {
					// println!("Virtio interrupt is ignored");
				}
			},
			InterruptType::Timer => panic!(), // should not happen
			InterruptType::KeyInput => {
				let result = self.handle_trap(Trap {
					exception_type: ExceptionType::SupervisorExternalInterrupt,
					address: self.pc // dummy
				}, true);
				if result {
					self.plic.update(&self.interrupt);
					self.interrupt = InterruptType::None;
					self.uart.reset_interrupt();
				} else {
					// println!("Virtio interrupt is ignored");
				}
			},
			InterruptType::None => {
				if self.plic_enabled && (self.clock % 0x200000) == 0 {	// @TODO: Fix me
					self.handle_trap(Trap {
						exception_type: ExceptionType::SupervisorSoftwareInterrupt,
						address: self.pc // dummy
					}, true);
					self.plic.update(&InterruptType::Timer);
				}
			}
		}
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
		let new_privilege_encoding = get_privilege_encoding(&new_privilege_mode) as u64;

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
		self.csr[csr_tval_address as usize] = trap.address;
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

	fn handle_disk_access(&mut self) {
		let avail_address = self.disk.get_avail_address();
		let base_desc_address = self.disk.get_desc_address() as u64;
		let base_used_address = self.disk.get_used_address();

		let flag = match self.load_halfword(avail_address, false) {
			Ok(data) => data,
			Err(_e) => panic!() // shouldn't happen
		};
		let offset = match self.load_halfword(avail_address.wrapping_add(1), false) {
			Ok(data) => data,
			Err(_e) => panic!() // shouldn't happen		
		};
		let index = match self.load_halfword(avail_address.wrapping_add(offset as u64 % 8).wrapping_add(2), false) {
			Ok(data) => data,
			Err(_e) => panic!() // shouldn't happen		
		};
		let desc_size = 16;

		let desc_address0 = base_desc_address + desc_size * index as u64;
		let addr0 = match self.load_doubleword(desc_address0, false) {
			Ok(data) => data,
			Err(_e) => panic!() // shouldn't happen		
		};
		let len0 = match self.load_word(desc_address0.wrapping_add(8), false) {
			Ok(data) => data,
			Err(_e) => panic!() // shouldn't happen		
		};
		let flags0 = match self.load_halfword(desc_address0.wrapping_add(12), false) {
			Ok(data) => data,
			Err(_e) => panic!() // shouldn't happen		
		};
		let next0 = match self.load_halfword(desc_address0.wrapping_add(14), false) {
			Ok(data) => data,
			Err(_e) => panic!() // shouldn't happen		
		};

		let desc_address1 = base_desc_address + desc_size * next0 as u64;
		let addr1 = match self.load_doubleword(desc_address1, false) {
			Ok(data) => data,
			Err(_e) => panic!() // shouldn't happen		
		};
		let len1 = match self.load_word(desc_address1.wrapping_add(8), false) {
			Ok(data) => data,
			Err(_e) => panic!() // shouldn't happen		
		};
		let flags1 = match self.load_halfword(desc_address1.wrapping_add(12), false) {
			Ok(data) => data,
			Err(_e) => panic!() // shouldn't happen		
		};
		let next1 = match self.load_halfword(desc_address1.wrapping_add(14), false) {
			Ok(data) => data,
			Err(_e) => panic!() // shouldn't happen		
		};

		let desc_address2 = base_desc_address + desc_size * next1 as u64;
		let addr2 = match self.load_doubleword(desc_address2, false) {
			Ok(data) => data,
			Err(_e) => panic!() // shouldn't happen		
		};
		let len2 = match self.load_word(desc_address2.wrapping_add(8), false) {
			Ok(data) => data,
			Err(_e) => panic!() // shouldn't happen		
		};
		let flags2 = match self.load_halfword(desc_address2.wrapping_add(12), false) {
			Ok(data) => data,
			Err(_e) => panic!() // shouldn't happen		
		};
		let next2 = match self.load_halfword(desc_address2.wrapping_add(14), false) {
			Ok(data) => data,
			Err(_e) => panic!() // shouldn't happen		
		};

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
		
		let blk_type = match self.load_word(addr0, false) {
			Ok(data) => data,
			Err(_e) => panic!() // shouldn't happen
		};
		let blk_reserved = match self.load_word(addr0.wrapping_add(4), false) {
			Ok(data) => data,
			Err(_e) => panic!() // shouldn't happen
		};
		let blk_sector = match self.load_doubleword(addr0.wrapping_add(8), false) {
			Ok(data) => data,
			Err(_e) => panic!() // shouldn't happen
		};

		/*
		println!("Blk type:{:X}", blk_type);
		println!("Blk reserved:{:X}", blk_reserved);
		println!("Blk sector:{:X}", blk_sector);
		*/

		match (flags1 & 2) == 0 {
			true => { // write to disk
				// println!("Write to disk");
				for i in 0..len1 as u64 {
					let data = match self.load_byte(addr1 + i, false) {
						Ok(data) => data,
						Err(_e) => panic!() // shouldn't happen
					};
					self.disk.write_to_disk(blk_sector * 512 + i, data);
					// print!("{:02X} ", data);
				}
				// println!();
			},
			false => { // read from disk
				// println!("Read from disk");
				for i in 0..len1 as u64 {
					let data = self.disk.read_from_disk(blk_sector * 512 + i);
					match self.store_byte(addr1 + i, data, false) {
						Ok(()) => {},
						Err(_e) => panic!() // shouldn't happen
					};
					// print!("{:02X} ", data);
				}
				// println!();
			}
		};
		
		let new_id = self.disk.get_new_id() as u16;
		self.store_halfword(base_used_address.wrapping_add(2), new_id % 8, false);
	}

	fn fetch(&mut self) -> Result<u32, Trap> {
		let word = match self.fetch_word(self.pc, true) {
			Ok(word) => word,
			Err(_e) => {
				let address = self.pc;
				self.pc = self.pc.wrapping_add(4);
				return Err(Trap {
					exception_type: ExceptionType::InstructionPageFault,
					address: address
				})
			}
		};
		// @TODO: Should I increment pc after operating an instruction because
		// some of the instruction operations need the address of the instruction?
		self.pc = self.pc.wrapping_add(4);
		Ok(word)
	}

	// @TOD: Can we combile with load_word?
	fn fetch_word(&mut self, address: u64, translation: bool) -> Result<u32, Trap> {
		let mut data = 0 as u32;
		for i in 0..4 {
			match self.fetch_byte(address.wrapping_add(i), translation) {
				Ok(byte) => {
					data |= (byte as u32) << (i * 8)
				},
				Err(e) => return Err(e)
			};
		}
		Ok(data)
	}

	// @TOD: Can we combile with load_byte?
	fn fetch_byte(&mut self, address: u64, translation: bool) -> Result<u8, Trap> {
		let p_address = match translation {
			true => match self.translate_address(address, MemoryAccessType::Execute) {
				Ok(address) => address,
				Err(_e) => return Err(Trap {
					exception_type: ExceptionType::InstructionPageFault,
					address: address
				})
			},
			false => address
		};
		Ok(self.load_memory(match self.xlen {
			Xlen::Bit32 => p_address & 0xffffffff,
			Xlen::Bit64 => p_address
		}))
	}

	fn load_doubleword(&mut self, address: u64, translation: bool) -> Result<u64, Trap> {
		let mut data = 0 as u64;
		for i in 0..8 {
			match self.load_byte(address.wrapping_add(i), translation) {
				Ok(byte) => {
					data |= (byte as u64) << (i * 8)
				},
				Err(e) => return Err(e)
			};
		}
		Ok(data)
	}

	fn load_word(&mut self, address: u64, translation: bool) -> Result<u32, Trap> {
		let mut data = 0 as u32;
		for i in 0..4 {
			match self.load_byte(address.wrapping_add(i), translation) {
				Ok(byte) => {
					data |= (byte as u32) << (i * 8)
				},
				Err(e) => return Err(e)
			};
		}
		Ok(data)
	}

	fn load_halfword(&mut self, address: u64, translation: bool) -> Result<u16, Trap> {
		let mut data = 0 as u16;
		for i in 0..2 {
			match self.load_byte(address.wrapping_add(i), translation) {
				Ok(byte) => {
					data |= (byte as u16) << (i * 8)
				},
				Err(e) => return Err(e)
			};
		}
		Ok(data)
	}

	fn load_byte(&mut self, address: u64, translation: bool) -> Result<u8, Trap> {
		let p_address = match translation {
			true => match self.translate_address(address, MemoryAccessType::Read) {
				Ok(address) => address,
				Err(_e) => return Err(Trap {
					exception_type: ExceptionType::LoadPageFault,
					address: address
				})
			},
			false => address
		};
		Ok(self.load_memory(match self.xlen {
			Xlen::Bit32 => p_address & 0xffffffff,
			Xlen::Bit64 => p_address
		}))
	}

	fn load_memory(&mut self, address: u64) -> u8 {
		// @TODO: Implement properly
		// 0x02000000 - 0xXXXXXXXX CLINT
		// 0x0C000000 - 0xXXXXXXXX PLIC
		// 0x10000000 - 0xXXXXXXXX UART0
		// 0x10001000 - 0xXXXXXXXX VIRTIO disk
		if (address as usize) < DRAM_BASE {
			return self.load_from_device(address);
		}
		if (address as usize) < DRAM_BASE {
			println!("Accessing out of address, {:X}", address);
			panic!();
		}
		self.memory[address as usize - DRAM_BASE]
	}

	fn load_from_device(&mut self, address: u64) -> u8 {
		// @TODO: implement
		// 0x200bff8: CLINT Timer register (mtime)
		// println!("Device IO load? PC:{:X} PA:{:X}", self.pc.wrapping_sub(4), address);
		// dummy
		match address {
			0x0c201004 => self.plic.load(address) as u8,
			0x10000000..=0x10000005 => self.uart.load(address),
			0x10001000..=0x10001FFF => self.disk.load(address),
			_ => 0
		}
	}

	fn store_doubleword(&mut self, address: u64, value: u64, translation: bool) -> Result<(), Trap> {
		for i in 0..8 {
			match self.store_byte(address.wrapping_add(i), ((value >> (i * 8)) & 0xff) as u8, translation) {
				Ok(()) => {},
				Err(e) => return Err(e)
			}
		}
		Ok(())
	}

	fn store_word(&mut self, address: u64, value: u32, translation: bool) -> Result<(), Trap> {
		for i in 0..4 {
			match self.store_byte(address.wrapping_add(i), ((value >> (i * 8)) & 0xff) as u8, translation) {
				Ok(()) => {},
				Err(e) => return Err(e)
			}
		}
		Ok(())
	}

	fn store_halfword(&mut self, address: u64, value: u16, translation: bool) -> Result<(), Trap> {
		for i in 0..2 {
			match self.store_byte(address.wrapping_add(i), ((value >> (i * 8)) & 0xff) as u8, translation) {
				Ok(()) => {},
				Err(e) => return Err(e)
			}
		}
		Ok(())
	}

	fn store_byte(&mut self, address: u64, value: u8, translation: bool) -> Result<(), Trap> {
		let p_address = match translation {
			true => match self.translate_address(address, MemoryAccessType::Write) {
				Ok(address) => address,
				Err(_e) => return Err(Trap {
					exception_type: ExceptionType::StorePageFault,
					address: address
				})
			},
			false => address
		};
		self.store_memory(match self.xlen {
			Xlen::Bit32 => p_address & 0xffffffff,
			Xlen::Bit64 => p_address
		}, value);
		Ok(())
	}

	fn store_memory(&mut self, address: u64, value: u8) {
		// @TODO: Implement properly
		if (address as usize) < DRAM_BASE {
			return self.store_to_device(address, value);
		}
		if (address as usize) < DRAM_BASE {
			println!("Accessing out of address, {:X}", address);
			panic!();
		}
		// println!("Store PA:{:X} value:{:X}", address, value);
		self.memory[address as usize - DRAM_BASE] = value;
	}

	fn store_to_device(&mut self, address: u64, value: u8) {
		// println!("Device IO store? PC:{:X} PA:{:X} Value:{:X}", self.pc.wrapping_sub(4), address, value);
		// dummy
		match address {
			0x0c002080 => { // PLIC_SENABLE(hart) (PLIC + 0x2080 + (hart)*0x100)
				self.plic_enabled = true;
			},
			0x10000000..=0x10000005 => {
				self.uart.store(address, value);
			},
			0x10001000..=0x10001FFF => { // @TODO: Check a valid range
				self.disk.store(address, value);
			},
			_ => {}
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
			_ => {
				println!("{} addressing_mode is not supported yet", get_addressing_mode_name(&self.addressing_mode));
				panic!();
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
			AddressingMode::SV32 => match self.load_word(pte_address, false) {
				Ok(data) => data as u64,
				Err(_e) => panic!() // Shouldn't happen
			},
			_ => match self.load_doubleword(pte_address, false) {
				Ok(data) => data,
				Err(_e) => panic!() // Shouldn't happen
			},
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
				AddressingMode::SV32 => match self.store_word(pte_address, new_pte as u32, false) {
					Ok(()) => {},
					Err(_e) => panic!() // Shouldn't happen
				},
				_ => match self.store_doubleword(pte_address, new_pte, false) {
					Ok(()) => {},
					Err(_e) => panic!() // Shouldn't happen
				}
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

	fn has_csr_access_privilege(&self, address: u16) -> bool {
		let privilege = (address >> 8) & 0x3; // the lowest privilege level that can access the CSR
		privilege as u8 <= get_privilege_encoding(&self.privilege_mode)
	}

	fn read_csr(&mut self, address: u16) -> Result<u64, Trap> {
		match self.has_csr_access_privilege(address) {
			true => Ok(self.csr[address as usize]),
			false => Err(Trap {
				exception_type: ExceptionType::IllegalInstruction,
				address: self.pc.wrapping_sub(4) // @TODO: Is this always correct?
			})
		}
	}

	fn write_csr(&mut self, address: u16, value: u64) -> Result<(), Trap> {
		// println!("PC:{:X} Privilege mode:{}", self.pc.wrapping_sub(4), get_privilege_mode_name(&self.privilege_mode));
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
				exception_type: ExceptionType::IllegalInstruction,
				address: self.pc.wrapping_sub(4) // @TODO: Is this always correct?
			})
		}
	}

	fn update_addressing_mode(&mut self, value: u64) {
		self.addressing_mode = match self.xlen {
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
		self.ppn = match self.xlen {
			Xlen::Bit32 => value & 0x3fffff,
			Xlen::Bit64 => value & 0xfffffffffff
		}
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
						1 => Instruction::AMOSWAP_W,
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
						9 => Instruction::SFENCE_VMA,
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
				5 => Instruction::CSRRWI,
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
					Instruction::CSRRS => {
						let data = match self.read_csr(csr) {
							Ok(data) => data,
							Err(e) => return Err(e)
						};
						self.x[rd as usize] = self.sign_extend(data as i64);
						self.x[0] = 0; // hard-wired zero
						match self.write_csr(csr, self.unsigned_data(self.x[rd as usize] | self.x[rs as usize])) {
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
						self.x[rd as usize] = match self.load_byte(self.x[rs1 as usize].wrapping_add(imm) as u64, true) {
							Ok(data) => data as i8 as i64,
							Err(e) => return Err(e)
						};
					},
					Instruction::LBU => {
						self.x[rd as usize] = match self.load_byte(self.x[rs1 as usize].wrapping_add(imm) as u64, true) {
							Ok(data) => data as i64,
							Err(e) => return Err(e)
						};
					},
					Instruction::LD => {
						self.x[rd as usize] = match self.load_doubleword(self.x[rs1 as usize].wrapping_add(imm) as u64, true) {
							Ok(data) => data as i64,
							Err(e) => return Err(e)
						};
					},
					Instruction::LH => {
						self.x[rd as usize] = match self.load_halfword(self.x[rs1 as usize].wrapping_add(imm) as u64, true) {
							Ok(data) => data as i16 as i64,
							Err(e) => return Err(e)
						};
					},
					Instruction::LHU => {
						self.x[rd as usize] = match self.load_halfword(self.x[rs1 as usize].wrapping_add(imm) as u64, true) {
							Ok(data) => data as i64,
							Err(e) => return Err(e)
						};
					},
					Instruction::LW => {
						self.x[rd as usize] = match self.load_word(self.x[rs1 as usize].wrapping_add(imm) as u64, true) {
							Ok(data) => data as i32 as i64,
							Err(e) => return Err(e)
						};
					},
					Instruction::LWU => {
						self.x[rd as usize] = match self.load_word(self.x[rs1 as usize].wrapping_add(imm) as u64, true) {
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
					Instruction::AMOSWAP_W => {
						let tmp = match self.load_word(self.unsigned_data(self.x[rs1 as usize]), true) {
							Ok(data) => data,
							Err(e) => return Err(e)
						};
						match self.store_word(self.unsigned_data(self.x[rs1 as usize]), self.x[rs2 as usize] as u32, true) {
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
							PrivilegeMode::User => ExceptionType::EnvironmentCallFromUMode,
							PrivilegeMode::Supervisor => ExceptionType::EnvironmentCallFromSMode,
							PrivilegeMode::Machine => ExceptionType::EnvironmentCallFromMMode,
							PrivilegeMode::Reserved => panic!()
						};
						return Err(Trap {
							exception_type: exception_type,
							address: self.pc.wrapping_sub(4)
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
								let mie = (status >> 3) & 1;
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
								let sie = (status >> 1) & 1;
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
						// println!("{}", get_privilege_mode_name(&self.privilege_mode));
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
					Instruction::SFENCE_VMA => {
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
						match self.store_byte(self.x[rs1 as usize].wrapping_add(imm) as u64, self.x[rs2 as usize] as u8, true) {
							Ok(()) => {},
							Err(e) => return Err(e)
						};
					},
					Instruction::SH => {
						match self.store_halfword(self.x[rs1 as usize].wrapping_add(imm) as u64, self.x[rs2 as usize] as u16, true) {
							Ok(()) => {},
							Err(e) => return Err(e)
						};
					},
					Instruction::SW => {
						match self.store_word(self.x[rs1 as usize].wrapping_add(imm) as u64, self.x[rs2 as usize] as u32, true) {
							Ok(()) => {},
							Err(e) => return Err(e)
						};
					},
					Instruction::SD => {
						match self.store_doubleword(self.x[rs1 as usize].wrapping_add(imm) as u64, self.x[rs2 as usize] as u64, true) {
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
		let word = match self.load_word(address, true) {
			Ok(word) => word,
			Err(_e) => return // @TODO: What should here do?
		};
		let pc = self.unsigned_data(address as i64);
		let opcode = word & 0x7f; // [6:0]
		println!("Pc:{:016x}, Opcode:{:07b}, Word:{:016x}", pc, opcode, word);
	}
	
	// Wasm specific
	pub fn get_output(&mut self) -> u8 {
		self.uart.get_output()
	}

	pub fn put_input(&mut self, data: u8) {
		self.uart.put_input(data);
	}
}

struct VirtioBlockDisk {
	id: u8,
	clock: u64,
	driver_features: u32,
	guest_page_size: u32,
	queue_select: u32,
	queue_num: u32,
	queue_pfn: u32,
	queue_notify: u32,
	status: u32,
	notify_clock: u64,
	interrupting: bool,
	contents: Vec<u8>
}

impl VirtioBlockDisk {
	fn new() -> Self {
		VirtioBlockDisk {
			id: 0,
			clock: 0,
			driver_features: 0,
			guest_page_size: 0,
			queue_select: 0,
			queue_num: 0,
			queue_pfn: 0,
			queue_notify: 0,
			status: 0,
			notify_clock: 0,
			interrupting: false,
			contents: vec![]
		}
	}

	fn init(&mut self, contents: Vec<u8>) {
		for i in 0..contents.len() {
			self.contents.push(contents[i]);
		}
	}

	fn tick(&mut self) {
		self.clock = self.clock.wrapping_add(1);
		if self.notify_clock > 0 && self.clock > self.notify_clock + 500 {
			self.interrupting = true;
		}
	}

	fn load(&self, address: u64) -> u8 {
		match address {
			0x10001000 => 0x76, // vertio disk magic value: 0x74726976
			0x10001001 => 0x69,
			0x10001002 => 0x72,
			0x10001003 => 0x74,
			0x10001004 => 1, // vertio version: 1
			0x10001008 => 2, // vertio device id: 2
			0x1000100c => 0x51, // vertio vendor id: 0x554d4551
			0x1000100d => 0x45,
			0x1000100e => 0x4d,
			0x1000100f => 0x55,
			0x10001034 => 8, // vertio  queue num max: At least 8
			_ => 0
		}
	}
	
	fn store(&mut self, address: u64, value: u8) {
		match address {
			0x10001020 => {
				self.driver_features = (self.driver_features & !0xff) | (value as u32);
			},
			0x10001021 => {
				self.driver_features = (self.driver_features & !0xff00) | ((value as u32) << 8);
			},
			0x10001022 => {
				self.driver_features = (self.driver_features & !0xff0000) | ((value as u32) << 16);			
			},
			0x10001023 => {
				self.driver_features = (self.driver_features & !0xff000000) | ((value as u32) << 24);
			},
			0x10001028 => {
				self.guest_page_size = (self.guest_page_size & !0xff) | (value as u32);
			},
			0x10001029 => {
				self.guest_page_size = (self.guest_page_size & !0xff00) | ((value as u32) << 8);
			},
			0x1000102a => {
				self.guest_page_size = (self.guest_page_size & !0xff0000) | ((value as u32) << 16);			
			},
			0x1000102b => {
				self.guest_page_size = (self.guest_page_size & !0xff000000) | ((value as u32) << 24);
			},
			0x10001030 => {
				self.queue_select = (self.queue_select & !0xff) | (value as u32);
			},
			0x10001031 => {
				self.queue_select = (self.queue_select & !0xff00) | ((value as u32) << 8);
			},
			0x10001032 => {
				self.queue_select = (self.queue_select & !0xff0000) | ((value as u32) << 16);			
			},
			0x10001033 => {
				self.queue_select = (self.queue_select & !0xff000000) | ((value as u32) << 24);
			},
			0x10001038 => {
				self.queue_num = (self.queue_num & !0xff) | (value as u32);
			},
			0x10001039 => {
				self.queue_num = (self.queue_num & !0xff00) | ((value as u32) << 8);
			},
			0x1000103a => {
				self.queue_num = (self.queue_num & !0xff0000) | ((value as u32) << 16);			
			},
			0x1000103b => {
				self.queue_num = (self.queue_num & !0xff000000) | ((value as u32) << 24);
			},
			0x10001040 => {
				self.queue_pfn = (self.queue_pfn & !0xff) | (value as u32);
			},
			0x10001041 => {
				self.queue_pfn = (self.queue_pfn & !0xff00) | ((value as u32) << 8);
			},
			0x10001042 => {
				self.queue_pfn = (self.queue_pfn & !0xff0000) | ((value as u32) << 16);			
			},
			0x10001043 => {
				self.queue_pfn = (self.queue_pfn & !0xff000000) | ((value as u32) << 24);
			},
			0x10001050 => {
				self.queue_notify = (self.queue_notify & !0xff) | (value as u32);
			},
			0x10001051 => {
				self.queue_notify = (self.queue_notify & !0xff00) | ((value as u32) << 8);
			},
			0x10001052 => {
				self.queue_notify = (self.queue_notify & !0xff0000) | ((value as u32) << 16);			
			},
			0x10001053 => {
				self.queue_notify = (self.queue_notify & !0xff000000) | ((value as u32) << 24);
				self.notify_clock = self.clock;
			},
			0x10001070 => {
				self.status = (self.status & !0xff) | (value as u32);
			},
			0x10001071 => {
				self.status = (self.status & !0xff00) | ((value as u32) << 8);
			},
			0x10001072 => {
				self.status = (self.status & !0xff0000) | ((value as u32) << 16);			
			},
			0x10001073 => {
				self.status = (self.status & !0xff000000) | ((value as u32) << 24);
			},
			_ => {}
		};
	}

	fn get_page_address(&self) -> u64 {
		self.queue_pfn as u64 * self.guest_page_size as u64
	}

	// desc = pages -- num * VRingDesc
	// avail = pages + 0x40 -- 2 * uint16, then num * uint16
	// used = pages + 4096 -- 2 * uint16, then num * vRingUsedElem
	
	fn get_desc_address(&self) -> u64 {
		self.get_page_address()
	}

	fn get_avail_address(&self) -> u64 {
		self.get_page_address() + 0x40
	}

	fn get_used_address(&self) -> u64 {
		self.get_page_address() + 4096
	}

	fn read_from_disk(&mut self, address: u64) -> u8 {
		self.contents[address as usize]
	}
	
	fn write_to_disk(&mut self, address: u64, value: u8) {
		self.contents[address as usize] = value
	}

	fn get_new_id(&mut self) -> u8 {
		self.id += 1;
		self.id
	}

	// @TODO: Rename
	fn reset_interrupt(&mut self) {
		self.interrupting = false;
		self.notify_clock = 0;
	}
}

struct Uart {
	clock: u64,
	receive_register: u8,
	transmit_register: u8,
	line_status_register: u8,
	interrupting: bool,
	display: Box<Display>
}

impl Uart {
	fn new(display: Box<Display>) -> Self {
		Uart {
			clock: 0,
			receive_register: 0,
			transmit_register: 0,
			line_status_register: 0x20,
			interrupting: false,
			display: display
		}
	}

	fn tick(&mut self) {
		self.clock = self.clock.wrapping_add(1);
		if (self.clock % 0x10000) == 0 && !self.interrupting {
			let value = self.display.get_input();
			if value != 0 {
				self.interrupting = true;
				self.receive_register = value;
				self.line_status_register = 1;
			}
		}
	}

	fn load(&mut self, address: u64) -> u8 {
		match address {
			0x10000000 => {
				let value = self.receive_register;
				self.receive_register = 0x0;
				self.line_status_register = 0x20;
				value
			},
			0x10000005 => self.line_status_register, // UART0 LSR
			_ => 0
		}
	}

	fn store(&mut self, address: u64, value: u8) {
		match address {
			0x10000000 => { // UART0 THR
				self.display.put_byte(value);
			},
			_ => {}
		};
	}
	
	// @TODO: Rename
	fn reset_interrupt(&mut self) {
		self.interrupting = false;
	}

	// Wasm specific

	fn get_output(&mut self) -> u8 {
		self.display.get_output()
	}
	
	fn put_input(&mut self, data: u8) {
		self.display.put_input(data);
	}
}

struct Plic {
	irq: u32
}

impl Plic {
	fn new() -> Self {
		Plic {
			irq: 0
		}
	}

	fn update(&mut self, interrupt_type: &InterruptType) {
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

	fn load(&self, address: u64) -> u32 {
		match address {
			0x0c201004 => self.irq, // PLIC_SCLAIM(hart) (PLIC + 0x201004 + (hart)*0x2000)
			_ => 0
		}
	}
}