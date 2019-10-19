use super::Raw;

// an Abstract Syntax Tree is just a list of nodes.
pub type Ast = Vec<Node>;

#[derive(Debug, PartialEq, Clone)]
pub enum Node {
    // holders
    Block(Ast),
    List(Ast),
    Value(Raw),
    Var(String),
    // commands
    Assign(String, Box<Node>),
    Call(String, Vec<Node>),
}
impl Node {
    pub fn new_block() -> Self {
        Node::Block(Vec::new())
    }
}
