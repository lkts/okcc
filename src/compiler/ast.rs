#[derive(Debug, PartialEq, Eq)]
pub struct Ast {
    nodes: Vec<Node>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct NodeRef(usize);

impl NodeRef {
    #[cfg(test)]
    pub fn from_raw(idx: usize) -> Self {
        Self(idx)
    }
}

// TODO remove Option, it's very annoying, the AST should be correct when constructed
// Also remove references somehow?
impl Ast {
    pub fn root(&self) -> Option<&Node> {
        self.nodes.last()
    }

    pub fn get(&self, node_ref: &NodeRef) -> Option<&Node> {
        self.nodes.get(node_ref.0)
    }

    #[cfg(test)]
    pub fn from_raw_nodes(nodes: Vec<Node>) -> Self {
        Self { nodes }
    }
}

pub struct AstBuilder {
    nodes: Vec<Node>,
}

impl AstBuilder {
    pub fn new() -> Self {
        AstBuilder { nodes: Vec::new() }
    }

    pub fn add(&mut self, node: Node) -> NodeRef {
        let next_idx = self.nodes.len();
        self.nodes.push(node);

        NodeRef(next_idx)
    }

    pub fn build(self) -> Ast {
        Ast { nodes: self.nodes }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Node {
    Program { function: NodeRef },
    FunctionDefinition(FunctionDefinition),
    Statement(Statement),
    Expression(Expression),
}

#[derive(Debug, PartialEq, Eq)]
pub struct FunctionDefinition {
    pub name: String,
    pub body: NodeRef,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Statement {
    Return { expression: NodeRef },
}

#[derive(Debug, PartialEq, Eq)]
pub enum Expression {
    Constant(i32),
}
