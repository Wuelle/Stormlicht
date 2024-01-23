mod builder;
mod exception;
mod instruction;
mod program;
mod vm;

pub use builder::{BasicBlockBuilder, ProgramBuilder, Register};
pub use exception::{Exception, ThrowCompletionOr};
pub use instruction::{Instruction, VariableHandle};
pub use program::{BasicBlock, BasicBlockExit, Program};
pub use vm::Vm;

pub trait CompileToBytecode {
    type Result = ();

    fn compile(&self, builder: &mut ProgramBuilder) -> Self::Result;
}
