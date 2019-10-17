#![feature(stmt_expr_attributes)]
use std::collections::HashMap;
use std::fmt;

pub fn eval<S: Into<String>>(src: S, ctx: Context) -> Result<Var, String> {
    Evaluator::new(ctx).eval(vec![parse(src)?], 0)
}

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
    pub fn call(&self, args: Vec<Var>) -> Result<Var, String> {
        use Var::*;

        match self {
            Raw(_) => Err(format!("{} isn't a function!", self)),
            List(_) => Err(format!("Can't call list {}!", self)),
            Function(f) => Ok((*f)(args)),
        }
    }

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

    /*
    pub fn boxed_clone(self) -> Self {
        use Var::*;

        match self {
            Raw(r) => Raw(r.clone()),
            Function(fn_box) => Function(fn_box.clone()),
        }
    }*/
}

pub struct Context {
    pub map: HashMap<String, Var>,
    pub parent: Option<usize>,
}
impl Context {
    pub fn std() -> Self {
        let mut map = HashMap::new();
        map.insert(
            "DISPLAY".to_string(),
            Var::Function(Box::new(|mut args: Vec<Var>| {
                //eprintln!("args len: {}", args.len());
                let mut output = String::new();
                while let Some(var) = args.pop() {
                    output = format!("{} {}", output, var).trim().to_owned();
                }
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
}

pub struct Evaluator {
    contexts: Vec<Context>,
}
impl Evaluator {
    pub fn new(context: Context) -> Self {
        Self {
            contexts: vec![context],
        }
    }

    pub fn eval(&mut self, mut ast: Ast, ctx: usize) -> Result<Var, String> {
        let mut vars = Vec::new();

        while let Some(node) = ast.pop() {
            eprintln!("evaluating {:?}", node);
            match node {
                Node::Block(children) => {
                    let children = children.iter().rev().map(|x| x.clone()).collect();
                    eprintln!("evaluating block, depth: {}", ctx);
                    let new_ctx = self.new_ctx(ctx);
                    vars.push(self.eval(children, new_ctx)?)
                }
                Node::Assign(id, val_node) => {
                    let to = self.eval(vec![*val_node], ctx)?;
                    eprintln!("assigning {}", id);
                    self.assign(ctx, id, to);
                }
                Node::Call(id, arg_node) => {
                    eprintln!("call arg_node: {:?}", arg_node);
                    let arg = self.eval(vec![*arg_node], ctx)?;

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

    fn new_ctx(&mut self, parent: usize) -> usize {
        self.contexts.push(Context {
            map: HashMap::new(),
            parent: Some(parent),
        });
        self.contexts.len() - 1
    }

    fn fetch(&self, ctx: usize, id: String) -> Result<&Var, String> {
        let Context { map, parent } = &self.contexts[ctx];
        map.get(&id)
            .or_else({
                let id = id.clone();
                move || parent.and_then(move |parent| self.fetch(parent, id).ok())
            })
            .ok_or(format!("couldn't find variable with identifier {}", id))
    }

    fn assign(&mut self, ctx: usize, id: String, to: Var) -> Option<Var> {
        self.contexts[ctx].map.insert(id, to)
    }
}

#[test]
fn test_eval() {
    fn eval<S: Into<String>>(source: S) -> Var {
        Evaluator::new(Context::std())
            .eval(
                vec![parse(source.into()).expect("couldn't parse source in eval test")],
                0,
            )
            .expect("error evaluating")
    }
    fn eval_raw<S: Into<String>>(source: S) -> Raw {
        match eval(source) {
            Var::Raw(r) => r,
            _ => panic!("eval_raw can't return that; must be function or list"),
        }
    }
    fn eval_list<S: Into<String>>(source: S) -> Vec<Raw> {
        match eval(source) {
            Var::List(l) => l
                .into_iter()
                .map(|v| match v {
                    Var::Raw(r) => r,
                    _ => panic!("list or raws not allowed inside list"),
                })
                .collect(),
            _ => panic!("even_list can't return that; must be function or raw"),
        }
    }

    assert_eq!(eval_raw("DISPLAY(3)"), Raw::Text("3".to_string()));

    assert_eq!(eval_raw("3+2+7"), Raw::Number(12.0),);

    assert_eq!(
        eval_raw(
            "s<-3+2+7
             DISPLAY(s)"
        ),
        Raw::Text("12".to_string())
    );
    assert_eq!(
        eval_list(
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
        vec![
            Raw::Text("14".to_string()),
            Raw::Text("1".to_string()),
            Raw::Text("4".to_string()),
        ]
    )
}

// an Abstract Syntax Tree is just a list of nodes.
type Ast = Vec<Node>;
#[derive(Debug, PartialEq, Clone)]
pub enum Node {
    // holders
    Block(Ast),
    Value(Raw),
    Var(String),
    // commands
    Assign(String, Box<Node>),
    Call(String, Box<Node>),
}
#[derive(Debug, PartialEq, Clone)]
pub enum Raw {
    Number(f64),
    Text(String),
}
impl Node {
    pub fn new_block() -> Self {
        Node::Block(Vec::new())
    }
}

struct Parser {
    tokens: Vec<Token>,
}
impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        let tokens = tokens.iter().rev().map(|x| x.clone()).collect();
        //eprintln!("{:?}", tokens);
        Self { tokens }
    }

    fn parse(&mut self, mut ast: Ast) -> Result<Ast, String> {
        if let Some(token) = self.tokens.pop() {
            //eprintln!("got {:?}", token);
            match token {
                Token::BlockOpen => {
                    let mut block_nodes = Ast::new();
                    loop {
                        let token = self
                            .tokens
                            .last()
                            .ok_or("blocks ain't supposed to close like that".to_string())?;
                        match token {
                            // remove the block then leave.
                            Token::BlockClose => {
                                self.tokens.pop();
                                break;
                            }
                            _ => {
                                block_nodes = self.parse(block_nodes)?;
                            }
                        }
                    }
                    ast.push(Node::Block(block_nodes));
                }
                Token::Identifier(a) => {
                    match self
                        .tokens
                        .last()
                        .ok_or("something's gotta follow an identifier".to_string())?
                    {
                        // if a storage arrow comes after the identifier,
                        // they're trying to assign the variable to a new value.
                        Token::StorageArrow => {
                            // remove the storage arrow because who would want that
                            self.tokens.pop();
                            // grab the thing after the arrow
                            let next_node = self
                                .parse(Ast::new())?
                                .pop()
                                .ok_or("arrow left us hangin'".to_string())?;
                            // push a new Assign Node into the AST where
                            // the Var we've found is assigned to the next_node.
                            //eprintln!("adding assign");
                            ast.push(Node::Assign(a, Box::new(next_node)));
                        }

                        // if a new block follows the identifier,
                        // it must be a function call.
                        Token::BlockOpen => {
                            ast.push(Node::Call(
                                a,
                                Box::new(
                                    self.parse(Ast::new())?
                                        .pop()
                                        .ok_or("nothing after assign".to_string())?,
                                ),
                            ));
                        }

                        // if neither of these follow an identifier,
                        // it must just be a reference to a variable.
                        _ => {
                            ast.push(Node::Var(a));
                        }
                    }
                }
                Token::StringLiteral(s) => {
                    ast.push(Node::Value(Raw::Text(s)));
                }
                Token::Number(n) => {
                    ast.push(Node::Value(Raw::Number(n)));
                }
                Token::BinaryOperation(op_name) => {
                    let left = ast.pop().ok_or("add what dude?".to_string())?;
                    ast.push(Node::Call(
                        op_name.clone(),
                        Box::new(Node::Block(vec![
                            left,
                            self.parse(Ast::new())?
                                .pop()
                                .ok_or(&format!("can't {} nothing", op_name))?,
                        ])),
                    ));
                }
                _ => {}
            }
        }
        Ok(ast
            .into_iter()
            .map(|node| match node {
                Node::Block(mut children) => {
                    if children.len() == 1 {
                        children.pop().unwrap()
                    } else {
                        Node::Block(children)
                    }
                }
                a => a,
            })
            .collect())
    }
}

pub fn parse<S: Into<String>>(src: S) -> Result<Node, String> {
    Parser::new(tokenize(src.into())?)
        .parse(Ast::new())?
        .pop()
        .ok_or("no output".to_string())
}

#[test]
fn test_parse() {
    use Node::*;

    assert_eq!(
        parse("s <- 3").unwrap(),
        Assign("s".to_string(), Box::new(Value(Raw::Number(3.0)))),
    );

    assert_eq!(
        parse("s<-3").unwrap(),
        Assign("s".to_string(), Box::new(Value(Raw::Number(3.0)))),
    );

    assert_eq!(
        parse("s<-3+2+7").unwrap(),
        Assign(
            "s".to_string(),
            Box::new(Call(
                "+".to_string(),
                Box::new(Block(vec![
                    Call(
                        "+".to_string(),
                        Box::new(Block(vec![
                            Value(Raw::Number(3.0)),
                            Value(Raw::Number(2.0)),
                        ])),
                    ),
                    Value(Raw::Number(7.0)),
                ]))
            )),
        )
    );

    assert_eq!(
        parse(
            "s <- 3
             DISPLAY(s)"
        )
        .unwrap(),
        Block(vec![
            Assign("s".to_string(), Box::new(Value(Raw::Number(3.0)))),
            Call("DISPLAY".to_string(), Box::new(Var("s".to_string())),)
        ]),
    );

    assert_eq!(
        parse(
            "\
            s <- 3
            l <- 4
            a <- 1
            s <- a + 5
            l <- a
            a <- a + 3
            DISPLAY(s)
            DISPLAY(l)
            DISPLAY(a)\
        "
        )
        .unwrap(),
        Block(vec![
            Assign("s".to_string(), Box::new(Value(Raw::Number(3.0)))),
            Assign("l".to_string(), Box::new(Value(Raw::Number(4.0)))),
            Assign("a".to_string(), Box::new(Value(Raw::Number(1.0)))),
            Assign(
                "s".to_string(),
                Box::new(Call(
                    "+".to_string(),
                    Box::new(Block(vec![Var("a".to_string()), Value(Raw::Number(5.0))])),
                ))
            ),
            Assign("l".to_string(), Box::new(Var("a".to_string()))),
            Assign(
                "a".to_string(),
                Box::new(Call(
                    "+".to_string(),
                    Box::new(Block(vec![Var("a".to_string()), Value(Raw::Number(3.0))])),
                ))
            ),
            Call("DISPLAY".to_string(), Box::new(Var("s".to_string()))),
            Call("DISPLAY".to_string(), Box::new(Var("l".to_string()))),
            Call("DISPLAY".to_string(), Box::new(Var("a".to_string()))),
        ]),
    );
}

#[derive(Clone, PartialEq, Debug)]
enum Token {
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

fn tokenize<S: Into<String>>(source: S) -> Result<Vec<Token>, String> {
    use Token::*;

    let mut tokens: Vec<Token> = vec![BlockOpen, BlockOpen];
    let source = source.into();
    let mut chars = source.chars().peekable();

    // I want variadic push
    macro_rules! token_push {
        ( $($s:expr $(,)? )+ ) => { $( tokens.push($s); )+ };
    }

    let mut read_until_end_of_line = false;

    while let Some(c) = chars.next() {
        match c {
            '<' => match chars.peek() {
                Some('-') => {
                    chars.next();
                    read_until_end_of_line = true;
                    token_push!(StorageArrow, BlockOpen);
                }
                _ => token_push!(LessThan),
            },

            '\n' => {
                if read_until_end_of_line {
                    //eprintln!("adding BlockClose for reading to end of line.");
                    token_push!(BlockClose);
                    read_until_end_of_line = false;
                }
                token_push!(BlockClose, BlockOpen);
            }
            '(' => token_push!(BlockOpen),
            ')' => token_push!(BlockClose),
            '+' | '-' | '*' | '/' | '^' => token_push!(BinaryOperation(c.to_string())),
            c => {
                if c.is_whitespace() {
                    // do nothing
                } else if c.is_alphanumeric() || c == '"' {
                    let mut name = c.to_string();

                    if c != '"' {
                        while let Some(fc) = chars.peek() {
                            if fc.is_alphanumeric() {
                                name.push(chars.next().unwrap())
                            } else {
                                break;
                            }
                        }
                    } else {
                        while let Some(fc) = chars.peek() {
                            if *fc != '"' {
                                name.push(chars.next().unwrap())
                            } else {
                                break;
                            }
                        }
                    }

                    if c == '"' {
                        // pushing it if it's a string literal.
                        if !(chars.next() == Some('"')) {
                            return Err("Unfinished string literal".to_string());
                        }
                        name.remove(0);
                        token_push!(StringLiteral(name));
                    } else if let Ok(n) = name.parse() {
                        // numbers
                        token_push!(Number(n));
                    } else {
                        // then it's gotta be an identifier.
                        token_push!(Identifier(name));
                    }
                } else {
                    eprintln!("ignoring {}", c);
                }
            }
        }
    }
    if read_until_end_of_line {
        token_push!(BlockClose);
    }

    token_push!(BlockClose, BlockClose);
    Ok(tokens)
}

#[test]
fn test_tokenize() {
    use Token::*;

    assert_eq!(
        tokenize("s <- 3").unwrap(),
        #[rustfmt::skip]
        [
            BlockOpen,
                BlockOpen,
                    Identifier("s".to_string()), StorageArrow, BlockOpen,
                        Number(3.0),
                    BlockClose,
                BlockClose,
            BlockClose,
        ]
    );

    assert_eq!(
        tokenize("s<-3").unwrap(),
        #[rustfmt::skip]
        [
            BlockOpen,
                BlockOpen,
                    Identifier("s".to_string()), StorageArrow, BlockOpen,
                        Number(3.0),
                    BlockClose,
                BlockClose,
            BlockClose,
        ]
    );

    assert_eq!(
        tokenize("s <- 3 + 2").unwrap(),
        #[rustfmt::skip]
        [
            BlockOpen,
                BlockOpen,
                    Identifier("s".to_string()), StorageArrow, BlockOpen,
                        Number(3.0),
                        BinaryOperation("+".to_string()),
                        Number(2.0),
                    BlockClose,
                BlockClose,
            BlockClose,
        ]
    );
    assert_eq!(
        tokenize("s<-3+2").unwrap(),
        #[rustfmt::skip]
        [
            BlockOpen,
                BlockOpen,
                    Identifier("s".to_string()), StorageArrow, BlockOpen,
                        Number(3.0),
                        BinaryOperation("+".to_string()),
                        Number(2.0),
                    BlockClose,
                BlockClose,
            BlockClose,
        ]
    );

    assert_eq!(
        tokenize(
            "\
            s <- 3
            DISPLAY(s)\
        "
        )
        .unwrap(),
        #[rustfmt::skip]
        [
            BlockOpen,
                BlockOpen,
                    Identifier("s".to_string()), StorageArrow, BlockOpen,
                        Number(3.0),
                    BlockClose,
                BlockClose,
                BlockOpen,
                    Identifier("DISPLAY".to_string()), BlockOpen,
                        Identifier("s".to_string()),
                    BlockClose,
                BlockClose,
            BlockClose,
        ]
    );

    assert_eq!(
        tokenize(
            "\
            s<-3
            DISPLAY(s)\
        "
        )
        .unwrap(),
        #[rustfmt::skip]
        [
            BlockOpen,
                BlockOpen,
                    Identifier("s".to_string()), StorageArrow, BlockOpen,
                        Number(3.0),
                    BlockClose,
                BlockClose,
                BlockOpen,
                    Identifier("DISPLAY".to_string()), BlockOpen,
                        Identifier("s".to_string()),
                    BlockClose,
                BlockClose,
            BlockClose,
        ]
    );

    assert_eq!(
        tokenize(
            "\
            s <- 3
            l <- 4
            a <- 1
            s <- a + 5
            l <- a
            a <- a + 3
            DISPLAY(s)
            DISPLAY(l)
            DISPLAY(a)\
        "
        )
        .unwrap(),
        #[rustfmt::skip]
        vec![
            BlockOpen,
                BlockOpen,
                    Identifier("s".to_string()), StorageArrow, BlockOpen,
                        Number(3.0),
                    BlockClose,
                BlockClose,
                BlockOpen,
                    Identifier("l".to_string()), StorageArrow, BlockOpen,
                        Number(4.0),
                    BlockClose,
                BlockClose,
                BlockOpen,
                    Identifier("a".to_string()), StorageArrow, BlockOpen,
                        Number(1.0),
                    BlockClose,
                BlockClose,
                BlockOpen,
                    Identifier("s".to_string()), StorageArrow, BlockOpen,
                        Identifier("a".to_string()), BinaryOperation("+".to_string()), Number(5.0),
                    BlockClose,
                BlockClose,
                BlockOpen,
                    Identifier("l".to_string()), StorageArrow, BlockOpen,
                        Identifier("a".to_string()),
                    BlockClose,
                BlockClose,
                BlockOpen,
                    Identifier("a".to_string()), StorageArrow, BlockOpen,
                        Identifier("a".to_string()), BinaryOperation("+".to_string()), Number(3.0),
                    BlockClose,
                BlockClose,
                BlockOpen,
                    Identifier("DISPLAY".to_string()), BlockOpen,
                        Identifier("s".to_string()),
                    BlockClose,
                BlockClose,
                BlockOpen,
                    Identifier("DISPLAY".to_string()), BlockOpen,
                        Identifier("l".to_string()),
                    BlockClose,
                BlockClose,
                BlockOpen,
                    Identifier("DISPLAY".to_string()), BlockOpen,
                        Identifier("a".to_string()),
                    BlockClose,
                BlockClose,
            BlockClose,
        ]
    )
}
