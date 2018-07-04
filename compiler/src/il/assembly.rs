use super::Class;

#[derive(Default)]
pub struct Assembly {
    externs: Vec<String>,
    classes: Vec<Class>,
}

impl Assembly {
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
}