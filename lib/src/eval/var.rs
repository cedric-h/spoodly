use std::fmt;
use super::Raw;

/// A value that can be manipulated.
/// These tend to be stored in Contexts.
/// They can enter programs from Literals embedded in the source, as the result of calculations,
/// and from being accessed from the standard library where any number of them may be stored.
/// Under the current implementation, having functions act as variables is really annoying
/// and make a number of things more difficult than they should be.
pub enum Var {
    Raw(Raw),
    List(Vec<Var>),
    Function(Box<dyn Fn(Vec<Var>) -> Var>),
}
impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Raw::*;

        match self {
            Var::Raw(r) => match r {
                Text(t) => write!(f, "\"{}\"", t),
                Number(n) => write!(f, "{}", n),
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
    pub fn call(&self, args: Vec<Var>) -> Result<Var, String> {
        use Var::*;

        match self {
            Raw(_) => Err(format!("{} isn't a function!", self)),
            List(_) => Err(format!("Can't call list {}!", self)),
            Function(f) => Ok((*f)(args)),
        }
    }

    /// Returns a number if the given variable can be turned into one, and a message explaining why
    /// if it can't.
    pub fn num(&self) -> Result<f64, String> {
        match self {
            Var::Raw(r) => match r {
                Raw::Number(n) => Ok(*n),
                Raw::Text(t) => {
                    if let Ok(n) = t.parse() {
                        Ok(n)
                    } else {
                        Err("String contained unparseable number.".to_string())
                    }
                }
            },
            _ => Err("Can't parse functions into numbers".to_string()),
        }
    }
}

