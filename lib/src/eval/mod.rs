mod var;
pub use var::Var;

mod context;
pub use context::Context;

use super::{
    ast::{Ast, Node},
    Raw,
};

/// An Evaluator evalutes source code and stores the Context that
/// source code is run inside of an manipulates/stores variables in.
pub struct Evaluator {
    contexts: Vec<Context>,
}
impl Evaluator {
    /// A good context to pass in here is Context::std(), so that the code
    /// that's run using this Evaluator has access to the standard dictionary.
    pub fn new(context: Context) -> Self {
        Self {
            contexts: vec![context],
        }
    }

    /// Runs the given AST. May manipulate Contexts stored in the Evaluator.
    /// Calls itself recursively to evaluate arbitrarily nested blocks.
    pub fn eval(&mut self, mut ast: Ast, ctx: usize) -> Result<Var, String> {
        let mut vars = Vec::new();

        while let Some(node) = ast.pop() {
            eprintln!("evaluating {:?}", node);
            match node {
                // Block is like a list, except the commands inside get their own scope
                // and only the last element is returned.
                Node::Block(children) => {
                    let children = children.iter().rev().map(|x| x.clone()).collect();
                    eprintln!("evaluating block, depth: {}", ctx);
                    let new_ctx = self.new_ctx(ctx);
                    let mut result = self.eval(children, new_ctx)?;
                    if let Var::List(mut children) = result {
                        result = if children.len() > 0 {
                            children
                                .pop()
                                .expect("length was greater than one but pop yielded nothing")
                        } else {
                            Var::List(vec![])
                        }
                    }

                    vars.push(result)
                }
                Node::List(children) => {
                    let children = children.iter().rev().map(|x| x.clone()).collect();
                    vars.push(self.eval(children, ctx)?)
                }
                Node::Assign(id, val_node) => {
                    let to = self.eval(vec![*val_node], ctx)?;
                    eprintln!("assigning {}", id);
                    self.assign(ctx, id, to);
                }
                Node::Call(id, args) => {
                    //eprintln!("call arg_node: {:?}", arg_node);
                    let arg = self.eval(args, ctx)?;

                    vars.push(self.fetch(ctx, id)?.call(match arg {
                        Var::List(args) => args,
                        _ => vec![arg],
                    })?);
                }
                Node::Value(raw) => vars.push(Var::Raw(raw)),
                Node::Var(id) => vars.push(match self.fetch(ctx, id)? {
                    Var::Raw(r) => Var::Raw(r.clone()),
                    Var::Function(_) => {
                        return Err("no using functions as variables yet".to_string())
                    }
                    Var::List(_) => return Err("no using lists as variables yet".to_string()),
                }),
            }
        }

        Ok(match vars.len() {
            1 => vars.pop().unwrap(),
            _ => Var::List(vars),
        })
    }

    /// This allocates a new empty context on the stack of contexts and returns the index of the
    /// new context that is created. All values in all ancestors of a context are accessible from
    /// the child context.
    fn new_ctx(&mut self, parent: usize) -> usize {
        self.contexts.push(Context::empty_child(parent));
        self.contexts.len() - 1
    }

    /// Recursively searches through a given context and then all of its ancestors for a certain
    /// value.
    fn fetch(&self, ctx: usize, id: String) -> Result<&Var, String> {
        let Context { map, parent } = &self.contexts[ctx];
        map.get(&id)
            .or_else({
                let id = id.clone();
                move || parent.and_then(move |parent| self.fetch(parent, id).ok())
            })
            .ok_or(format!("couldn't find variable with identifier {}", id))
    }

    /// Stores a new value in the most local context.
    fn assign(&mut self, ctx: usize, id: String, to: Var) -> Option<Var> {
        self.contexts[ctx].map.insert(id, to)
    }
}

#[test]
fn test_eval() {
    fn eval<S: Into<String>>(source: S) -> String {
        use std::sync::{Arc, Mutex};
        let mut testing_std = Context::std();
        let stdout = Arc::new(Mutex::new(String::new()));

        testing_std.map.insert(
            "DISPLAY".to_string(),
            Var::Function({
                let stdout = stdout.clone();
                Box::new(move |args: Vec<Var>| {
                    let output = args.iter().fold(String::new(), |acc, arg| {
                        format!("{} {}", acc, arg).trim().to_owned()
                    });
                    let mut stdout = stdout.lock().unwrap();
                    stdout.push_str(&output);
                    stdout.push('\n');
                    Var::Raw(Raw::Text(output))
                })
            }),
        );


        Evaluator::new(testing_std)
            .eval(
                vec![super::parse(source.into()).expect("couldn't parse source in eval test")],
                0,
            )
            .expect("error evaluating");

        let output = stdout.lock().unwrap().to_string();
        output
    }

    assert_eq!(
        eval("DISPLAY(3)"),
        "3\n".to_string()
    );

    assert_eq!(
        eval("3+2+7"),
        "".to_string()
    );

    assert_eq!(
        eval(
            "s<-3+2+7
             DISPLAY(s)"
        ),
        "12\n".to_string(),
    );
    assert_eq!(
        eval(
            "\
            s <- 3
            l <- 4
            a <- 1
            s <- a + 5 +4  + 4
            l <- a
            a <- a + 3
            DISPLAY(s)
            DISPLAY(l)
            DISPLAY(a)\
        "
        ),
        "14\n1\n4\n".to_string()
    )
}
