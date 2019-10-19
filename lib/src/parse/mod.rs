/// an Abstract Syntax Tree is parsed from Tokens.
/// The Tree is comprised of Nodes, which could represent function calls, literal values, or
/// variable identifiers.
pub mod ast;
pub use ast::{Ast, Node};

use super::{lex::Token, Raw};

/// A parser takes a sequence of tokens and turns them into an Abstract Syntax Tree.
struct Parser {
    tokens: Vec<Token>,
}
impl Parser {
    /// When creating a new Parser, you pass in the tokens you'd like for it to parse.
    fn new(tokens: Vec<Token>) -> Self {
        let tokens = tokens.iter().rev().map(|x| x.clone()).collect();
        //eprintln!("{:?}", tokens);
        Self { tokens }
    }

    /// Depletes the series of tokens stored internally, turning them into commands
    /// that can be stored in the AST that's passed in.
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

/// Takes source code, turns it into tokens, creates a new parser, passes it the tokens,
/// parses them into an AST, and returns said AST.
pub fn parse<S: Into<String>>(src: S) -> Result<Node, String> {
    Parser::new(super::tokenize(src.into())?)
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

