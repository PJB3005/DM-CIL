use std::collections::HashMap;
use dreammaker::Location;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct ByondPath {
    segments: Vec<String>,
    rooted: bool
}

impl ByondPath {
    pub fn new<P: AsRef<str>>(segments: &[P], rooted: bool) -> ByondPath {
        let mut vec = vec![];
        for x in segments {
            vec.push(x.as_ref().to_owned());
        }
        ByondPath {
            segments: vec,
            rooted
        }
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
            segments: vec
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
}

impl CompilerType {
    pub fn new(path: &ByondPath) -> CompilerType {
        CompilerType {
            path: path.clone(),
            children: vec![],
            procs: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Proc {
    pub name: String,
    pub parameters: Vec<ProcParameter>,
    pub var_arg: bool,
    pub source: ProcSource,
    pub is_static: bool
}

impl Proc {
    pub fn new(name: &str, source: ProcSource) -> Proc {
        Proc {
            name: name.to_owned(),
            parameters: vec![],
            var_arg: false,
            source,
            is_static: false
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProcParameter {
    pub name: String,
    pub var_type: VariableType
}

impl ProcParameter {
    pub fn new(name: &str, var_type: VariableType) -> ProcParameter {
        ProcParameter {
            name: name.to_owned(),
            var_type
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
    Unimplemented(String)
}

#[derive(Clone, Debug)]
pub enum VariableType {
    Unspecified,
    Object(ByondPath)
}

#[derive(Debug, Clone)]
pub struct GlobalVar {
    pub name: String,
    pub var_type: VariableType,
}
