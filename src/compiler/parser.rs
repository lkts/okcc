use snafu::Snafu;

use crate::compiler::{
    Keyword, Token,
    ast::{Ast, AstBuilder, Expression, FunctionDefinition, Node, Statement},
};

#[derive(Debug, Snafu, PartialEq, Eq)]
pub enum ParserError {
    UnexpectedToken { token: Token },
    MissingToken { token: Token },
    MissingIdentifier,
}

pub fn parse(tokens: Vec<Token>) -> Result<Ast, ParserError> {
    let mut ast_builder = AstBuilder::new();

    let mut iterator = tokens.into_iter();
    let function = parse_function(&mut ast_builder, &mut iterator)?;

    let function_ref = ast_builder.add(function);

    // We should have parsed everything we need, so anything extra is unexpected
    if let Some(token) = iterator.next() {
        UnexpectedTokenSnafu { token }.fail()?;
    }

    let program = Node::Program {
        function: function_ref,
    };
    ast_builder.add(program);

    Ok(ast_builder.build())
}

fn parse_function(
    ast_builder: &mut AstBuilder,
    tokens: &mut impl Iterator<Item = Token>,
) -> Result<Node, ParserError> {
    // "int" <identifier> "(" "void" ")" "{" <statement> "}"

    expect(Token::Keyword(Keyword::Int), tokens)?;

    let function_name = expect_identifier(tokens)?;
    expect(Token::OpenParen, tokens)?;
    expect(Token::Keyword(Keyword::Void), tokens)?;
    expect(Token::CloseParen, tokens)?;

    expect(Token::OpenBrace, tokens)?;
    let function_body = parse_statement(ast_builder, tokens)?;
    expect(Token::CloseBrace, tokens)?;

    let function_body_node = ast_builder.add(Node::Statement(function_body));
    let function_definition = FunctionDefinition {
        name: function_name,
        body: function_body_node,
    };
    Ok(Node::FunctionDefinition(function_definition))
}

fn parse_statement(
    ast_builder: &mut AstBuilder,
    tokens: &mut impl Iterator<Item = Token>,
) -> Result<Statement, ParserError> {
    // "return" <exp> ";"

    expect(Token::Keyword(Keyword::Return), tokens)?;
    let expr = parse_expression(tokens)?;
    expect(Token::Semicolon, tokens)?;

    let expr_node = ast_builder.add(Node::Expression(expr));
    Ok(Statement::Return {
        expression: (expr_node),
    })
}

fn parse_expression(tokens: &mut impl Iterator<Item = Token>) -> Result<Expression, ParserError> {
    let constant = expect_constant(tokens)?;

    Ok(Expression::Constant(constant))
}

fn expect(token: Token, tokens: &mut impl Iterator<Item = Token>) -> Result<(), ParserError> {
    match tokens.next() {
        Some(t) if t == token => Ok(()),
        Some(t) => UnexpectedTokenSnafu { token: t.clone() }.fail(),
        None => MissingTokenSnafu { token }.fail(),
    }
}

fn expect_identifier(tokens: &mut impl Iterator<Item = Token>) -> Result<String, ParserError> {
    match tokens.next() {
        Some(Token::Identifier { value }) => Ok(value),
        Some(t) => UnexpectedTokenSnafu { token: t }.fail(),
        None => MissingIdentifierSnafu.fail(),
    }
}

fn expect_constant(tokens: &mut impl Iterator<Item = Token>) -> Result<i32, ParserError> {
    match tokens.next() {
        Some(Token::Constant { value }) => Ok(value),
        Some(t) => UnexpectedTokenSnafu { token: t }.fail(),
        None => MissingIdentifierSnafu.fail(),
    }
}

#[cfg(test)]
mod tests {

    use crate::compiler::ast::NodeRef;

    use super::*;
    #[test]
    fn return_two() {
        // See lexer tests.
        // Corresponds to:
        // ```
        // int main(void)
        // {
        //     return 2;
        // }
        // ```
        let tokens = vec![
            Token::Keyword(Keyword::Int),
            Token::Identifier {
                value: "main".to_string(),
            },
            Token::OpenParen,
            Token::Keyword(Keyword::Void),
            Token::CloseParen,
            Token::OpenBrace,
            Token::Keyword(Keyword::Return),
            Token::Constant { value: 2 },
            Token::Semicolon,
            Token::CloseBrace,
        ];

        // the top node of the ast is built last so the order here is "reversed"
        let ast = Ast::from_raw_nodes(vec![
            Node::Expression(Expression::Constant(2)),
            Node::Statement(Statement::Return {
                expression: NodeRef::from_raw(0),
            }),
            Node::FunctionDefinition(FunctionDefinition {
                name: "main".to_string(),
                body: NodeRef::from_raw(1),
            }),
            Node::Program {
                function: NodeRef::from_raw(2),
            },
        ]);

        assert_eq!(ast, parse(tokens).unwrap());
    }

    #[test]
    fn extra_junk() {
        // Corresponds to:
        // ```
        // int main(void)
        // {
        //     return 2;
        // }
        // // A single identifier outside of a declaration isn't a valid top-level construct
        // foo
        // ```
        let tokens = vec![
            Token::Keyword(Keyword::Int),
            Token::Identifier {
                value: "main".to_string(),
            },
            Token::OpenParen,
            Token::Keyword(Keyword::Void),
            Token::CloseParen,
            Token::OpenBrace,
            Token::Keyword(Keyword::Return),
            Token::Constant { value: 2 },
            Token::Semicolon,
            Token::CloseBrace,
            Token::Identifier {
                value: "foo".to_string(),
            },
        ];

        assert_eq!(
            Err(ParserError::UnexpectedToken {
                token: Token::Identifier {
                    value: "foo".to_string()
                }
            }),
            parse(tokens)
        );
    }
}
