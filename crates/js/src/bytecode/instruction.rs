use super::{vm::Value, Register};

#[derive(Clone, Copy, Debug)]
pub struct VariableHandle(usize);

impl VariableHandle {
    #[must_use]
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    #[must_use]
    pub const fn index(&self) -> usize {
        self.0
    }
}

#[derive(Clone, Debug)]
pub enum Instruction {
    LoadImmediate {
        destination: Register,
        immediate: Value,
    },
    CreateVariable {
        handle: VariableHandle,
    },
    UpdateVariable {
        handle: VariableHandle,
        src: Register,
    },
    Add {
        lhs: Register,
        rhs: Register,
        dst: Register,
    },
    Subtract {
        lhs: Register,
        rhs: Register,
        dst: Register,
    },
    Multiply {
        lhs: Register,
        rhs: Register,
        dst: Register,
    },
    Divide {
        lhs: Register,
        rhs: Register,
        dst: Register,
    },
    BitwiseOr {
        lhs: Register,
        rhs: Register,
        dst: Register,
    },
    BitwiseAnd {
        lhs: Register,
        rhs: Register,
        dst: Register,
    },
    BitwiseXor {
        lhs: Register,
        rhs: Register,
        dst: Register,
    },
    LogicalAnd {
        lhs: Register,
        rhs: Register,
        dst: Register,
    },
    LogicalOr {
        lhs: Register,
        rhs: Register,
        dst: Register,
    },
}
