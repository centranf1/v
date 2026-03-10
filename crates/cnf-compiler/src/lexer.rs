//! Lexer — Tokenize CENTRA-NF source.
//!
//! Responsibility: Convert source string into Token stream.
//! Fail fast on unrecognized characters.

/// Structured error with position and context
#[derive(Debug, Clone, PartialEq)]

pub struct LexError {
    pub message: String,
    pub line: usize,
    pub col: usize,
    pub context: String,
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Lexer error at {}:{}: {}\n  | {}",
            self.line, self.col, self.message, self.context
        )
    }
}

impl std::error::Error for LexError {}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // CSM protocol (v154)
    MapCsm,
    CompressCsm,
    DecompressCsm,
    DictionaryRef,
    ProtocolVersion,
    Density,
    // Divisions
    IdentificationDiv,
    EnvironmentDiv,
    DataDiv,
    ProcedureDiv,
    VerificationDiv,
    // Verification keywords
    PreCondition,
    PostCondition,
    Invariant,
    Prove,
    AssertKw,
    Satisfies,
    AuditLog,
    ComplianceReport,
    // Keywords
    Division,
    ProgramId,
    Author,
    Version,
    Os,
    Arch,
    RuntimeVersion,
    Input,
    Output,
    Compress,
    VerifyIntegrity,
    Transcode,
    Filter,
    Aggregate,
    Convert,
    Merge,
    Split,
    Validate,
    Extract,
    Encrypt,
    Decrypt,
    Display,
    Print,
    Read,
    Open,
    ReadFile,
    WriteFile,
    Close,
    Checkpoint,
    Replay,
    As,
    // Arithmetic operations
    Set,
    Add,
    Subtract,
    Multiply,
    Divide,
    Max,
    Min,
    Abs,
    // String operations
    Concatenate,
    Substring,
    Length,
    Uppercase,
    Lowercase,
    Trim,
    // Control flow
    If,
    Else,
    Then,
    EndIf,
    For,
    While,
    Do,
    EndFor,
    EndWhile,
    In,
    // Functions
    Define,
    Function,
    EndFunction,
    Parameters,
    Returns,
    // Network operations
    Network,
    Node,
    At,
    Self_,
    Topology,
    Pipeline,
    Mesh,
    Star,
    Timeout,
    Send,
    Receive,
    To,
    From,
    Pipe,
    CallRemote,
    // Data types
    VideoMp4,
    ImageJpg,
    FinancialDecimal,
    AudioWav,
    CsvTable,
    BinaryBlob,
    JsonObject,
    XmlDocument,
    ParquetTable,
    TextString,
    NumberInteger,
    NumberDecimal,
    FileHandle,
    RecordStream,
    // Quantum operations (v0.8.0)
    QuantumEncrypt,
    QuantumDecrypt,
    QuantumSign,
    QuantumVerifySig,
    QuantumSignEncrypt,
    QuantumVerifyDecrypt,
    GenerateKeypair,
    LongTermSign,
    // Governance-related (v0.9.0)
    GovernanceDiv,
    Policy,
    Formula,
    Regulation,
    Clause,
    DataSovereignty,
    AccessControl,
    AuditLedger,
    DecisionQuorum,
    Votes,
    Threshold,
    Standard,
    User,
    Resource,
    Action,
    Entry,
    // Signature-related
    Signature,
    Algorithm,
    SignedBy,
    With,
    // Literals and punctuation
    Identifier(String),
    String(String),
    Period,
    Eof,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::MapCsm => write!(f, "MAP-CSM"),
            Token::CompressCsm => write!(f, "COMPRESS-CSM"),
            Token::DecompressCsm => write!(f, "DECOMPRESS-CSM"),
            Token::DictionaryRef => write!(f, "DICTIONARY-REF"),
            Token::ProtocolVersion => write!(f, "PROTOCOL-VERSION"),
            Token::Density => write!(f, "DENSITY"),
            Token::IdentificationDiv => write!(f, "IDENTIFICATION DIVISION"),
            Token::EnvironmentDiv => write!(f, "ENVIRONMENT DIVISION"),
            Token::DataDiv => write!(f, "DATA DIVISION"),
            Token::ProcedureDiv => write!(f, "PROCEDURE DIVISION"),
            Token::VerificationDiv => write!(f, "VERIFICATION DIVISION"),
            Token::GovernanceDiv => write!(f, "GOVERNANCE DIVISION"),
            Token::PreCondition => write!(f, "PRE-CONDITION"),
            Token::PostCondition => write!(f, "POST-CONDITION"),
            Token::Invariant => write!(f, "INVARIANT"),
            Token::Prove => write!(f, "PROVE"),
            Token::AssertKw => write!(f, "ASSERT"),
            Token::Satisfies => write!(f, "SATISFIES"),
            Token::AuditLog => write!(f, "AUDIT-LOG"),
            Token::ComplianceReport => write!(f, "COMPLIANCE-REPORT"),
            Token::Division => write!(f, "DIVISION"),
            Token::ProgramId => write!(f, "PROGRAM-ID"),
            Token::Author => write!(f, "AUTHOR"),
            Token::Version => write!(f, "VERSION"),
            Token::Os => write!(f, "OS"),
            Token::Arch => write!(f, "ARCH"),
            Token::RuntimeVersion => write!(f, "RUNTIME-VERSION"),
            Token::Input => write!(f, "INPUT"),
            Token::Output => write!(f, "OUTPUT"),
            Token::Compress => write!(f, "COMPRESS"),
            Token::VerifyIntegrity => write!(f, "VERIFY-INTEGRITY"),
            Token::Transcode => write!(f, "TRANSCODE"),
            Token::Filter => write!(f, "FILTER"),
            Token::Aggregate => write!(f, "AGGREGATE"),
            Token::Convert => write!(f, "CONVERT"),
            Token::Merge => write!(f, "MERGE"),
            Token::Split => write!(f, "SPLIT"),
            Token::Validate => write!(f, "VALIDATE"),
            Token::Extract => write!(f, "EXTRACT"),
            Token::Encrypt => write!(f, "ENCRYPT"),
            Token::Decrypt => write!(f, "DECRYPT"),
            Token::Display => write!(f, "DISPLAY"),
            Token::Print => write!(f, "PRINT"),
            Token::Read => write!(f, "READ"),
            Token::Open => write!(f, "OPEN"),
            Token::ReadFile => write!(f, "READ-FILE"),
            Token::WriteFile => write!(f, "WRITE-FILE"),
            Token::As => write!(f, "AS"),
            Token::Close => write!(f, "CLOSE"),
            Token::Checkpoint => write!(f, "CHECKPOINT"),
            Token::Replay => write!(f, "REPLAY"),
            Token::Set => write!(f, "SET"),
            Token::Add => write!(f, "ADD"),
            Token::Subtract => write!(f, "SUBTRACT"),
            Token::Multiply => write!(f, "MULTIPLY"),
            Token::Divide => write!(f, "DIVIDE"),
            Token::Max => write!(f, "MAX"),
            Token::Min => write!(f, "MIN"),
            Token::Abs => write!(f, "ABS"),
            Token::Concatenate => write!(f, "CONCATENATE"),
            Token::Substring => write!(f, "SUBSTRING"),
            Token::Length => write!(f, "LENGTH"),
            Token::Uppercase => write!(f, "UPPERCASE"),
            Token::Lowercase => write!(f, "LOWERCASE"),
            Token::Trim => write!(f, "TRIM"),
            Token::If => write!(f, "IF"),
            Token::Else => write!(f, "ELSE"),
            Token::Then => write!(f, "THEN"),
            Token::EndIf => write!(f, "END-IF"),
            Token::For => write!(f, "FOR"),
            Token::While => write!(f, "WHILE"),
            Token::Do => write!(f, "DO"),
            Token::EndFor => write!(f, "END-FOR"),
            Token::EndWhile => write!(f, "END-WHILE"),
            Token::In => write!(f, "IN"),
            Token::Define => write!(f, "DEFINE"),
            Token::Function => write!(f, "FUNCTION"),
            Token::EndFunction => write!(f, "END-FUNCTION"),
            Token::Parameters => write!(f, "PARAMETERS"),
            Token::Returns => write!(f, "RETURNS"),
            Token::Network => write!(f, "NETWORK"),
            Token::Node => write!(f, "NODE"),
            Token::At => write!(f, "AT"),
            Token::Self_ => write!(f, "SELF"),
            Token::Topology => write!(f, "TOPOLOGY"),
            Token::Pipeline => write!(f, "PIPELINE"),
            Token::Mesh => write!(f, "MESH"),
            Token::Star => write!(f, "STAR"),
            Token::Timeout => write!(f, "TIMEOUT"),
            Token::Send => write!(f, "SEND"),
            Token::Receive => write!(f, "RECEIVE"),
            Token::To => write!(f, "TO"),
            Token::From => write!(f, "FROM"),
            Token::Pipe => write!(f, "PIPE"),
            Token::CallRemote => write!(f, "CALL-REMOTE"),
            Token::VideoMp4 => write!(f, "VIDEO-MP4"),
            Token::ImageJpg => write!(f, "IMAGE-JPG"),
            Token::FinancialDecimal => write!(f, "FINANCIAL-DECIMAL"),
            Token::AudioWav => write!(f, "AUDIO-WAV"),
            Token::CsvTable => write!(f, "CSV-TABLE"),
            Token::BinaryBlob => write!(f, "BINARY-BLOB"),
            Token::JsonObject => write!(f, "JSON-OBJECT"),
            Token::XmlDocument => write!(f, "XML-DOCUMENT"),
            Token::ParquetTable => write!(f, "PARQUET-TABLE"),
            Token::TextString => write!(f, "TEXT-STRING"),
            Token::NumberInteger => write!(f, "NUMBER-INTEGER"),
            Token::NumberDecimal => write!(f, "NUMBER-DECIMAL"),
            Token::FileHandle => write!(f, "FILE-HANDLE"),
            Token::RecordStream => write!(f, "RECORD-STREAM"),
            Token::QuantumEncrypt => write!(f, "QUANTUM-ENCRYPT"),
            Token::QuantumDecrypt => write!(f, "QUANTUM-DECRYPT"),
            Token::QuantumSign => write!(f, "QUANTUM-SIGN"),
            Token::QuantumVerifySig => write!(f, "QUANTUM-VERIFY-SIG"),
            Token::QuantumSignEncrypt => write!(f, "QUANTUM-SIGN-ENCRYPT"),
            Token::QuantumVerifyDecrypt => write!(f, "QUANTUM-VERIFY-DECRYPT"),
            Token::GenerateKeypair => write!(f, "GENERATE-KEYPAIR"),
            Token::LongTermSign => write!(f, "LONG-TERM-SIGN"),
            Token::Policy => write!(f, "POLICY"),
            Token::Formula => write!(f, "FORMULA"),
            Token::Regulation => write!(f, "REGULATION"),
            Token::Clause => write!(f, "CLAUSE"),
            Token::DataSovereignty => write!(f, "DATA-SOVEREIGNTY"),
            Token::AccessControl => write!(f, "ACCESS-CONTROL"),
            Token::AuditLedger => write!(f, "AUDIT-LEDGER"),
            Token::DecisionQuorum => write!(f, "DECISION-QUORUM"),
            Token::Votes => write!(f, "VOTES"),
            Token::Threshold => write!(f, "THRESHOLD"),
            Token::Standard => write!(f, "STANDARD"),
            Token::User => write!(f, "USER"),
            Token::Resource => write!(f, "RESOURCE"),
            Token::Action => write!(f, "ACTION"),
            Token::Entry => write!(f, "ENTRY"),
            Token::Signature => write!(f, "SIGNATURE"),
            Token::Algorithm => write!(f, "ALGORITHM"),
            Token::SignedBy => write!(f, "SIGNED-BY"),
            Token::With => write!(f, "WITH"),
            Token::Identifier(s) => write!(f, "IDENTIFIER({})", s),
            Token::String(s) => write!(f, "STRING({})", s),
            Token::Period => write!(f, "."),
            Token::Eof => write!(f, "EOF"),
        }
    }
}

/// Tokenize CENTRA-NF source code.
/// Rejects unrecognized characters immediately.
pub fn tokenize(source: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();

    while let Some(&_ch) = chars.peek() {
        todo!()
    }

    tokens.push(Token::Eof);
    Ok(tokens)
}

// ...existing code...

/// Tokenize CENTRA-NF source code.
/// Rejects unrecognized characters immediately.
// ...fungsi duplikat dihapus...

/// Convert identifier string to keyword token, or Identifier if not a keyword.
// ...fungsi duplikat dihapus...

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_rejects_unknown_character() {
        let result = tokenize("COMPRESS @");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unrecognized character '@'"));
    }

    #[test]
    fn test_lexer_handles_identifiers() {
        let tokens = tokenize("PROGRAM-ID MyApp").unwrap();
        assert_eq!(tokens[0], Token::ProgramId);
        assert_eq!(tokens[1], Token::Identifier("MyApp".to_string()));
    }

    #[test]
    fn test_lexer_recognizes_encrypt_decrypt() {
        let tokens = tokenize("ENCRYPT BUFFER DECRYPT BUFFER").unwrap();
        assert_eq!(tokens[0], Token::Encrypt);
        assert_eq!(tokens[2], Token::Decrypt);
    }

    #[test]
    fn test_lexer_recognizes_quantum_encrypt() {
        let tokens = tokenize("QUANTUM-ENCRYPT").unwrap();
        assert_eq!(tokens[0], Token::QuantumEncrypt);
    }

    #[test]
    fn test_lexer_recognizes_quantum_decrypt() {
        let tokens = tokenize("QUANTUM-DECRYPT").unwrap();
        assert_eq!(tokens[0], Token::QuantumDecrypt);
    }

    #[test]
    fn test_lexer_recognizes_quantum_sign() {
        let tokens = tokenize("QUANTUM-SIGN").unwrap();
        assert_eq!(tokens[0], Token::QuantumSign);
    }

    #[test]
    fn test_lexer_recognizes_quantum_verify_sig() {
        let tokens = tokenize("QUANTUM-VERIFY-SIG").unwrap();
        assert_eq!(tokens[0], Token::QuantumVerifySig);
    }

    #[test]
    fn test_lexer_recognizes_quantum_sign_encrypt() {
        let tokens = tokenize("QUANTUM-SIGN-ENCRYPT").unwrap();
        assert_eq!(tokens[0], Token::QuantumSignEncrypt);
    }

    #[test]
    fn test_lexer_recognizes_quantum_verify_decrypt() {
        let tokens = tokenize("QUANTUM-VERIFY-DECRYPT").unwrap();
        assert_eq!(tokens[0], Token::QuantumVerifyDecrypt);
    }

    #[test]
    fn test_lexer_recognizes_generate_keypair() {
        let tokens = tokenize("GENERATE-KEYPAIR").unwrap();
        assert_eq!(tokens[0], Token::GenerateKeypair);
    }

    #[test]
    fn test_lexer_recognizes_long_term_sign() {
        let tokens = tokenize("LONG-TERM-SIGN").unwrap();
        assert_eq!(tokens[0], Token::LongTermSign);
    }

    #[test]
    fn test_lexer_recognizes_quantum_supporting_tokens() {
        let tokens = tokenize("SIGNATURE ALGORITHM SIGNED-BY WITH").unwrap();
        assert_eq!(tokens[0], Token::Signature);
        assert_eq!(tokens[1], Token::Algorithm);
        assert_eq!(tokens[2], Token::SignedBy);
        assert_eq!(tokens[3], Token::With);
    }
}
