use snafu::prelude::*;
use std::{io, path::Path, process::Command};
use tempfile::tempdir;
mod compiler;

#[derive(Debug, Snafu)]
pub enum Error {
    Preprocessing { source: std::io::Error },
    Compilation { source: compiler::CompilationError },
    Assemble { source: std::io::Error },
    Driver { source: std::io::Error },
}

#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    Lex,
    Parse,
    Codegen,
    Assembly,
}

pub fn driver(input: &Path, mode: Option<Mode>) -> Result<(), Error> {
    // Temp directory is removed when dropped.
    let temp_dir = tempdir().context(DriverSnafu)?;
    let file_name = input.file_name().expect("Validated by main");

    let preprocessed = temp_dir
        .path()
        .with_file_name(file_name)
        .with_extension("i");
    preprocess(input, &preprocessed).context(PreprocessingSnafu)?;

    let assembly = temp_dir
        .path()
        .with_file_name(file_name)
        .with_extension("s");
    compiler::compile(&preprocessed, &assembly, mode).context(CompilationSnafu)?;

    let executable = input.with_extension("");
    assemble(&assembly, &executable).context(AssembleSnafu)?;

    Ok(())
}

fn preprocess(input: &Path, output: &Path) -> io::Result<()> {
    Command::new("gcc")
        .arg("-E")
        .arg("-P")
        .arg(input)
        .arg("-o")
        .arg(&output)
        .output()
        .map(|_| ())
}

fn assemble(input: &Path, output: &Path) -> io::Result<()> {
    Command::new("gcc")
        .arg(input)
        .arg("-o")
        .arg(&output)
        .output()
        .map(|_| ())
}
