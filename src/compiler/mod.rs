#[cfg(test)]
use proptest_derive::Arbitrary;
use snafu::{ResultExt, Snafu};
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

use crate::{
    Mode,
    compiler::{code_emission::emit, codegen::generate_assembly, lexer::lex, parser::parse},
};

mod ast;
mod code_emission;
mod codegen;
mod lexer;
mod parser;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum Token {
    Identifier {
        #[cfg_attr(test, proptest(regex("[a-zA-Z_][a-zA-Z0-9]*")))]
        value: String,
    },
    Constant {
        #[cfg_attr(test, proptest(filter("|n| n.is_negative() == false")))]
        value: i32,
    },
    Keyword(Keyword),
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Semicolon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum Keyword {
    Int,
    Return,
    Void,
}

#[derive(Debug, Snafu)]
pub enum CompilationError {
    SyntaxError { source: lexer::LexerError },
    ParseError { source: parser::ParserError },
    AssemblyGenerationError { source: codegen::AsmGenError },
}

pub fn compile(input: &Path, output: &Path, mode: Option<Mode>) -> Result<(), CompilationError> {
    let mut input_string = String::new();
    File::open(input)
        .unwrap()
        .read_to_string(&mut input_string)
        .unwrap();

    let tokens = lex(&input_string).context(SyntaxSnafu)?;
    if mode == Some(Mode::Lex) {
        println!("Lexer output: {tokens:?}");
        return Ok(());
    }

    let ast = parse(tokens).context(ParseSnafu)?;
    if mode == Some(Mode::Parse) {
        println!("Parser output: {ast:?}");
        return Ok(());
    }

    let assembly = generate_assembly(ast).context(AssemblyGenerationSnafu)?;
    if mode == Some(Mode::Codegen) {
        println!("Codegen output: {assembly:?}");
        return Ok(());
    }

    let assembly_file = emit(assembly);

    File::create(output)
        .unwrap()
        .write_all(assembly_file.as_bytes())
        .unwrap();

    Ok(())
}
