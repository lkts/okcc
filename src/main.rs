use snafu::Snafu;
use snafu::prelude::*;
use std::{
    env::{self},
    path::PathBuf,
};

use okcc::{Mode, driver};

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("Must provide a path to a C file"))]
    MissingInputFile,
    #[snafu(display("Invalid file path: {}", path.display()))]
    InvalidPath {
        path: PathBuf,
    },
    CompilationError {
        source: okcc::Error,
    },
}

#[snafu::report]
fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();

    let mode: Option<Mode> = match args.get(1) {
        None => MissingInputFileSnafu.fail()?,
        Some(value) if value == "--lex" => Some(Mode::Lex),
        Some(value) if value == "--parse" => Some(Mode::Parse),
        Some(value) if value == "--codegen" => Some(Mode::Codegen),
        Some(value) if value == "-S" => Some(Mode::Assembly),
        Some(_) => None, // not a known flag so must be a path
    };

    let path: PathBuf = args
        .get(if mode.is_some() { 2 } else { 1 })
        .map(PathBuf::from)
        .unwrap();

    if path.exists() == false || path.is_file() == false {
        InvalidPathSnafu { path: path.clone() }.fail()?;
    }

    driver(&path, mode).context(CompilationSnafu)?;

    Ok(())
}
