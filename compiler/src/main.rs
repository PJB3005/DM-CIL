use dreammaker::{FileId, Location};
use std::fmt;
use std::fs::File;
use std::path::PathBuf;
use std::process::Command;

mod compiler_state;
mod dm_std;
mod dmstate;
mod il;
mod proc_transpiler;

use compiler_state::*;
use dmstate::DMState;
use il::*;

use structopt::StructOpt;

fn main() -> std::io::Result<()> {
    let opt = Opt::from_args();
    let path = PathBuf::from(&opt.input);

    let state = DMState::load(&path)?;

    let compiler_state = create_everything(&state);

    let mut asm = Assembly::new(
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap()
            .to_owned(),
    );

    write_everything(&mut asm, &state, &compiler_state);

    let (il_path, mut handle) = get_il_file(&opt)?;
    asm.write(&mut handle)?;
    handle.flush()?;

    if opt.noassemble {
        return Ok(());
    }

    let out_path = if let Some(out_path) = &opt.output {
        PathBuf::from(out_path)
    } else {
        let mut out_path = path.clone();
        out_path.set_extension("exe");
        out_path
    };

    let mut output_arg = std::ffi::OsStr::new("/output:").to_owned();
    output_arg.push(out_path.as_os_str());

    let status = Command::new("ilasm")
        .arg("/exe")
        .arg(output_arg)
        .arg(il_path)
        .status()?;

    if !status.success() {
        panic!("Assembly error!");
    }

    if !opt.nopeverify {
        let status = Command::new("peverify").arg(out_path).status()?;

        if !status.success() {
            panic!("Code validation failed.");
        }
    }

    Ok(())
}

#[derive(StructOpt)]
#[structopt(name = "DM CIL Compiler")]
struct Opt {
    /// The DM code file to compile.
    #[structopt(name = "input")]
    input: String,

    #[structopt(short = "o", long = "output")]
    output: Option<String>,

    /// Do not run peverify.exe.
    #[structopt(long = "nopeverify")]
    nopeverify: bool,

    /// Do not run ilasm.exe, only generate IL code.
    #[structopt(long = "noassemble")]
    noassemble: bool,

    /// Optional path to dump IL code to.
    #[structopt(long = "il")]
    il_path: Option<String>,
}

fn create_everything(dm_state: &DMState) -> CompilerState {
    let mut state = CompilerState::default();

    // Generate the std lib first.
    dm_std::create_std(&mut state);

    let tree = dm_state.get_tree();
    // Do globals first.
    let tree_root = tree.root().get();

    for (name, var) in &tree_root.vars {
        let declaration = var
            .declaration
            .as_ref()
            .expect("Global vars should have a declaration, right?");
        let value = &var.value;
        let var_type = if declaration.var_type.type_path.len() == 0 {
            VariableType::Unspecified
        } else {
            VariableType::Object(ByondPath::new(&declaration.var_type.type_path, true))
        };

        let initializer = if let Some(constant) = &value.constant {
            Some(VariableInitializer::Constant(constant.clone()))
        } else if let Some(expr) = &value.expression {
            Some(VariableInitializer::Expression(expr.clone()))
        } else {
            None
        };

        let mut global_var = GlobalVar::new(&name, &var_type);
        global_var.initializer = initializer;
        if declaration.var_type.is_const {
            global_var.mutability = VariableMutability::Constant;
        }
        state.global_vars.insert(name.clone(), global_var);
    }

    for (name, proc_type) in &tree_root.procs {
        if proc_type.value.len() > 1 {
            compiler_warning(format!("Skipping proc with multiple values: {}", &name));
            continue;
        }

        let value = &proc_type.value[0];

        let source = if value.location.file == FileId::builtins() {
            if !state.global_procs.contains_key(name) {
                ProcSource::Std(StdProc::Unimplemented(name.clone()))
            } else {
                // Implemented std proc that already exists!
                // Yay!
                continue;
            }
        } else {
            ProcSource::Code(value.location)
        };

        let mut global_proc = Proc::new(&name, source);
        for param in &value.parameters {
            let var_type = if param.var_type.type_path.len() > 0 {
                VariableType::Object(ByondPath::new(&param.var_type.type_path, true))
            } else {
                VariableType::Unspecified
            };
            let param = ProcParameter::new(&param.name, var_type);
            global_proc.parameters.push(param);
        }

        state.global_procs.insert(name.clone(), global_proc);
    }

    state
}

fn write_everything(asm: &mut Assembly, dm_state: &DMState, compiler_state: &CompilerState) {
    // Create externs.
    {
        let externs = asm.get_externs_mut();
        externs.push("mscorlib".to_owned());
        externs.push("System.Core".to_owned());
        // We use C#'s dynamic system.
        // Even though we're not C#.
        // What're you gonna do about it?
        externs.push("Microsoft.CSharp".to_owned());
        externs.push("DM".to_owned());
    }

    let mut stack = vec![];
    let mut class_root = Class::new(
        "byond_root".to_owned(),
        ClassAccessibility::Public,
        None,
        "byond_root".to_owned(),
        false,
    );
    stack.push("byond_root".to_owned());

    // Create global vars.
    for (name, var) in &compiler_state.global_vars {
        let mut field = Field::default();
        field.name = name.clone();
        field.type_name = "object".into();
        field.is_static = true;
        field.accessibility = FieldAccessibility::Public;
        if var.mutability != VariableMutability::Normal {
            field.is_initonly = true;
        }
        class_root.insert_field(field);
    }

    let global_cctor = dm_std::create_global_cctor(dm_state, compiler_state, &mut class_root);
    class_root.insert_method(global_cctor);
    class_root.insert_method(dm_std::create_stock_ctor("[mscorlib]System.Object"));

    {
        let mut code = InstructionBlob::default();
        code.instruction(Instruction::call("object byond_root::main()".into()));
        code.instruction(Instruction::pop);
        code.instruction(Instruction::ret);
        let mut entry_point = Method::new(
            "<>EntryPoint".into(),
            "void".into(),
            MethodAccessibility::Public,
            MethodVirtuality::NotVirtual,
            code,
            true,
        );
        entry_point.is_entry_point = true;
        entry_point.maxstack = 1;
        class_root.insert_method(entry_point);
    }

    for (name, global_proc) in &compiler_state.global_procs {
        let method = match &global_proc.source {
            ProcSource::Std(std) => Ok(dm_std::create_std_proc(std)),
            ProcSource::Code(_loc) => proc_transpiler::create_proc(
                &global_proc,
                &mut class_root,
                &name,
                true,
                dm_state,
                &compiler_state,
            ),
        };

        match method {
            Ok(method) => {
                class_root.insert_method(method);
            }
            Err(error) => println!("ERROR in proc {}: {}", name, error),
        };
    }

    for (_path, compiler_type) in compiler_state
        .types
        .iter()
        .filter(|(path, _)| path.segment_count() == 1)
    {
        let class = create_type(asm, compiler_type, compiler_state, dm_state, &mut stack);
        class_root.insert_child_class(class);
    }

    asm.get_classes_mut().push(class_root);
}

fn create_type(
    _asm: &mut Assembly,
    compiler_type: &CompilerType,
    compiler_state: &CompilerState,
    dm_state: &DMState,
    type_stack: &mut Vec<String>,
) -> Class {
    let parent_type_name = type_stack.join("/");
    let name = compiler_type.path.last_segment();
    let mut class = Class::new(
        name.into(),
        ClassAccessibility::NestedPublic,
        Some(parent_type_name.clone()),
        format!("{}/{}", parent_type_name, name),
        false,
    );

    // Make stock .ctor.
    let ctor = dm_std::create_stock_ctor(&parent_type_name);
    class.insert_method(ctor);

    for (name, child_proc) in &compiler_type.procs {
        let method = match &child_proc.source {
            ProcSource::Std(std) => Ok(dm_std::create_std_proc(std)),
            ProcSource::Code(_loc) => proc_transpiler::create_proc(
                &child_proc,
                &mut class,
                &name,
                true,
                dm_state,
                &compiler_state,
            ),
        };

        match method {
            Ok(method) => {
                class.insert_method(method);
            }
            Err(error) => println!("ERROR in proc {}: {}", name, error),
        };
    }

    type_stack.push(name.into());

    type_stack.pop();

    class
}

fn get_il_file(opt: &Opt) -> std::io::Result<(PathBuf, Box<std::io::Write>)> {
    if let Some(il_path) = &opt.il_path {
        Ok((PathBuf::from(il_path), Box::new(File::create(&il_path)?)))
    } else {
        if opt.noassemble {
            panic!("Set to not assemble IL code but no IL path specified!");
        }
        let tmp = tempfile::NamedTempFile::new()?;
        Ok((tmp.path().to_owned(), Box::new(tmp)))
    }
}

pub fn compiler_warning<A>(string: A)
where
    A: AsRef<str>,
{
    println!("WARNING: {}", string.as_ref());
}

#[derive(Clone, Debug)]
pub struct CompilerError {
    pub location: Option<Location>,
    pub end_location: Option<Location>,
    pub message: String,
}

impl<A> From<A> for CompilerError
where
    A: AsRef<str>,
{
    fn from(string_ref: A) -> CompilerError {
        CompilerError {
            location: None,
            end_location: None,
            message: string_ref.as_ref().to_owned(),
        }
    }
}

impl fmt::Display for CompilerError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "{}", &self.message)
    }
}
