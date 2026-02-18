use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[error("line {line}, column {col}")]
pub struct SourceLocation {
    pub line: u64,
    pub col: u64,
}

#[derive(Error, Debug, Clone)]
pub enum AssemblerError {
    #[error("Tokenizer error: {message} at {location}")]
    TokenizerError {
        message: String,
        location: SourceLocation,
    },
    #[error("Parser error: {message} at {location}")]
    ParserError {
        message: String,
        location: SourceLocation,
    },
    // FIX:
    #[error("Multiple errors")]
    MultipleErrors(Vec<AssemblerError>),
}
