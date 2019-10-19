use crate::{eval::Var, Raw};
use std::collections::HashMap;

/// This guys stores which variables are available in a given scope.
/// Scopes have parents; if they don't know about a variable, they'll
/// tell you to ask their parent about it. That's why parent is stored here.
pub struct Context {
    pub map: HashMap<String, Var>,
    /// An index into the array of Contexts Evaluator stores.
    pub parent: Option<usize>,
}

impl Context {
    /// This Context is often used as the parent context that all other contexts spawn from.
    /// STD stands for "standard" because this is the dictionary of standard functions.
    pub fn std() -> Self {
        let mut map = HashMap::new();
        map.insert(
            "DISPLAY".to_string(),
            Var::Function(Box::new(|args: Vec<Var>| {
                let output = args.iter().fold(String::new(), |acc, arg| {
                    format!("{} {}", acc, arg).trim().to_owned()
                });
                print!("{}", output);
                Var::Raw(Raw::Text(output))
            })),
        );
        map.insert(
            "+".to_string(),
            Var::Function(Box::new(|args: Vec<Var>| {
                Var::Raw(Raw::Number(
                    args[0]
                        .num()
                        .and_then(|x| args[1].num().map(|y| y + x))
                        .expect("Can only add #'s!"),
                ))
            })),
        );

        Self { map, parent: None }
    }

    /// The Context a new scope starts with.
    pub fn empty_child(parent: usize) -> Self {
        Self {
            map: HashMap::new(),
            parent: Some(parent),
        }
    }
}
