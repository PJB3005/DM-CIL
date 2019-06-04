use dreammaker::ast::*;
use dreammaker::constants::Constant;
use dreammaker::Location;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct ByondPath {
    segments: Vec<String>,
    rooted: bool,
}

impl ByondPath {
    pub fn new<P: AsRef<str>>(segments: &[P], rooted: bool) -> ByondPath {
        let mut vec = vec![];
        for x in segments {
            vec.push(x.as_ref().to_owned());
        }
        ByondPath {
            segments: vec,
            rooted,
        }
    }

    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }

    pub fn last_segment(&self) -> &str {
        if self.segment_count() == 0 {
            panic!("Can't get the segment count on a zero-length path.");
        }

        &self.segments[self.segments.len() - 1]
    }

    pub fn is_rooted(&self) -> bool {
        self.rooted
    }
}

impl fmt::Display for ByondPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.rooted {
            write!(f, "/")?;
        }
        write!(f, "{}", self.segments.join("/"))
    }
}

impl<'a> From<&'a str> for ByondPath {
    fn from(val: &'a str) -> ByondPath {
        let mut rooted = false;
        let mut vec = vec![];
        for (i, section) in val.split('/').enumerate() {
            if section == "" {
                if i == 0 {
                    rooted = true;
                }

                continue;
            }
            vec.push(section.to_owned());
        }

        ByondPath {
            rooted,
            segments: vec,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CompilerState {
    pub types: HashMap<ByondPath, CompilerType>,
    pub global_procs: HashMap<String, Proc>,
    pub global_vars: HashMap<String, GlobalVar>,
}

#[derive(Debug, Clone)]
pub struct CompilerType {
    pub path: ByondPath,
    pub children: Vec<String>,
    pub procs: HashMap<String, Proc>,
    pub special_class: Option<SpecialClass>,
}

impl CompilerType {
    pub fn new(path: &ByondPath) -> CompilerType {
        CompilerType {
            path: path.clone(),
            children: vec![],
            procs: HashMap::new(),
            special_class: None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum SpecialClass {
    World,
}

#[derive(Clone, Debug)]
pub struct Proc {
    pub name: String,
    pub parameters: Vec<ProcParameter>,
    pub var_arg: bool,
    pub source: ProcSource,
    pub is_static: bool,
}

impl Proc {
    pub fn new(name: &str, source: ProcSource) -> Proc {
        Proc {
            name: name.to_owned(),
            parameters: vec![],
            var_arg: false,
            source,
            is_static: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProcParameter {
    pub name: String,
    pub var_type: VariableType,
}

impl ProcParameter {
    pub fn new(name: &str, var_type: VariableType) -> ProcParameter {
        ProcParameter {
            name: name.to_owned(),
            var_type,
        }
    }
}

#[derive(Clone, Debug)]
pub enum ProcSource {
    Std(StdProc),
    Code(Location),
}

#[derive(Clone, Debug)]
pub enum StdProc {
    Abs,
    WorldOutput,
    Sin,
    Cos,
    Unimplemented(String),
}

#[derive(Clone, Debug)]
pub enum VariableType {
    Unspecified,
    Object(ByondPath),
}

#[derive(Debug, Clone)]
pub struct GlobalVar {
    pub name: String,
    pub var_type: VariableType,
    pub initializer: Option<VariableInitializer>,
    pub mutability: VariableMutability,
}

impl GlobalVar {
    pub fn new<A>(name: A, var_type: &VariableType) -> GlobalVar
    where
        A: AsRef<str>,
    {
        GlobalVar {
            name: name.as_ref().to_owned(),
            var_type: var_type.clone(),
            initializer: None,
            mutability: VariableMutability::Normal,
        }
    }
}

#[derive(Debug, Clone)]
pub enum VariableInitializer {
    Constant(Constant),
    Expression(Expression),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariableMutability {
    Normal,
    Readonly,
    Constant,
}
