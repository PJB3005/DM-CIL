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

    pub fn absord(&mut self, mut other: InstructionBlob) {
        self.code.append(&mut other.code);
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
    _box(String),
    brfalse(String),
    brtrue(String),
    br(String),
    call(String),
    callvirt(String),
    dup,
    ldarg(u16),
    ldarg0,
    ldarg1,
    ldci4(i32),
    ldci40,
    ldci41,
    ldcr4(f32),
    ldfld(String),
    ldloc(u16),
    ldloc0,
    ldnull,
    ldsfld(String),
    ldstr(String),
    ldtoken(String),
    newarr(String),
    newobj(String),
    nop,
    pop,
    ret,
    stelemref,
    stloc(u16),
    stloc0,
    stsfld(String),
    throw,
    unbox(String),
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Instruction::*;
        match self {
            _box(meta) => write!(f, "box {}", meta),
            brfalse(label) => write!(f, "brfalse {}", label),
            brtrue(label) => write!(f, "brtrue {}", label),
            br(label) => write!(f, "br {}", label),
            call(meta) => write!(f, "call {}", meta),
            callvirt(meta) => write!(f, "callvirt {}", meta),
            dup => write!(f, "dup"), 
            ldarg(num) => write!(f, "ldarg {}", num),
            ldarg0 => write!(f, "ldarg.0"),
            ldarg1 => write!(f, "ldarg.1"),
            ldci40 => write!(f, "ldc.i4.0"),
            ldci41 => write!(f, "ldc.i4.1"),
            ldci4(num) => write!(f, "ldc.i4 {}", num),
            ldcr4(num) => write!(f, "ldc.r4 {}", num),
            ldfld(field) => write!(f, "ldfld {}", field),
            ldloc0 => write!(f, "ldloc.0"),
            ldloc(idx) => write!(f, "ldloc {}", idx),
            ldnull => write!(f, "ldnull"),
            ldsfld(field) => write!(f, "ldsfld {}", field),
            ldstr(literal) => write!(f, "ldstr \"{}\"", literal),
            ldtoken(meta) => write!(f, "ldtoken {}", meta),
            nop => write!(f, "nop"),
            newarr(constructor) => write!(f, "newarr {}", constructor),
            newobj(constructor) => write!(f, "newobj {}", constructor),
            pop => write!(f, "pop"),
            ret => write!(f, "ret"),
            stelemref => write!(f, "stelem.ref"),
            stloc(idx) => write!(f, "stloc {}", idx),
            stloc0 => write!(f, "stloc.0"),
            stsfld(field) => write!(f, "stsfld {}", field),
            throw => write!(f, "throw"),
            unbox(meta) => write!(f, "unbox {}", meta),
        }
    }
}


