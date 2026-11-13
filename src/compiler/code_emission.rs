use crate::compiler::codegen::{AsmProgram, FunctionDefinition, Instruction, Operand};
use std::fmt::Write;

// TODO cleanup unwraps
pub fn emit(program: AsmProgram) -> String {
    let mut output = String::new();

    emit_function(program.function, &mut output);

    writeln!(output, ".section .note.GNU-stack,\"\",@progbits").unwrap();

    output
}

fn emit_function(function: FunctionDefinition, output: &mut String) {
    writeln!(output, "    .globl {}", function.name).unwrap();
    writeln!(output, "{}:", function.name).unwrap();
    emit_instructions(function.instructions, output);
}

fn emit_instructions(instructions: Vec<Instruction>, output: &mut String) {
    for instruction in instructions {
        match instruction {
            Instruction::Mov { src, dst } => {
                write!(output, "movl    ").unwrap();
                emit_operand(src, output);
                write!(output, ", ").unwrap();
                emit_operand(dst, output);
                writeln!(output).unwrap();
            }
            Instruction::Ret => writeln!(output, "ret").unwrap(),
        }
    }
}

fn emit_operand(operand: Operand, dest: &mut String) {
    match operand {
        Operand::Imm(n) => write!(dest, "${n}").unwrap(),
        Operand::Register => write!(dest, "%eax").unwrap(),
    }
}
