/// Supported instructions:
/// LUI rd, imm #imm 0x00000 to 0xFFFFF
/// ADDI rd, rs1, imm #imm -0x800 to +0x7FF
/// ADD rd, rs1, rs2
/// SUB rd, rs1, rs2
/// Supported pseudoinstructions:
/// INC rd -> ADDI rd, rd, 1
/// DEC rd -> ADDI rd, rd, -1
/// MV rd, rs1 -> ADDI rd, rs1, 0
/// NOP -> ADDI x0, x0, 0
/// NEG rd -> SUB rd, x0, rd
/// LI rd, imm -> DEPENDS ON imm SIZE (1-3 instructions)
pub struct Assembler {
    bytes: Vec<u8>,
}

impl Assembler {}
