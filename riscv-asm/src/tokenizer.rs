use anyhow::anyhow;

use crate::error::{AssemblerError, SourceLocation};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    kind: TokenKind,
    text: Option<String>,
    location: SourceLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Base {
    Dec,
    Hex,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Instruction,       // "add", "sub", "lui", etc.
    Pseudoinstruction, // "mv", "dec", etc.
    Directive,         // ".word", ".text", ".global"
    Identifier,        // labels
    Register,          // "x0", "zero", "sp", etc.
    Comment,           // Text after "#"
    Number(Base),
    Comma,
    Colon,
    LParen,
    RParen,
    Newline,
    EndOfFile,
    String,
}

pub fn tokenize(source: &str) -> anyhow::Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut line_num = 1;
    let mut lines = source.lines();

    while let Some(line) = lines.next() {
        let mut col_num = 1;
        let mut chars = line.chars().peekable();

        while let Some(char) = chars.next() {
            let location = SourceLocation {
                line: line_num,
                col: col_num,
            };

            match char {
                // Whitespace
                ' ' | '\t' | '\r' => {
                    col_num += 1;
                }
                // Comment (ignored)
                '#' => {
                    // tokens.push(Token {
                    //     kind: TokenKind::Comment,
                    //     text: Some(chars.filter(|c| !c.is_whitespace()).collect()),
                    //     location,
                    // });
                    break;
                }
                // Punctuation
                ',' => {
                    tokens.push(Token {
                        kind: TokenKind::Comma,
                        text: Some(char.to_string()),
                        location,
                    });
                    col_num += 1;
                }
                ':' => {
                    tokens.push(Token {
                        kind: TokenKind::Colon,
                        text: Some(char.to_string()),
                        location,
                    });
                    col_num += 1;
                }
                '(' => {
                    tokens.push(Token {
                        kind: TokenKind::LParen,
                        text: Some(char.to_string()),
                        location,
                    });
                    col_num += 1;
                }
                ')' => {
                    tokens.push(Token {
                        kind: TokenKind::RParen,
                        text: Some(char.to_string()),
                        location,
                    });
                    col_num += 1;
                }
                // Numbers
                '-' | '0'..='9' => {
                    let mut base = Base::Dec;
                    let mut text = String::new();
                    text.push(char);
                    col_num += 1;

                    if char == '-' {
                        if !chars.peek().is_some_and(|c| c.is_ascii_digit()) {
                            anyhow!("expected digit after '-' on {}", location);
                        }
                        while let Some(c) = chars.peek() {
                            if c.is_ascii_digit() {
                                text.push(chars.next().unwrap()); // SAFETY: we know that next
                                // character exists after peeking
                                col_num += 1;
                            } else {
                                break;
                            }
                        }
                    } else if char == '0' && chars.peek() == Some(&'x') {
                        base = Base::Hex;
                        text.push(chars.next().unwrap()); // SAFETY: already checked that the next char exists and is 'x'
                        col_num += 1;
                        while let Some(c) = chars.peek() {
                            if c.is_ascii_hexdigit() {
                                text.push(chars.next().unwrap()); // SAFETY: we know that next character exists after peeking
                                col_num += 1;
                            } else {
                                break;
                            }
                        }
                    } else {
                        while let Some(c) = chars.peek() {
                            if c.is_ascii_digit() {
                                text.push(chars.next().unwrap()); // SAFETY: we know that next character exists after peeking
                                col_num += 1;
                            } else {
                                break;
                            }
                        }
                    }
                    // Can update col_num based on text length instead

                    tokens.push(Token {
                        kind: TokenKind::Number(base),
                        text: Some(text),
                        location,
                    })
                }
                // Directive (".text", ".DATA", etc.)
                '.' => {
                    let mut text = String::new();
                    text.push(char);
                    col_num += 1;

                    while let Some(c) = chars.peek() {
                        if c.is_ascii_alphabetic() {
                            text.push(chars.next().unwrap()); // SAFETY: we know that next character exists after peeking
                            col_num += 1;
                        } else {
                            break;
                        }
                    }

                    tokens.push(Token {
                        kind: TokenKind::Directive,
                        text: Some(line.to_string()),
                        location,
                    })
                }
                // Identifiers (instruction, register, label, etc.)
                'a'..='z' | 'A'..='Z' | '_' => {
                    let mut text = String::new();
                    text.push(char);
                    col_num += 1;

                    while let Some(c) = chars.peek() {
                        if c.is_ascii_alphabetic() || c == &'_' {
                            text.push(chars.next().unwrap()); // SAFETY: we know that next character exists after peeking
                            col_num += 1;
                        } else {
                            break;
                        }
                    }

                    let kind = classify_identifier(&text);
                    tokens.push(Token {
                        kind,
                        text: Some(text.to_string()),
                        location,
                    })
                }
                // String literals
                '"' => {
                    let mut text = String::new();
                    text.push(char);
                    col_num += 1;

                    let mut escaped = false;
                    while let Some(c) = chars.next() {
                        text.push(c);
                        col_num += 1;

                        if c == '"' && !escaped {
                            break;
                        }
                        if c == '\\' {
                            escaped = true;
                            continue;
                        } else {
                            escaped = false
                        }
                    }

                    if !text.ends_with('"') {
                        anyhow!("unterminated string literal on {}", location);
                    }

                    tokens.push(Token {
                        kind: TokenKind::String,
                        text: Some(text.to_string()),
                        location,
                    })
                }
                _ => {
                    anyhow!("unexpected character '{}' on {}", char, location);
                }
            }
        }

        tokens.push(Token {
            kind: TokenKind::Newline,
            text: None,
            location: SourceLocation {
                line: line_num,
                col: col_num,
            },
        });
        line_num += 1;
    }

    tokens.push(Token {
        kind: TokenKind::EndOfFile,
        text: None,
        location: SourceLocation {
            line: line_num,
            col: 1,
        },
    });

    anyhow::Ok(tokens)
}

fn classify_identifier(s: &str) -> TokenKind {
    match s {
        // Registers
        "zero" | "ra" | "sp" | "gp" | "tp" | "fp" | "s0" | "s1" | "s2" | "s3" | "s4" | "s5"
        | "s6" | "s7" | "s8" | "s9" | "s10" | "s11" | "a0" | "a1" | "a2" | "a3" | "a4" | "a5"
        | "a6" | "a7" | "t0" | "t1" | "t2" | "t3" | "t4" | "t5" | "t6" | "x0" | "x1" | "x2"
        | "x3" | "x4" | "x5" | "x6" | "x7" | "x8" | "x9" | "x10" | "x11" | "x12" | "x13"
        | "x14" | "x15" | "x16" | "x17" | "x18" | "x19" | "x20" | "x21" | "x22" | "x23" | "x24"
        | "x25" | "x26" | "x27" | "x28" | "x29" | "x30" | "x31" => TokenKind::Register,
        // Instructions (RV32I)
        "add" | "sub" | "sll" | "slt" | "sltu" | "xor" | "srl" | "sra" | "or" | "and" | // R-type
        "addi" | "slti" | "sltiu" | "xori" | "ori" | "andi" | "slli" | "srli" | "srai" | // I-type (ALU)
        "lb" | "lh" | "lw" | "lbu" | "lhu" | // I-type (Load)
        "jalr" | // I-type (Jump)
        "sb" | "sh" | "sw" | // S-type
        "beq" | "bne" | "blt" | "bge" | "bltu" | "bgeu" | // B-type
        "lui" | "auipc" | // U-type
        "jal" | // J-type
        "ecall" => TokenKind::Instruction,
        // Pseudoinstructions
        "inc" | "dec" | "mv" | "nop" | "neg" | "li" => TokenKind::Pseudoinstruction,
        // Default to identifier (likely a label)
        _ => TokenKind::Identifier,}
}

//  Unit Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_instruction() {
        let code = "loop: addi x1, x0, 5 # comment\n";
        let tokens = tokenize(code).unwrap();
        assert_eq!(tokens.len(), 10); // loop, :, Identifier(addi), Register(x1), Comma, Register(x0), Comma, Integer(5), Newline, EOF
        assert_eq!(tokens[0].kind, TokenKind::Identifier); // "loop" -> classified as Identifier initially
        assert_eq!(tokens[0].text, Some("loop".to_string()));
        assert_eq!(tokens[1].kind, TokenKind::Colon);
        assert_eq!(tokens[2].kind, TokenKind::Instruction);
        assert_eq!(tokens[2].text, Some("addi".to_string()));
        assert_eq!(tokens[3].kind, TokenKind::Register);
        assert_eq!(tokens[3].text, Some("x1".to_string()));
        assert_eq!(tokens[4].kind, TokenKind::Comma);
        assert_eq!(tokens[5].kind, TokenKind::Register);
        assert_eq!(tokens[5].text, Some("x0".to_string()));
        assert_eq!(tokens[6].kind, TokenKind::Comma);
        assert_eq!(tokens[7].kind, TokenKind::Number(Base::Dec));
        assert_eq!(tokens[7].text, Some("5".to_string()));
        assert_eq!(tokens[8].kind, TokenKind::Newline);
        assert_eq!(tokens[9].kind, TokenKind::EndOfFile); // Added EOF token
    }

    #[test]
    fn test_hex_number() {
        let code = "li a0, 0xFF"; // li might be pseudo-instruction, handle in parser
        let tokens = tokenize(code).unwrap();
        assert!(
            tokens
                .iter()
                .any(|t| t.kind == TokenKind::Number(Base::Hex)
                    && t.text == Some("0xFF".to_string()))
        );
    }

    #[test]
    fn test_simple_instruction_1() {
        let code = "lb a0, 8(sp)";
        let tokens = tokenize(code).unwrap();

        // Check the instruction token
        assert_eq!(tokens[0].kind, TokenKind::Instruction);
        assert_eq!(tokens[0].text, Some("lb".to_string()));

        // Check the register token
        assert_eq!(tokens[1].kind, TokenKind::Register);
        assert_eq!(tokens[1].text, Some("a0".to_string()));

        // Check the comma token
        assert_eq!(tokens[2].kind, TokenKind::Comma);
        assert_eq!(tokens[2].text, Some(",".to_string()));

        // Check the immediate value token
        assert_eq!(tokens[3].kind, TokenKind::Number(Base::Dec));
        assert_eq!(tokens[3].text, Some("8".to_string()));

        // Check the left parenthesis token
        assert_eq!(tokens[4].kind, TokenKind::LParen);
        assert_eq!(tokens[4].text, Some("(".to_string()));

        // Check the base register token
        assert_eq!(tokens[5].kind, TokenKind::Register);
        assert_eq!(tokens[5].text, Some("sp".to_string()));

        // Check the right parenthesis token
        assert_eq!(tokens[6].kind, TokenKind::RParen);
        assert_eq!(tokens[6].text, Some(")".to_string()));

        // Check the right parenthesis token
        assert_eq!(tokens[7].kind, TokenKind::EndOfFile);
        assert_eq!(tokens[7].text, Some("".to_string()));

        // Verify the total number of tokens
        assert_eq!(tokens.len(), 8);
    }

    #[test]
    fn test_directive() {
        let code = ".data\nmy_var: .word 123";
        let tokens = tokenize(code).unwrap();
        assert!(
            tokens
                .iter()
                .any(|t| t.kind == TokenKind::Directive && t.text == Some(".data".to_string()))
        );
        assert!(
            tokens
                .iter()
                .any(|t| t.kind == TokenKind::Directive && t.text == Some(".word".to_string()))
        );
    }

    #[test]
    fn test_comment_stripping() {
        let code = " add x1, x2, x3 # This should be ignored";
        let tokens = tokenize(code).unwrap();
        assert!(!tokens.iter().any(|t| t.kind == TokenKind::Comment));
        assert!(
            tokens
                .iter()
                .any(|t| t.kind == TokenKind::Instruction && t.text == Some("add".to_string()))
        ); // Check instruction is present
        assert_eq!(tokens.last().unwrap().kind, TokenKind::EndOfFile); // Should end with EOF, not newline if no newline after comment
    }

    #[test]
    fn test_location() {
        let code = "line1\n line2: lw x1, 0(sp)";
        let tokens = tokenize(code).unwrap();
        let lw_token = tokens
            .iter()
            .find(|t| t.text == Some("lw".to_string()))
            .unwrap();
        assert_eq!(lw_token.location.line, 2);
        assert!(lw_token.location.col > 1); // Should not be column 1
        let sp_token = tokens
            .iter()
            .find(|t| t.text == Some("sp".to_string()))
            .unwrap();
        assert_eq!(sp_token.location.line, 2);
    }
}
