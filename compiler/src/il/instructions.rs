use std::fmt;
use std::io;

/// Responsible for turning instructions into CIL code.
#[derive(Clone, Default)]
pub struct InstructionBlob {
    code: Vec<CodePart>
}

impl InstructionBlob {
    pub fn instruction(&mut self, instruction: Instruction) {
        self.code.push(CodePart::Instruction(instruction));
    }

    pub fn label(&mut self, label: String) {
        self.code.push(CodePart::Label(label));
    }

    pub fn write<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        for part in &self.code {
            match part {
                CodePart::Instruction(i) => writeln!(writer, "{}", i)?,
                CodePart::Label(l) => write!(writer, "{}: ", l)?,
            }
        }

        Ok(())
    }

    /// Write in a NotImplementedException throw.
    pub fn not_implemented(&mut self, reason: &str) {
        // I love these.
        // You can literally put them ANYWHERE.
        self.instruction(Instruction::ldstr(reason.to_owned()));
        self.instruction(Instruction::newobj("instance void [mscorlib]System.NotImplementedException::'.ctor' (string)".to_owned()));
        self.instruction(Instruction::throw);
    }
}

#[derive(Clone, Debug)]
enum CodePart {
    Instruction(Instruction),
    Label(String)
}

/// The CIL instruction set,
/// or more precisely, how much of it we use.
#[allow(non_camel_case_types, dead_code)]
#[derive(Clone, Debug)]
pub enum Instruction {
    ldarg(u16),
    ldarg0,
    ldcr4(f32),
    ldloc(u16),
    ldnull,
    ldsfld(String),
    ldstr(String),
    newobj(String),
    nop,
    pop,
    stsfld(String),
    throw,
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Instruction::ldarg(num) => write!(f, "ldarg {}", num),
            Instruction::ldarg0 => write!(f, "ldarg.0"),
            Instruction::ldcr4(num) => write!(f, "ldc.r4 {}", num), 
            Instruction::ldloc(idx) => write!(f, "ldloc {}", idx),
            Instruction::ldnull => write!(f, "ldnull"),
            Instruction::ldsfld(field) => write!(f, "ldsfld {}", field),
            Instruction::ldstr(literal) => write!(f, "ldstr \"{}\"", literal),
            Instruction::nop => write!(f, "nop"),
            Instruction::newobj(constructor) => write!(f, "newobj {}", constructor),
            Instruction::pop => write!(f, "pop"),
            Instruction::stsfld(field) => write!(f, "stsfld {}", field),
            Instruction::throw => write!(f, "throw"),
        }
    }
}


