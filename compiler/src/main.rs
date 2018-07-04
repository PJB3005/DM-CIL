#![allow(dead_code)]
extern crate dreammaker;
#[macro_use]
extern crate bitflags;

use dreammaker as dm;
use dm::objtree::{TypeRef};
use std::fs::File;
use std::io::prelude::*;

mod il;
mod dmstate;

use dmstate::DMState;
use il::Assembly;

fn main() {
    let state = DMState::load("derp.dm");
    let tree = state.get_tree();

    let mut asm = Assembly::default();

    {
        let externs = asm.get_externs_mut();
        externs.push("mscorlib".to_owned());
        externs.push("System.Core".to_owned());
        // We use C#'s dynamic system.
        externs.push("Microsoft.CSharp".to_owned());
    }

    let mut file = File::create("derp.il").unwrap();
    writeln!(file, "
.assembly extern mscorlib {{ }}
.assembly 'derp' {{ }}
.module derp.exe
").unwrap();

    let mut stack = vec![];

    write_node(&mut file, &state, tree.root(), &mut stack);
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
}

fn write_node<W: Write>(file: &mut W, state: &DMState, node: TypeRef, stack: &mut Vec<String>) {
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
