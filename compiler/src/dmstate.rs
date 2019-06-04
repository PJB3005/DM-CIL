use dreammaker::indents::IndentProcessor;
use dreammaker::objtree::ObjectTree;
use dreammaker::parser::Parser;
use dreammaker::preprocessor::Preprocessor;
use dreammaker::Context;
use std::io;
use std::path::Path;

/// Handles storage of the DM Object/Syntax trees and such.
pub(crate) struct DMState {
    tree: ObjectTree,
}

impl DMState {
    pub fn load<P: AsRef<Path>>(path: P) -> io::Result<DMState> {
        let tree = {
            let context = Context::default();
            let preprocess = Preprocessor::new(&context, path.as_ref().to_owned())?;
            let indents = IndentProcessor::new::<Preprocessor>(&context, preprocess);
            let mut parser = Parser::new(&context, indents);
            parser.enable_procs();
            let tree = parser.parse_object_tree();

            tree
        };

        Ok(DMState { tree })
    }

    pub fn get_tree(&self) -> &ObjectTree {
        &self.tree
    }
}
