#[derive(Clone, PartialEq, Debug)]
pub enum Token {
    StorageArrow,
    LessThan,
    ArgsOpen,
    ArgsClose,
    BlockOpen,
    BlockClose,
    BinaryOperation(String),
    StringLiteral(String),
    Number(f64),
    Identifier(String),
}
