use std::fmt;
use std::io;

#[derive(Default)]
pub struct Field {
    pub name: String,
    pub type_name: String,
    pub accessibility: FieldAccessibility,
    pub is_static: bool,
}

impl Field {
    pub fn write<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writeln!(writer, ".field {} {} {} '{}'",
                 self.accessibility,
                 if self.is_static { "static" } else { "" },
                 self.type_name, self.name)
    }
}

/// Field attributes corresponding to accessibility.
/// Spec II.16.1
#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum FieldAccessibility {
    CompilerControlled,
    Private,
    Public,
    Assembly,
    FamilyAndAssembly,
    FamilyOrAssembly,
    Family,
}

impl Default for FieldAccessibility {
    fn default() -> FieldAccessibility {
        FieldAccessibility::Private
    }
}

impl fmt::Display for FieldAccessibility {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            FieldAccessibility::CompilerControlled => "compilercontrolled",
            FieldAccessibility::Private => "private",
            FieldAccessibility::Public => "public",
            FieldAccessibility::Assembly => "assembly",
            FieldAccessibility::Family => "family",
            FieldAccessibility::FamilyAndAssembly => "famandassem",
            FieldAccessibility::FamilyOrAssembly => "famorassem",
        })
    }
}