use std::fmt;
use std::io;
use super::InstructionBlob;

#[derive(Default)]
pub struct Method {
    pub name: String,
    pub return_type: String,
    pub accessibility: MethodAccessibility,
    pub code: InstructionBlob,
    pub virtuality: MethodVirtuality,
    pub is_static: bool,
    pub is_rt_special_name: bool,
    pub is_special_name: bool,
    pub params: Vec<MethodParameter>,
    pub locals: Vec<String>,
    pub maxstack: u16,
    pub is_entry_point: bool,
}

impl Method {
    pub fn new(name: String,
               return_type: String,
               accessibility: MethodAccessibility,
               virtuality: MethodVirtuality,
               code: InstructionBlob,
               is_static: bool) -> Method {
        Method {
            name,
            return_type,
            accessibility,
            code,
            virtuality,
            is_static,
            is_rt_special_name: false,
            is_special_name: false,
            params: vec![],
            locals: vec![],
            maxstack: 32,
            is_entry_point: false
        }
    }

    pub fn write<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writeln!(writer, ".method hidebysig {} {} {} {} {} default {} '{}' (",
               if self.is_rt_special_name { "rtspecialname" } else { "" },
               if self.is_special_name { "specialname" } else { "" },
               self.accessibility,
               self.virtuality,
               if self.is_static { "static" } else { "instance" },
               self.return_type,
               self.name)?;
        
        for (i, param) in self.params.iter().enumerate() {
            if i != 0 {
                write!(writer, ", ")?;
            }
            write!(writer, "{} {}", param.type_name, param.name)?;
        }

        writeln!(writer, ") cil managed\n{{")?;
        if self.is_entry_point {
            writeln!(writer, ".entrypoint")?;
        }
        writeln!(writer, ".maxstack {}", self.maxstack)?;

        if self.locals.len() != 0 {
            write!(writer, ".locals init (")?;
            for (i, local) in self.locals.iter().enumerate() {
                if i != 0 {
                    write!(writer, ", ")?;
                }
                write!(writer, "[{}] {}", i, local)?;
            }
            writeln!(writer, ")")?;
        }

        self.code.write(writer)?;

        writeln!(writer, "}}")?;

        Ok(())
    }
}

#[derive(Default, Debug, Clone)]
pub struct MethodParameter {
    pub name: String,
    pub type_name: String,
    pub custom_attributes: Vec<String>,
}

impl MethodParameter {
    pub fn new(name: &str, type_name: &str) -> MethodParameter {
        MethodParameter {
            name: name.to_owned(),
            type_name: type_name.to_owned(),
            custom_attributes: vec![],  
        }
    }
}

/// Method attributes corresponding to accessibility.
/// Spec II.15.4.2
#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum MethodAccessibility {
    CompilerControlled,
    Private,
    Public,
    Assembly,
    FamilyAndAssembly,
    FamilyOrAssembly,
    Family,
}

impl Default for MethodAccessibility {
    fn default() -> MethodAccessibility {
        MethodAccessibility::Private
    }
}

impl fmt::Display for MethodAccessibility {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            MethodAccessibility::CompilerControlled => "compilercontrolled",
            MethodAccessibility::Private => "private",
            MethodAccessibility::Public => "public",
            MethodAccessibility::Assembly => "assembly",
            MethodAccessibility::Family => "family",
            MethodAccessibility::FamilyAndAssembly => "famandassem",
            MethodAccessibility::FamilyOrAssembly => "famorassem",
        })
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum MethodVirtuality {
    NotVirtual,
    Virtual,
    VirtualNewSlot,
}

impl Default for MethodVirtuality {
    fn default() -> MethodVirtuality {
        MethodVirtuality::NotVirtual
    }
}

impl fmt::Display for MethodVirtuality {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            MethodVirtuality::VirtualNewSlot => "virtual newslot",
            MethodVirtuality::Virtual => "virtual",
            MethodVirtuality::NotVirtual => "",
        })
    }
}