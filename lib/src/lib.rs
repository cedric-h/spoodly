#![feature(stmt_expr_attributes)]

pub mod eval;
pub mod lex;
pub mod parse;

pub use eval::{Context, Evaluator};
pub use lex::tokenize;
pub use parse::{ast, parse};

/// Returns the Result (which might be an error!) of running the source String that's provided.
/// # Panics:
/// This shouldn't panic. It might panic. Optimally, errors are handled and returned as Error
/// messages.
pub fn interpret<S: Into<String>>(src: S, ctx: Context) -> Result<eval::Var, String> {
    Evaluator::new(ctx).eval(vec![parse(src)?], 0)
}

/// Raw values are stored as literals in program code, or used inside of variables.
#[derive(Debug, PartialEq, Clone)]
pub enum Raw {
    Number(f64),
    Text(String),
}
