use super::*;

use std::fmt;
use std::io;
use std::collections::HashMap;

pub struct Class {
    name: String,
    full_name: String,

    /// This is the INHERITANCE parent.
    parent: String,
    accessibility: ClassAccessibility,
    children: HashMap<String, Class>,
    fields: HashMap<String, Field>,
    methods: HashMap<String, Method>,
    is_static: bool
}

impl Class {
    pub fn new(name: String, accessibility: ClassAccessibility, parent: Option<String>, full_name: String, is_static: bool) -> Class {
        Class {
            name,
            full_name,
            parent: parent.unwrap_or("[mscorlib]System.Object".to_owned()),
            accessibility,
            children: HashMap::new(),
            methods: HashMap::new(),
            fields: HashMap::new(),
            is_static,
        }
    }

    pub fn get_full_name(&self) -> &str {
        &self.full_name
    }

    pub fn get_accessibility(&self) -> ClassAccessibility {
        self.accessibility
    }

    pub fn insert_child_class(&mut self, class: Class) -> Option<Class> {
        if !class.get_accessibility().is_nested() {
            panic!("Child class must use one of the nested accessibility modifiers.");
        }
        self.children.insert(class.name.clone(), class)
    }

    pub fn insert_field(&mut self, field: Field) -> Option<Field> {
        self.fields.insert(field.name.to_owned(), field)
    }

    pub fn insert_method(&mut self, method: Method) -> Option<Method> {
        self.methods.insert(method.name.to_owned(), method)
    }

    pub fn has_child_class(&self, name: &str) -> bool {
        self.children.contains_key(name)
    }

    pub fn get_child_class(&self, name: &str) -> Option<&Class> {
        self.children.get(name)
    }

    pub fn get_child_class_mut(&mut self, name: &str) -> Option<&mut Class> {
        self.children.get_mut(name)
    }

    pub fn get_field(&self, name: &str) -> Option<&Field> {
        self.fields.get(name)
    }

    pub fn get_method(&self, name: &str) -> Option<&Method> {
        self.methods.get(name)
    }

    pub fn write<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writeln!(writer, ".class {} auto ansi {} beforefieldinit '{}' extends {} {{",
                 self.accessibility,
                 if self.is_static { "abstract sealed"} else { "" },
                 self.name,
                 self.parent)?;

        for field in self.fields.values() {
            field.write(writer)?;
        }

        for method in self.methods.values() {
            method.write(writer)?;
        }

        for class in self.children.values() {
            class.write(writer)?;
        }

        writeln!(writer, "}}")?;
        Ok(())
    }
}

/// Accessibility/Visibiliy for classes.
/// Spec II.10.1.1
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

impl ClassAccessibility {
    pub fn is_nested(self) -> bool {
        self != ClassAccessibility::Public && self != ClassAccessibility::Private
    }
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