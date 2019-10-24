use super::{Parameters, Raw};
use crate::parse::Node;
use std::fmt;

/// A value that can be manipulated.
/// These tend to be stored in Contexts.
/// They can enter programs from Literals embedded in the source, as the result of calculations,
/// and from being accessed from the standard library where any number of them may be stored.
/// Under the current implementation, having functions act as variables is really annoying
/// and make a number of things more difficult than they should be.
pub enum Var {
    Raw(Raw),
    List(Vec<Var>),
    Lambda(Node),
    Function(Box<dyn Fn(Parameters) -> Var>),
}
impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Raw::*;

        match self {
            Var::Raw(r) => match r {
                Text(t) => write!(f, "\"{}\"", t),
                Number(n) => write!(f, "{}", n),
                Bool(b) => write!(f, "{}", b),
            },
            Var::List(l) => {
                write!(f, "[")?;
                for (i, var) in l.iter().enumerate() {
                    match i {
                        0 => write!(f, "{}", var)?,
                        _ => write!(f, ", {}", var)?,
                    }
                }
                write!(f, "]")
            }
            _ => write!(f, "You can't print functions yet!"),
        }
    }
}
impl Var {
    /// Yields the result of calling the function if the variable is one, and a String explaining
    /// that it isn't a function if it isn't.
    #[inline]
    pub fn fn_call(&self, args: Vec<Var>) -> Result<Var, String> {
        use Var::*;

        match self {
            Raw(_) => Err(format!("{} isn't a function!", self)),
            List(_) => Err(format!("Can't call list {}!", self)),
            Lambda(_) => Err(format!("Can't call lambda {} with fn_call!", self)),
            Function(f) => Ok((*f)(Parameters(args))),
        }
    }

    /// Returns a number if the given variable can be turned into one, and a message explaining why
    /// if it can't.
    #[inline]
    pub fn number(&self) -> Result<f64, String> {
        match self {
            Var::Raw(r) => match r {
                Raw::Number(n) => Ok(*n),
                Raw::Text(_) => Err("Can't coerce Text into number".to_string()),
                Raw::Bool(_) => Err("Can't coerce Bool into number".to_string()),
            },
            _ => Err("Can't coerce functions into numbers".to_string()),
        }
    }

    #[inline]
    pub fn string(&self) -> Result<String, String> {
        match self {
            Var::Raw(r) => Ok(match r {
                Raw::Number(n) => format!("{}", n),
                Raw::Bool(b) => format!("{}", b),
                Raw::Text(t) => t.to_string(),
            }),
            _ => Err("Can't parse functions into numbers".to_string()),
        }
    }

    #[inline]
    pub fn boolean(&self) -> Result<bool, String> {
        match self {
            Var::Raw(r) => match r {
                Raw::Number(_) => Err("Can't turn Number into bool".to_string()),
                Raw::Text(_) => Err("Can't turn Text into bool".to_string()),
                Raw::Bool(b) => Ok(*b),
            },
            _ => Err("Can't parse functions into booleans".to_string()),
        }
    }
}
