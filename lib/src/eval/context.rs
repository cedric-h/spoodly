use super::{Parameters, Var};
use crate::Raw;
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

        macro_rules! insert_number_ops {
            ( $( $op:tt : $( $op_name:ident )* ( $op_symbol:expr, $convert:tt, $type:tt ) )* ) => {
                $(map.insert(
                    $op_symbol.to_string(),
                    Var::Function(Box::new(|args: Parameters| {
                        let args = args.$convert().expect(
                            concat!("Can only ", $( stringify!($op_name) , )* " numbers!")
                        );
                        Var::Raw(Raw::$type(args[0] $op args[1]))
                    })),
                );)*
            }
        }

        map.insert(
            "true".to_string(),
            Var::Raw(Raw::Bool(true))
        );
        map.insert(
            "false".to_string(),
            Var::Raw(Raw::Bool(false))
        );
        map.insert(
            "DISPLAY".to_string(),
            Var::Function(Box::new(|Parameters(args): Parameters| {
                let output = args.iter().fold(String::new(), |acc, arg| {
                    format!("{} {}", acc, arg).trim().to_owned()
                });
                print!("{}", output);
                Var::Raw(Raw::Text(output))
            })),
        );
        map.insert(
            "=".to_string(),
            Var::Function(Box::new(|args: Parameters| {
                Var::Raw(Raw::Bool(
                    if let Ok(nums) = args.nums() {
                        nums[0] == nums[1]
                    } else if let Ok(bools) = args.booleans() {
                        bools[0] == bools[1]
                    } else if let Ok(strings) = args.strings() {
                        strings[0] == strings[1]
                    } else {
                        false
                    }
                 ))
            }))
        );

        // format:
        /* Rust Version: long name("pseudo version", conversion function, output type) */
        insert_number_ops!(
            +: add          ("+",   nums,       Number  )
            -: subtract     ("-",   nums,       Number  )
            /: divide       ("/",   nums,       Number  )
            *: multiply     ("*",   nums,       Number  )
            %: modulo       ("MOD", nums,       Number  )

            >: less than    (">",   nums,       Bool    )
            <: greater than ("<",   nums,       Bool    )

            &&: AND         ("AND", booleans,   Bool    )
            ||: OR          ("OR",  booleans,   Bool    )
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
