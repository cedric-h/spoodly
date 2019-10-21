#[derive(Clone, PartialEq, Debug)]
pub enum Token {
    StorageArrow,
    ArgsOpen,
    ArgsClose,
    BlockOpen,
    BlockClose,
    BinaryOperation(String),
    StringLiteral(String),
    Number(f64),
    Identifier(String),
}
