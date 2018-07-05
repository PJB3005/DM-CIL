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
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn write<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writeln!(writer, ".method hidebysig {} {} {} default {} '{}' () cil managed\n{{",
               self.accessibility,
               self.virtuality,
               if self.is_static { "static" } else { "instance" },
               self.return_type,
               self.name)?;
            
        if self.name == "EntryPoint" {
            // TODO: Make this less hacky.
            writeln!(writer, ".entrypoint")?;
        }

        self.code.write(writer)?;

        writeln!(writer, "}}")?;

        Ok(())
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