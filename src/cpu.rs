const MEMORY_CAPACITY: usize = 1024 * 16; // @TODO: temporal
const CSR_CAPACITY: usize = 4096;

pub struct Cpu {
	xlen: Xlen,
	// using only lower 32bits of x and pc registers
	// for 32-bit mode
	x: [i64; 32],
	pc: u64,
	csr: [u64; CSR_CAPACITY], // @TODO: Check the width of csr
	memory: [u8; MEMORY_CAPACITY]
}

pub enum Xlen {
	Bit32,
	Bit64
	// @TODO: Support Bit128
}

enum Instruction {
	ADD,
	ADDI,
	ADDIW,
	ADDW,
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
	MRET,
	OR,
	ORI,
	SB,
	SD,
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
	SRL,
	SRLI,
	SRLIW,
	SRLW,
	SUB,
	SUBW,
	SW,
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

fn get_instruction_name(instruction: &Instruction) -> &'static str {
	match instruction {
		Instruction::ADD => "ADD",
		Instruction::ADDI => "ADDI",
		Instruction::ADDIW => "ADDIW",
		Instruction::ADDW => "ADDW",
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
		Instruction::OR => "OR",
		Instruction::ORI => "ORI",
		Instruction::SB => "SB",
		Instruction::SD => "SD",
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
		Instruction::SRL => "SRL",
		Instruction::SRLI => "SRLI",
		Instruction::SRLIW => "SRLIW",
		Instruction::SRLW => "SRLW",
		Instruction::SUB => "SUB",
		Instruction::SUBW => "SUBW",
		Instruction::SW => "SW",
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
		Instruction::AND |
		Instruction::ECALL |
		Instruction::MRET |
		Instruction::MUL |
		Instruction::MULH |
		Instruction::MULHU |
		Instruction::OR |
		Instruction::SUB |
		Instruction::SUBW |
		Instruction::SLL |
		Instruction::SLLW |
		Instruction::SLT |
		Instruction::SLTU |
		Instruction::SRA |
		Instruction::SRAW |
		Instruction::SRL |
		Instruction::SRLW |
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
	pub fn new(xlen: Xlen) -> Self {
		Cpu {
			xlen: xlen,
			x: [0; 32],
			pc: 0,
			csr: [0; CSR_CAPACITY],
			memory: [0; MEMORY_CAPACITY]
		}
	}

	pub fn run_test(&mut self, data: Vec<u8>) {
		for i in 0..data.len() {
			self.memory[i] = data[i];
		}
		self.pc = 0;
		loop {
			// @TODO: Temporal termination check, ends at ECALL instruction.
			// I should check more properly.
			let terminate = match self.load_word(self.pc) {
				0x00000073 => true,
				_ => false
			};
			self.tick();
			if terminate {
				// @TODO: Check if this condition is true
				match self.x[10] {
					0 => println!("Test Passed"),
					_ => println!("Test Failed")
				};
				break;
			}
		}
	}

	pub fn tick(&mut self) {
		let word = self.fetch();
		let instruction = self.decode(word);
		// @TODO: Remove if the emulator becomes stable
		println!("PC:{:08x}, Word:{:08x}, Inst:{}",
			self.pc.wrapping_sub(4), word, get_instruction_name(&instruction));
		self.operate(word, instruction);
	}

	fn fetch(&mut self) -> u32 {
		let word = self.load_word(self.pc);
		// @TODO: Should I increment pc after operating an instruction because
		// some of the instruction operations need the address of the instruction?
		self.pc = self.pc.wrapping_add(4);
		word
	}

	fn load_doubleword(&self, address: u64) -> u64 {
		((self.load_byte(address.wrapping_add(7)) as u64) << 56) |
		((self.load_byte(address.wrapping_add(6)) as u64) << 48) |
		((self.load_byte(address.wrapping_add(5)) as u64) << 40) |
		((self.load_byte(address.wrapping_add(4)) as u64) << 32) |
		((self.load_byte(address.wrapping_add(3)) as u64) << 24) |
		((self.load_byte(address.wrapping_add(2)) as u64) << 16) |
		((self.load_byte(address.wrapping_add(1)) as u64) << 8) |
		(self.load_byte(address) as u64)
	}

	fn load_word(&self, address: u64) -> u32 {
		((self.load_byte(address.wrapping_add(3)) as u32) << 24) |
		((self.load_byte(address.wrapping_add(2)) as u32) << 16) |
		((self.load_byte(address.wrapping_add(1)) as u32) << 8) |
		(self.load_byte(address) as u32)
	}

	fn load_halfword(&self, address: u64) -> u16 {
		((self.load_byte(address.wrapping_add(1)) as u16) << 8) |
		(self.load_byte(address) as u16)
	}

	fn load_byte(&self, address: u64) -> u8 {
		self.memory[match self.xlen {
			Xlen::Bit32 => address & 0xffffffff,
			Xlen::Bit64 => address
		} as usize]
	}

	fn store_doubleword(&mut self, address: u64, value: u64) {
		self.store_byte(address, (value & 0xff) as u8);
		self.store_byte(address.wrapping_add(1), ((value >> 8) & 0xff) as u8);
		self.store_byte(address.wrapping_add(2), ((value >> 16) & 0xff) as u8);
		self.store_byte(address.wrapping_add(3), ((value >> 24) & 0xff) as u8);
		self.store_byte(address.wrapping_add(4), ((value >> 32) & 0xff) as u8);
		self.store_byte(address.wrapping_add(5), ((value >> 40) & 0xff) as u8);
		self.store_byte(address.wrapping_add(6), ((value >> 48) & 0xff) as u8);
		self.store_byte(address.wrapping_add(7), ((value >> 56) & 0xff) as u8);
	}

	fn store_word(&mut self, address: u64, value: u32) {
		self.store_byte(address, (value & 0xff) as u8);
		self.store_byte(address.wrapping_add(1), ((value >> 8) & 0xff) as u8);
		self.store_byte(address.wrapping_add(2), ((value >> 16) & 0xff) as u8);
		self.store_byte(address.wrapping_add(3), ((value >> 24) & 0xff) as u8);
	}

	fn store_halfword(&mut self, address: u64, value: u16) {
		self.store_byte(address, (value & 0xff) as u8);
		self.store_byte(address.wrapping_add(1), ((value >> 8) & 0xff) as u8);
	}

	fn store_byte(&mut self, address: u64, value: u8) {
		self.memory[match self.xlen {
			Xlen::Bit32 => address & 0xffffffff,
			Xlen::Bit64 => address
		} as usize] = value;
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

	fn decode(&self, word: u32) -> Instruction {
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
				2 => Instruction::SLT,
				3 => match funct7 {
					0 => Instruction::SLTU,
					1 => Instruction::MULHU,
					_ => {
						println!("Unknown funct7: {:07b}", funct7);
						self.dump_instruction(self.pc.wrapping_sub(4));
						panic!();
					}
				},
				4 => Instruction::XOR,
				5 => match funct7 {
					0 => Instruction::SRL,
					0x20 => Instruction::SRA,
					_ => {
						println!("Unknown funct7: {:07b}", funct7);
						self.dump_instruction(self.pc.wrapping_sub(4));
						panic!();
					}
				},
				6 => Instruction::OR,
				7 => Instruction::AND,
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
					0x20 => Instruction::SUBW,
					_ => {
						println!("Unknown funct7: {:07b}", funct7);
						self.dump_instruction(self.pc.wrapping_sub(4));
						panic!();
					}
				},
				1 => Instruction::SLLW,
				5 => match funct7 {
					0 => Instruction::SRLW,
					0x20 => Instruction::SRAW,
					_ => {
						println!("Unknown funct7: {:07b}", funct7);
						self.dump_instruction(self.pc.wrapping_sub(4));
						panic!();
					}
				},
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
					match word {
						0x00000073 => Instruction::ECALL,
						0x30200073 => Instruction::MRET,
						_ => {
							println!("Priviledged instruction 0x{:08x} is not supported yet", word);
							self.dump_instruction(self.pc.wrapping_sub(4));
							panic!();
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

	fn operate(&mut self, word: u32, instruction: Instruction) {
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
				let csr = (word >> 20) & 0xfff; // [31:20];
				let rs = (word >> 15) & 0x1f; // [19:15];
				let rd = (word >> 7) & 0x1f; // [11:7];
				// @TODO: Don't write if csr bits aren't writable
				match instruction {
					Instruction::CSRRS => {
						self.x[rd as usize] = self.sign_extend(self.csr[csr as usize] as i64);
						self.x[0] = 0; // hard-wired zero
						self.csr[csr as usize] = self.unsigned_data(self.x[rd as usize] | self.x[rs as usize]);
					},
					Instruction::CSRRW => {
						self.x[rd as usize] = self.sign_extend(self.csr[csr as usize] as i64);
						self.x[0] = 0; // hard-wired zero
						self.csr[csr as usize] = self.unsigned_data(self.x[rs as usize]);
					},
					Instruction::CSRRWI => {
						self.x[rd as usize] = self.sign_extend(self.csr[csr as usize] as i64);
						self.x[0] = 0; // hard-wired zero
						self.csr[csr as usize] = rs as u64;
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
						self.x[rd as usize] = self.sign_extend(self.pc as i64);
						self.pc = (self.x[rs1 as usize] as u64).wrapping_add(imm as u64);
					},
					Instruction::LB => {
						self.x[rd as usize] = self.load_byte(self.x[rs1 as usize].wrapping_add(imm) as u64) as i8 as i64;
					},
					Instruction::LBU => {
						self.x[rd as usize] = self.load_byte(self.x[rs1 as usize].wrapping_add(imm) as u64) as i64;
					},
					Instruction::LD => {
						self.x[rd as usize] = self.load_doubleword(self.x[rs1 as usize].wrapping_add(imm) as u64) as i64;
					},
					Instruction::LH => {
						self.x[rd as usize] = self.load_halfword(self.x[rs1 as usize].wrapping_add(imm) as u64) as i16 as i64;
					},
					Instruction::LHU => {
						self.x[rd as usize] = self.load_halfword(self.x[rs1 as usize].wrapping_add(imm) as u64) as i64;
					},
					Instruction::LW => {
						self.x[rd as usize] = self.load_word(self.x[rs1 as usize].wrapping_add(imm) as u64) as i32 as i64;
					},
					Instruction::LWU => {
						self.x[rd as usize] = self.load_word(self.x[rs1 as usize].wrapping_add(imm) as u64) as i64;
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
					Instruction::AND => {
						self.x[rd as usize] = self.sign_extend(self.x[rs1 as usize] & self.x[rs2 as usize]);
					},
					Instruction::ECALL => {
						// @TODO: Implement
					},
					Instruction::MRET => {
						// @TODO: Implement
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
					Instruction::OR => {
						self.x[rd as usize] = self.sign_extend(self.x[rs1 as usize] | self.x[rs2 as usize]);
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
						self.store_byte(self.x[rs1 as usize].wrapping_add(imm) as u64, self.x[rs2 as usize] as u8);
					},
					Instruction::SH => {
						self.store_halfword(self.x[rs1 as usize].wrapping_add(imm) as u64, self.x[rs2 as usize] as u16);
					},
					Instruction::SW => {
						self.store_word(self.x[rs1 as usize].wrapping_add(imm) as u64, self.x[rs2 as usize] as u32);
					},
					Instruction::SD => {
						self.store_doubleword(self.x[rs1 as usize].wrapping_add(imm) as u64, self.x[rs2 as usize] as u64);
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
					word & 0xfffff000 // imm[31:12] = [31:12]
				) as u64;
				match instruction {
					Instruction::AUIPC => {
						self.x[rd as usize] = self.pc.wrapping_sub(4).wrapping_add(imm) as i32 as i64;
					},
					Instruction::LUI => {
						self.x[rd as usize] = imm as i32 as i64;
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
	}

	fn dump_instruction(&self, address: u64) {
		let word = self.load_word(address);
		let pc = self.unsigned_data(address as i64);
		let opcode = word & 0x7f; // [6:0]
		println!("Pc: {:08x}, Opcode: {:07b}, Word: {:08x}", pc, opcode, word);
	}
}