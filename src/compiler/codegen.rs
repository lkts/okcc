use snafu::Snafu;

use crate::compiler::ast::{self, Ast};

// TODO probably needs to be a tree as well but works for now
#[derive(Debug)]
pub struct AsmProgram {
    pub function: FunctionDefinition,
}

#[derive(Debug)]
pub struct FunctionDefinition {
    pub name: String,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug)]
pub enum Instruction {
    Mov { src: Operand, dst: Operand },
    Ret,
}

#[derive(Debug)]
pub enum Operand {
    Imm(i32),
    Register,
}

#[derive(Debug, Snafu)]
pub enum AsmGenError {
    MissingNode,
    UnexpectedNode,
}

pub fn generate_assembly(ast: Ast) -> Result<AsmProgram, AsmGenError> {
    let ast_root = ast.root().ok_or_else(|| MissingNodeSnafu.build())?;

    let ast_function = expect_program(&ast, ast_root)?;
    let function = expect_function(&ast, ast_function)?;

    Ok(AsmProgram { function })
}

fn expect_program<'a>(ast: &'a Ast, node: &ast::Node) -> Result<&'a ast::Node, AsmGenError> {
    if let ast::Node::Program { function } = node {
        ast.get(function).ok_or_else(|| MissingNodeSnafu.build())
    } else {
        UnexpectedNodeSnafu.fail()
    }
}

fn expect_function(ast: &Ast, node: &ast::Node) -> Result<FunctionDefinition, AsmGenError> {
    if let ast::Node::FunctionDefinition(def) = node {
        let body = ast.get(&def.body).ok_or_else(|| MissingNodeSnafu.build())?;
        let instructions = expect_statement(ast, body)?;

        Ok(FunctionDefinition {
            name: def.name.clone(),
            instructions,
        })
    } else {
        UnexpectedNodeSnafu.fail()
    }
}

fn expect_statement(ast: &Ast, node: &ast::Node) -> Result<Vec<Instruction>, AsmGenError> {
    if let ast::Node::Statement(statement) = node {
        match statement {
            ast::Statement::Return { expression } => {
                let expression_node = ast
                    .get(expression)
                    .ok_or_else(|| MissingNodeSnafu.build())?;
                let operand = expect_expression(expression_node)?;

                return Ok(vec![
                    Instruction::Mov {
                        src: operand,
                        dst: Operand::Register,
                    },
                    Instruction::Ret,
                ]);
            }
        }
    } else {
        UnexpectedNodeSnafu.fail()
    }
}

fn expect_expression(node: &ast::Node) -> Result<Operand, AsmGenError> {
    if let ast::Node::Expression(expr) = node {
        match expr {
            ast::Expression::Constant(c) => Ok(Operand::Imm(*c)),
        }
    } else {
        UnexpectedNodeSnafu.fail()
    }
}
