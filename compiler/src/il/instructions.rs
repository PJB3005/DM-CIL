use std::fmt;

/// The CIL instruction set,
/// or more precisely, how much of it we use.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub enum Instruction {
    nop,
    ldarg(u16),
    ldarg0,
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Instruction::nop => f.write_str("nop"),
            Instruction::ldarg(num) => write!(f, "ldarg {}", num),
            Instruction::ldarg0 => f.write_str("ldarg.0")
        }
    }
}


