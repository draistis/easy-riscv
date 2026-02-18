use crate::{
    error::{AssemblerError, SourceLocation},
    tokenizer::tokenize,
};

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

pub fn assemble(source: &str) -> anyhow::Result<Vec<u8>> {
    let tokens = tokenize(source)?;

    let mut symbol_table = SymbolTable::new();
    let mut parser = Parser::new();
    let parsed_items = parser.parse_all(&mut symbol_table)?;

    let unresolved = symbol_table.check_for_unresolved();
    if !unresolved.is_empty() {
        let mut errors = Vec::new();
        for (name, line) in unresolved {
            errors.push(AssemblerError::SymbolError {
                message: format!("Undefined symbol: {}", name),
                location: SourceLocation { line, col: 0 },
            })
        }
        if errors.len() == 1 {
            return anyhow::Err(errors.remove(0));
        } else {
            return anyhow::Err(AssemblerError::MultipleErrors(errors));
        }
    }

    let mut memory_map = MemoryMap::new();
    allocate_memory(&mut memory_map, &parsed_items)?;

    let output = generate_machine_code(&memory_map, &symbol_table, &parsed_items)?;
    Ok(output)
}
