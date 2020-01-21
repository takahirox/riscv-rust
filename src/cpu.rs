const MEMORY_CAPACITY: usize = 1024 * 16; // @TODO: temporal
const CSR_CAPACITY: usize = 4096;

pub struct Cpu {
	x: [i32; 32],
	pc: u32,
	csr: [u32; CSR_CAPACITY],
	memory: [u8; MEMORY_CAPACITY]
}

enum Instruction {
	ADD,
	ADDI,
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
	LH,
	LHU,
	LUI,
	LW,
	MRET,
	OR,
	ORI,
	SB,
	SH,
	SLL,
	SLLI,
	SLT,
	SLTI,
	SLTU,
	SLTIU,
	SRA,
	SRAI,
	SRL,
	SRLI,
	SUB,
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
		Instruction::LH => "LH",
		Instruction::LHU => "LHU",
		Instruction::LUI => "LUI",
		Instruction::LW => "LW",
		Instruction::MRET => "MRET",
		Instruction::OR => "OR",
		Instruction::ORI => "ORI",
		Instruction::SB => "SB",
		Instruction::SH => "SH",
		Instruction::SLL => "SLL",
		Instruction::SLLI => "SLLI",
		Instruction::SLT => "SLT",
		Instruction::SLTI => "SLTI",
		Instruction::SLTU => "SLTU",
		Instruction::SLTIU => "SLTIU",
		Instruction::SRA => "SRA",
		Instruction::SRAI => "SRAI",
		Instruction::SRL => "SRL",
		Instruction::SRLI => "SRLI",
		Instruction::SUB => "SUB",
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
		Instruction::ANDI |
		Instruction::JALR |
		Instruction::LB |
		Instruction::LBU |
		Instruction::LH |
		Instruction::LHU |
		Instruction::LW |
		Instruction::ORI |
		Instruction::SLLI |
		Instruction::SLTI |
		Instruction::SLTIU |
		Instruction::SRLI |
		Instruction::SRAI |
		Instruction::XORI => InstructionFormat::I,
		Instruction::JAL => InstructionFormat::J,
		Instruction::FENCE => InstructionFormat::O,
		Instruction::ADD |
		Instruction::AND |
		Instruction::ECALL |
		Instruction::MRET |
		Instruction::OR |
		Instruction::SUB |
		Instruction::SLL |
		Instruction::SLT |
		Instruction::SLTU |
		Instruction::SRA |
		Instruction::SRL |
		Instruction::XOR => InstructionFormat::R,
		Instruction::SB |
		Instruction::SH |
		Instruction::SW => InstructionFormat::S,
		Instruction::AUIPC |
		Instruction::LUI => InstructionFormat::U
	}
}

impl Cpu {
	pub fn new() -> Self {
		Cpu {
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
		while true {
			// @TODO: Temporal termination check, ends at ECALL instruction.
			// I should fix.
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

	fn load_word(&self, address: u32) -> u32 {
		((self.memory[address as usize + 3] as u32) << 24) |
		((self.memory[address as usize + 2] as u32) << 16) |
		((self.memory[address as usize + 1] as u32) << 8) |
		(self.memory[address as usize] as u32)
	}

	fn load_halfword(&self, address: u32) -> u16 {
		((self.memory[address as usize + 1] as u16) << 8) |
		(self.memory[address as usize] as u16)
	}

	fn load_byte(&self, address: u32) -> u8 {
		self.memory[address as usize]
	}

	fn store_word(&mut self, address: u32, value: u32) {
		self.memory[address as usize] = (value & 0xff) as u8;
		self.memory[address as usize + 1] = ((value >> 8) & 0xff) as u8;
		self.memory[address as usize + 2] = ((value >> 16) & 0xff) as u8;
		self.memory[address as usize + 3] = ((value >> 24) & 0xff) as u8;
	}

	fn store_halfword(&mut self, address: u32, value: u16) {
		self.memory[address as usize] = (value & 0xff) as u8;
		self.memory[address as usize + 1] = ((value >> 8) & 0xff) as u8;
	}

	fn store_byte(&mut self, address: u32, value: u8) {
		self.memory[address as usize] = value;
	}

	fn decode(&self, word: u32) -> Instruction {
		let opcode = word & 0x7f; // [6:0]
		let funct3 = (word >> 12) & 0x7; // [14:12]
		let funct7 = (word >> 25) & 0x7f; // [31:25]

		// B-Type

		if opcode == 0x63 {
			return match funct3 {
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
			};
		}

		// R-Type and C-Type (CSR)

		if opcode == 0x33 {
			return match funct3 {
				0 => match funct7 {
					0 => Instruction::ADD,
					0x20 => Instruction::SUB,
					_ => {
						println!("Unknown funct7: {:07b}", funct7);
						self.dump_instruction(self.pc.wrapping_sub(4));
						panic!();
					}
				},
				1 => Instruction::SLL,
				2 => Instruction::SLT,
				3 => Instruction::SLTU,
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
			};
		}

		if opcode == 0x73 {
			return match funct3 {
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
			};
		}

		// I-Type

		if opcode == 0x03 {
			return match funct3 {
				0 => Instruction::LB,
				1 => Instruction::LH,
				2 => Instruction::LW,
				4 => Instruction::LBU,
				5 => Instruction::LHU,
				_ => {
					println!("Unknown funct3: {:03b}", funct3);
					self.dump_instruction(self.pc.wrapping_sub(4));
					panic!();
				}
			}
		}

		if opcode == 0x13 {
			return match funct3 {
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
			};
		}

		if opcode == 0x67 {
			return Instruction::JALR;
		}

		// J-Type

		if opcode == 0x6f {
			return Instruction::JAL;
		}

		// S-Type

		if opcode == 0x23 {
			return match funct3 {
				0 => Instruction::SB,
				1 => Instruction::SH,
				2 => Instruction::SW,
				_ => {
					println!("Unknown funct3: {:03b}", funct3);
					self.dump_instruction(self.pc.wrapping_sub(4));
					panic!();
				}
			};
		}

		// U-Type

		if opcode == 0x17 {
			return Instruction::AUIPC;
		}

		if opcode == 0x37 {
			return Instruction::LUI;
		}

		// Others

		if opcode == 0x0f {
			return Instruction::FENCE;
		}

		println!("Unknown Instruction type.");
		self.dump_instruction(self.pc.wrapping_sub(4));
		panic!();
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
				) as u32;
				match instruction {
					Instruction::BEQ => {
						if self.x[rs1 as usize] == self.x[rs2 as usize] {
							self.pc = self.pc.wrapping_sub(4).wrapping_add(imm);
						}
					},
					Instruction::BGE => {
						if self.x[rs1 as usize] >= self.x[rs2 as usize] {
							self.pc = self.pc.wrapping_sub(4).wrapping_add(imm);
						}
					},
					Instruction::BGEU => {
						if (self.x[rs1 as usize] as u32) >= (self.x[rs2 as usize] as u32) {
							self.pc = self.pc.wrapping_sub(4).wrapping_add(imm);
						}
					},
					Instruction::BLT => {
						if self.x[rs1 as usize] < self.x[rs2 as usize] {
							self.pc = self.pc.wrapping_sub(4).wrapping_add(imm);
						}
					},
					Instruction::BLTU => {
						if (self.x[rs1 as usize] as u32) < (self.x[rs2 as usize] as u32) {
							self.pc = self.pc.wrapping_sub(4).wrapping_add(imm);
						}
					},
					Instruction::BNE => {
						if self.x[rs1 as usize] != self.x[rs2 as usize] {
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
						self.x[rd as usize] = self.csr[csr as usize] as i32;
						self.x[0] = 0; // hard-wired zero
						self.csr[csr as usize] = (self.x[rd as usize] | self.x[rs as usize]) as u32;
					},
					Instruction::CSRRW => {
						self.x[rd as usize] = self.csr[csr as usize] as i32;
						self.x[0] = 0; // hard-wired zero
						self.csr[csr as usize] = self.x[rs as usize] as u32;
					},
					Instruction::CSRRWI => {
						self.x[rd as usize] = self.csr[csr as usize] as i32;
						self.x[0] = 0; // hard-wired zero
						self.csr[csr as usize] = rs as u32;
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
				) as i32;
				match instruction {
					Instruction::ADDI => {
						self.x[rd as usize] = self.x[rs1 as usize].wrapping_add(imm);
					},
					Instruction::ANDI => {
						self.x[rd as usize] = self.x[rs1 as usize] & imm;
					},
					Instruction::JALR => {
						self.x[rd as usize] = self.pc as i32;
						self.pc = (self.x[rs1 as usize] as u32).wrapping_add(imm as u32);
					},
					Instruction::LB => {
						self.x[rd as usize] = self.load_byte(self.x[rs1 as usize].wrapping_add(imm) as u32) as i8 as i32;
					},
					Instruction::LBU => {
						self.x[rd as usize] = self.load_byte(self.x[rs1 as usize].wrapping_add(imm) as u32) as i32;
					},
					Instruction::LH => {
						self.x[rd as usize] = self.load_halfword(self.x[rs1 as usize].wrapping_add(imm) as u32) as i16 as i32;
					},
					Instruction::LHU => {
						self.x[rd as usize] = self.load_halfword(self.x[rs1 as usize].wrapping_add(imm) as u32) as i32;
					},
					Instruction::LW => {
						self.x[rd as usize] = self.load_word(self.x[rs1 as usize].wrapping_add(imm) as u32) as i32;
					},
					Instruction::ORI => {
						self.x[rd as usize] = self.x[rs1 as usize] | imm;
					},
					Instruction::SLLI => {
						let shamt = (imm & 0x1f) as u32;
						self.x[rd as usize] = self.x[rs1 as usize] << shamt;
					},
					Instruction::SLTI => {
						self.x[rd as usize] = match self.x[rs1 as usize] < imm {
							true => 1,
							false => 0
						}
					},
					Instruction::SLTIU => {
						self.x[rd as usize] = match (self.x[rs1 as usize] as u32) < (imm as u32) {
							true => 1,
							false => 0
						}
					},
					Instruction::SRAI => {
						let shamt = (imm & 0x1f) as u32;
						self.x[rd as usize] = self.x[rs1 as usize] >> shamt;
					},
					Instruction::SRLI => {
						let shamt = (imm & 0x1f) as u32;
						self.x[rd as usize] = ((self.x[rs1 as usize] as u32) >> shamt) as i32;
					},
					Instruction::XORI => {
						self.x[rd as usize] = self.x[rs1 as usize] ^ imm;
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
				) as u32;
				match instruction {
					Instruction::JAL => {
						self.x[rd as usize] = self.pc as i32;
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
						self.x[rd as usize] = self.x[rs1 as usize].wrapping_add(self.x[rs2 as usize]);
					},
					Instruction::AND => {
						self.x[rd as usize] = self.x[rs1 as usize] & self.x[rs2 as usize];
					},
					Instruction::ECALL => {
						// @TODO: Implement
					},
					Instruction::MRET => {
						// @TODO: Implement
					},
					Instruction::OR => {
						self.x[rd as usize] = self.x[rs1 as usize] | self.x[rs2 as usize];
					},
					Instruction::SUB => {
						self.x[rd as usize] = self.x[rs1 as usize].wrapping_sub(self.x[rs2 as usize]);
					},
					Instruction::SLL => {
						self.x[rd as usize] = self.x[rs1 as usize].wrapping_shl(self.x[rs2 as usize] as u32);
					},
					Instruction::SLT => {
						self.x[rd as usize] = match self.x[rs1 as usize] < self.x[rs2 as usize] {
							true => 1,
							false => 0
						}
					},
					Instruction::SLTU => {
						self.x[rd as usize] = match (self.x[rs1 as usize] as u32) < (self.x[rs2 as usize] as u32) {
							true => 1,
							false => 0
						}
					},
					Instruction::SRA => {
						self.x[rd as usize] = self.x[rs1 as usize].wrapping_shr(self.x[rs2 as usize] as u32);
					},
					Instruction::SRL => {
						self.x[rd as usize] = (self.x[rs1 as usize] as u32).wrapping_shr(self.x[rs2 as usize] as u32) as i32;
					},
					Instruction::XOR => {
						self.x[rd as usize] = self.x[rs1 as usize] ^ self.x[rs2 as usize];
					},
					_ => {
						println!("{}", get_instruction_name(&instruction).to_owned() + " instruction is not supported yet.");
						self.dump_instruction(self.pc.wrapping_sub(4));
						panic!();
					}
				};
			},
			InstructionFormat::S => {
				let rd = (word >> 7) & 0x1f; // [11:7]
				let rs1 = (word >> 15) & 0x1f; // [19:15]
				let rs2 = (word >> 20) & 0x1f; // [24:20]
				let imm = (
					match word & 0x80000000 {
						0x80000000 => 0xfffff000,
						_ => 0
					} | // imm[31:12] = [31]
					((word & 0xfe000000) >> 20) | // imm[11:5] = [31:25],
					((word & 0x00000f80) >> 7) // imm[4:0] = [11:7]
				) as i32;
				match instruction {
					Instruction::SB => {
						self.store_byte(self.x[rs1 as usize].wrapping_add(imm) as u32, self.x[rs2 as usize] as u8);
					},
					Instruction::SH => {
						self.store_halfword(self.x[rs1 as usize].wrapping_add(imm) as u32, self.x[rs2 as usize] as u16);
					},
					Instruction::SW => {
						self.store_word(self.x[rs1 as usize].wrapping_add(imm) as u32, self.x[rs2 as usize] as u32);
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
				let imm = word & 0xfffff000; // imm[31:12] = [31:12]
				match instruction {
					Instruction::AUIPC => {
						self.x[rd as usize] = self.pc.wrapping_sub(4).wrapping_add(imm) as i32;
					},
					Instruction::LUI => {
						self.x[rd as usize] = imm as i32;
					}
					_ => {
						println!("{}", get_instruction_name(&instruction).to_owned() + " instruction is not supported yet.");
						self.dump_instruction(self.pc.wrapping_sub(4));
						panic!();
					}
				};
			},
			_ => {
				println!("{}", get_instruction_name(&instruction).to_owned() + " instruction is not supported yet.");
				self.dump_instruction(self.pc.wrapping_sub(4));
				panic!();
			}
		}
		self.x[0] = 0; // hard-wired zero
	}

	fn dump_instruction(&self, pc: u32) {
		let word = self.load_word(pc);
		let opcode = word & 0x7f; // [6:0]
		println!("Pc: {:08x}, Opcode: {:07b}, Word: {:08x}", pc, opcode, word);
	}
}