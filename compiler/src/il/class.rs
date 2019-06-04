use super::*;

use std::collections::HashMap;
use std::fmt;
use std::io;

pub struct Class {
    pub name: String,
    pub full_name: String,

    /// This is the INHERITANCE parent.
    pub parent: String,
    pub accessibility: ClassAccessibility,
    pub children: HashMap<String, Class>,
    pub fields: HashMap<String, Field>,
    pub methods: HashMap<String, Method>,
    pub is_static: bool,
    pub beforefieldinit: bool,
}

#[allow(dead_code)]
impl Class {
    pub fn new(
        name: String,
        accessibility: ClassAccessibility,
        parent: Option<String>,
        full_name: String,
        is_static: bool,
    ) -> Class {
        Class {
            name,
            full_name,
            parent: parent.unwrap_or("[mscorlib]System.Object".to_owned()),
            accessibility,
            children: HashMap::new(),
            methods: HashMap::new(),
            fields: HashMap::new(),
            is_static,
            beforefieldinit: true,
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

    pub fn set_before_field_init(&mut self, before_field_init: bool) {
        self.beforefieldinit = before_field_init;
    }

    pub fn write<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writeln!(
            writer,
            ".class {} auto ansi {} {} '{}' extends {} {{",
            self.accessibility,
            if self.is_static {
                "abstract sealed"
            } else {
                ""
            },
            if self.beforefieldinit {
                "beforefieldinit"
            } else {
                ""
            },
            self.name,
            self.parent
        )?;

        let mut keys = self.fields.keys().collect::<Vec<&String>>();
        keys.sort_unstable();
        for key in keys {
            self.fields[key].write(writer)?;
        }

        let mut keys = self.methods.keys().collect::<Vec<&String>>();
        keys.sort_unstable();
        for key in keys {
            self.methods[key].write(writer)?;
        }

        let mut keys = self.children.keys().collect::<Vec<&String>>();
        keys.sort_unstable();
        for key in keys {
            self.children[key].write(writer)?;
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
