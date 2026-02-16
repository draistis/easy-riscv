pub struct Cpu {
    /// Program counter
    pub pc: u32,
    /// Registers
    pub regs: [u32; 32],
    /// Program code
    pub dram: Vec<u8>,
}

impl Cpu {
    pub fn new_with_instructions(instructions: Vec<u8>) -> Self {
        Self {
            pc: 0,
            regs: [0; 32],
            dram: instructions,
        }
    }

    pub fn step(&mut self) {
        // Fetch instruction
        let instruction = self.fetch();

        // Increment program counter (4 bytes, 32 bits per instruction)
        self.pc += 4;
        // Reset the "0" register
        self.regs[0] = 0;

        // Decode instruction
        // &
        // Execute the instruction
        self.execute(instruction);
    }

    fn fetch(&self) -> u32 {
        let index = self.pc as usize;

        // Using little-endian
        self.dram[index] as u32
            | (self.dram[index + 1] as u32) << 8
            | (self.dram[index + 2] as u32) << 16
            | (self.dram[index + 3] as u32) << 24
    }

    fn execute(&mut self, instruction: u32) {
        let immediate = instruction;
        let opcode = instruction & 0x7f; // 7 bits
        let rd = ((instruction >> 7) & 0x1f) as usize; // 5 bits
        #[allow(unused_variables)]
        let funct3 = ((instruction >> 12) & 0x7) as usize; // 3 bits
        let rs1 = ((instruction >> 15) & 0x1f) as usize; // 5 bits
        let rs2 = ((instruction >> 20) & 0x1f) as usize; // 5 bits
        #[allow(unused_variables)]
        let funct7 = ((instruction >> 25) & 0x7f) as usize; // 7 bits

        match opcode {
            // IMMEDIATE
            0b0110111 => {
                // LUI
                self.regs[rd] = immediate & 0xFFFFF000;
            }
            0b0010011 => {
                // ADDI
                self.regs[rd] = self.regs[rs1].wrapping_add((immediate as i32 >> 20) as u32);
            }
            // REGULAR
            0b0110011 => {
                match funct3 {
                    0x0 => {
                        if funct7 == 0x20 {
                            // SUB
                            self.regs[rd] = self.regs[rs1].wrapping_sub(self.regs[rs2]);
                        } else if funct7 == 0x0 {
                            // ADD
                            self.regs[rd] = self.regs[rs1].wrapping_add(self.regs[rs2]);
                        }
                        dbg!("Undefined");
                    }
                    0x4 => {
                        // XOR
                        self.regs[rd] = self.regs[rs1] ^ self.regs[rs2];
                    }
                    0x6 => {
                        // OR
                        self.regs[rd] = self.regs[rs1] | self.regs[rs2];
                    }
                    0x7 => {
                        // AND
                        self.regs[rd] = self.regs[rs1] & self.regs[rs2];
                    }
                    _ => {
                        dbg!("Not implemented");
                    }
                }
            }
            _ => {
                dbg!("Not implemented");
            }
        }
    }
}
