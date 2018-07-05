use std::io;
use super::Class;

pub struct Assembly {
    name: String,
    externs: Vec<String>,
    classes: Vec<Class>,
}

impl Assembly {
    pub fn new(name: String) -> Self {
        Assembly {
            name,
            externs: vec![],
            classes: vec![],
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_externs(&self) -> &[String] {
        &self.externs
    }

    pub fn get_externs_mut(&mut self) -> &mut Vec<String> {
        &mut self.externs
    }

    pub fn get_classes(&self) -> &[Class] {
        &self.classes
    }

    pub fn get_classes_mut(&mut self) -> &mut Vec<Class> {
        &mut self.classes
    }

    pub fn write<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        for extern_assembly in &self.externs {
            writeln!(writer, ".assembly extern {} {{}}", extern_assembly)?;
        }

        writeln!(writer, ".assembly '{0}' {{}}\n.module {0}.dll", self.name)?;

        for class in &self.classes {
            class.write(writer)?;
        }

        Ok(())
    }
}