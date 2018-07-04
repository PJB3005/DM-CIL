use super::*;

use std::fmt;

pub struct Class {
    name: String,
    accessibility: ClassAccessibility,
    children: Vec<Class>,
    fields: Vec<Field>,
    methods: Vec<Method>,
}

impl Class {
    pub fn new(name: String, accessibility: ClassAccessibility) -> Class {
        Class {
            name,
            accessibility,
            children: vec![],
            methods: vec![],
            fields: vec![]
        }
    }

    pub fn get_children_mut(&mut self) -> &mut Vec<Class> {
        &mut self.children
    }
}

/// Accessibility/Visibiliy for classes.
/// Spec II.10.1.1
#[derive(Clone, Copy, Debug)]
pub enum ClassAccessibility {
    Private,
    Public,
    NestedAssembly,
    NestedFamily,
    NestedFamilyAndAssembly,
    NestedFamilyOrAssembly,
    NestedPrivate,
    NestedPublic,
}

impl fmt::Display for ClassAccessibility {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            ClassAccessibility::Private => "private",
            ClassAccessibility::Public => "public",
            ClassAccessibility::NestedAssembly => "nested assembly",
            ClassAccessibility::NestedFamily => "nested family",
            ClassAccessibility::NestedFamilyAndAssembly => "nested famandassem",
            ClassAccessibility::NestedFamilyOrAssembly => "nested famorassem",
            ClassAccessibility::NestedPrivate => "nested private",
            ClassAccessibility::NestedPublic => "nested public",
        })
    }
}