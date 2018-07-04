use std::fmt;

pub struct Method {
    name: String,
    accessibility: MethodAccessibility,
}

/// Method attributes corresponding to accessibility.
/// Spec II.15.4.2
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

