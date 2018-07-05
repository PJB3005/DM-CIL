extern crate dreammaker;
//#[macro_use]
//extern crate bitflags;
#[macro_use]
extern crate structopt;

use dreammaker as dm;
use dm::objtree::TypeRef;
use std::fs::File;
use std::path::PathBuf;

mod dm_std;
mod il;
mod dmstate;
mod proc_transpiler;

use dmstate::DMState;
use il::{Assembly, Class, ClassAccessibility, Field, InstructionBlob, Method, Instruction, MethodAccessibility, MethodVirtuality, FieldAccessibility};

use structopt::StructOpt;

fn main() -> std::io::Result<()> {
    let opt = Opt::from_args();
    let path = PathBuf::from(&opt.input);

    let state = DMState::load(&path)?;

    if opt.print_annotations {
        for annotation in state.get_all_annotations() {
            println!("{:?}", annotation);
        }
    }

    let mut asm = Assembly::new(path.file_stem().and_then(|s| s.to_str()).unwrap().to_owned());

    {
        let externs = asm.get_externs_mut();
        externs.push("mscorlib {
.publickeytoken = (B7 7A 5C 56 19 34 E0 89 )
.ver 4:0:0:0
}".to_owned());
        externs.push("System.Core
{
  .publickeytoken = (B7 7A 5C 56 19 34 E0 89 )
  .ver 4:0:0:0
}".to_owned());
        // We use C#'s dynamic system.
        // Even though we're not C#.
        // What're you gonna do about it?
        externs.push("Microsoft.CSharp
{
  .publickeytoken = (B0 3F 5F 7F 11 D5 0A 3A )
  .ver 4:0:0:0
}".to_owned());
        //externs.push("DM {}".to_owned());
    }

    create_everything(&mut asm, &state);

    let out_path = if let Some(out_path) = &opt.output {
        PathBuf::from(out_path)
    } else {
        let mut out_path = path.clone();
        out_path.set_extension("il");
        out_path
    };

    let mut file = File::create(&out_path).unwrap();
    asm.write(&mut file)?;
/*
    let node = tree.find("/mob").unwrap().get();

    let proc_type = &node.procs["Login"];
    //println!("{:?}", at.get_location(proc_type.value.location));

    for x in state.get_annotations(proc_type.value.location) {
        if let (loc, dm::annotation::Annotation::ProcHeader(_)) = x {
            println!("yes");
            let mut end = loc.end;
            end.column += 1;
            for x in state.get_annotations(end) {
                println!("THIS IS IT: {:?}", x);
            }
        }
        println!("{:?}", x);
    }
*/
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

    #[structopt(long = "annotations")]
    print_annotations: bool,
}

fn create_everything(asm: &mut Assembly, state: &DMState) {
    let mut stack = vec![];
    let tree = state.get_tree();
    let mut class_root = Class::new("byond_root".to_owned(), ClassAccessibility::Public, None, "byond_root".to_owned(), false);
    stack.push("byond_root".to_owned());

    let root = tree.root();
    create_vars(root, &mut class_root);

    class_root.insert_method(dm_std::create_global_cctor());

    for (name, typeproc) in &root.procs {
        let method = proc_transpiler::create_proc(typeproc, &mut class_root, &name, true, state);

        class_root.insert_method(method);
    }

    for child in root.children(state.get_tree()) {
        create_node(asm, &mut class_root, state, child, &mut stack);
    }

    dm_std::create_world_class(&mut class_root);

    asm.get_classes_mut().push(class_root);
}

fn create_node(asm: &mut Assembly, parent: &mut Class, state: &DMState, noderef: TypeRef, mut type_stack: &mut Vec<String>) {
    // NOTE: parent is for the HIERARCHY, NOT inheritance.

    let node = noderef.get();
    // TODO: Handle DM parent_type.
    let parent_type_name = type_stack.join("/");
    //println!("name: '{}' stack: {}", node.name, &parent_type_name);
    let mut class = Class::new(node.name.clone(),
                               ClassAccessibility::NestedPublic,
                               Some(parent_type_name.clone()),
                               format!("{}/{}", parent_type_name, node.name),
                               false);

    create_vars(noderef, &mut class);

    for (name, _) in &node.procs {
        //println!("{}/{}: {}", parent_type_name, node.name, name);

        let mut instructions = InstructionBlob::default();
        instructions.not_implemented("The compiler is too simple to compile this proc.");

        let method = Method::new(name.to_owned(), "object".to_owned(), MethodAccessibility::Public, MethodVirtuality::VirtualNewSlot, instructions, false);
        class.insert_method(method);
    }

    type_stack.push(node.name.clone());

    for child in noderef.children(state.get_tree()) {
        create_node(asm, &mut class, state, child, &mut type_stack);
    }

    type_stack.pop();

    parent.insert_child_class(class);
}

fn create_vars(node: TypeRef, class: &mut Class) {
    for (name, typevar) in &node.vars {
        let mut field = Field::default();
        field.name = name.to_owned();
        field.type_name = "object".to_owned();
        if let Some(decl) = &typevar.declaration {
            field.is_static = decl.var_type.is_static;
        }
        field.accessibility = FieldAccessibility::Public;
        class.insert_field(field);
    }
}




/*
fn write_node(file: &mut W, state: &DMState, node: TypeRef, stack: &mut Vec<String>) {
    println!("writing: {}", node.get().name);

    let (parent, access) = if stack.len() == 0 {
        ("[mscorlib]System.Object".to_owned(), "public")
    } else {
        (stack.join("/"), "nested public")
    };

    let mut name = node.get().name.to_owned();

    if name == "" {
        name = "byond_root".to_owned();
    }


    writeln!(file, ".class {0} auto ansi beforefieldinit {1}
extends {2}
{{
", access, name, parent).unwrap();

    for x in &node.get().procs {
        println!("PROC {}: {:?}", x.0, x.1);
        writeln!(file, ".method public virtual hidebysig ").unwrap();
        if let Some(_) = x.1.declaration {
            writeln!(file, "newslot");
        }
        writeln!(file, "instance default void A ()").unwrap();
    }

    for x in &node.get().vars {
        println!("VAR {}: {:?}", x.0, x.1);
    }

    stack.push(name);

    for x in node.children(state.get_tree()) {
        write_node(file, state, x, stack);
    }

    stack.pop();

    writeln!(file, "}}").unwrap()
}

/*
        Ok(parser::parse(self,
            indents::IndentProcessor::new(self,
                preprocessor::Preprocessor::new(self, dme.to_owned())?
            )
        ))
*/
*/