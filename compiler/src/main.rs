extern crate dreammaker;

use dreammaker as dm;
use dm::objtree::{ObjectTree, TypeRef};
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;

mod il;

fn main() {
    let mut at = dm::annotation::AnnotationTree::default();

    let tree = {
        let context = dm::Context::default();
        let preprocess = dm::preprocessor::Preprocessor::new(&context, Path::new("../derp.dm").to_owned()).unwrap();
        let indents = dm::indents::IndentProcessor::new::<dm::preprocessor::Preprocessor>(&context, preprocess);
        let mut parser = dm::parser::Parser::new(&context, indents);
        parser.annotate_to(&mut at);
        parser.run();

        let sloppy = context.errors().iter().any(|p| p.severity() == dm::Severity::Error);
        let mut tree = parser.take_tree();
        tree.finalize(&context, sloppy);
        tree
    };

    let mut file = File::create("derp.il").unwrap();
    writeln!(file, "
.assembly extern mscorlib {{ }}
.assembly 'derp' {{ }}
.module derp.exe

    ").unwrap();

    write_node(&mut file, &tree, tree.root());

    let node = tree.find("/mob").unwrap().get();
    println!("{:?}, {}", node.procs["Login"], at.len());

    let proc_type = &node.procs["Login"];
    //println!("{:?}", at.get_location(proc_type.value.location));

    for x in at.get_location(proc_type.value.location) {
        if let (loc, dm::annotation::Annotation::ProcHeader(_)) = x {
            println!("yes");
            let mut end = loc.end;
            end.column += 1;
            for x in at.get_location(end) {
                println!("THIS IS IT: {:?}", x);
            }
        }
        println!("{:?}", x);
    }
}

fn write_node<W: Write>(file: &mut W, tree: &ObjectTree, node: TypeRef) {
    println!("writing: {}", node.get().name);
    for x in node.children(tree) {
        write_node(file, tree, x);
    }
}

/*
        Ok(parser::parse(self,
            indents::IndentProcessor::new(self,
                preprocessor::Preprocessor::new(self, dme.to_owned())?
            )
        ))
*/
