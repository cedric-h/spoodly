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

        macro_rules! insert_ops {
            ( $(
                $op:tt : $( $op_name:ident )* (
                    $op_symbol:expr,
                    $( ( $convert:tt, $type:tt $(: $postfix:tt )? $(| $prefix:tt )? ) $(,)? )+
                ) 
            )* ) => {
                $(map.insert(
                    $op_symbol.to_string(),
                    Var::Function(Box::new(|args: Parameters| {
                        $( if let Ok(args) = args.$convert() {
                            return Var::Raw(Raw::$type(args[0]$(.$postfix())? $op $($prefix)? args[1]));
                        };)+
                        Var::Raw(Raw::Text(concat!(
                            "Can only apply the ", concat!( $( stringify!($op_name), " ", )* ),
                            "operation to ", operator_error!(first: $( $convert ,)+ ),
                        ).to_string()))
                    })),
                );)*
            };

        }
        macro_rules! operator_error {
            ( first: $first:ident, $($tail:tt, )* ) => {
                concat!(
                    stringify!($first),
                    operator_error!( : $( $tail , )* ),
                )
            };

            ( : $middle:ident, $($tail:tt, )+ ) => {
                concat!(
                    ", ",
                    stringify!($middle),
                    operator_error!( : $( $tail , )+ ),
                )
            };

            // ending when more than one type is involved
            ( : $last:ident , ) => {
                concat!(
                    " or ",
                    stringify!($last),
                    "!",
                )
            };

            // ending when only one was involved.
            ( : ) => { "!" }
        }

        map.insert("true".to_string(), Var::Raw(Raw::Bool(true)));
        map.insert("false".to_string(), Var::Raw(Raw::Bool(false)));
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

        // format:
        /* Rust Version: long name("pseudo version", conversion function, output type) */
        insert_ops!(
            ==: equals      ("=",   (numbers, Bool), (booleans, Bool), (strings, Bool))
            +: add          ("+",   (numbers, Number), (strings, Text :clone |&))
            -: subtract     ("-",   (numbers, Number))
            /: divide       ("/",   (numbers, Number))
            *: multiply     ("*",   (numbers, Number))
            %: modulo       ("MOD", (numbers, Number))

            >: less than    (">",   (numbers, Bool))
            <: greater than ("<",   (numbers, Bool))

            &&: AND         ("AND", (booleans, Bool))
            ||: OR          ("OR",  (booleans, Bool))
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
