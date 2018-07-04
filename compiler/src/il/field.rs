use std::fmt;

pub struct Field {
    name: String,
    type_name: String,
    accessibility: FieldAccessibility
}

impl Field {
    pub fn new(name: String, type_name: String, accessibility: FieldAccessibility) -> Field {
        Field {
            name, type_name, accessibility
        }
    }
}

/// Field attributes corresponding to accessibility.
/// Spec II.16.1
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