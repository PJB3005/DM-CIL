use std::path::Path;
use dm::annotation::{AnnotationTree, Iter};
use dm::objtree::ObjectTree;
use dm::{Context, Severity, Location};
use dm::preprocessor::Preprocessor;
use dm::indents::IndentProcessor;
use dm::parser::Parser;

/// Handles storage of the DM Object/Syntax trees and such.
pub(crate) struct DMState {
    annotations: AnnotationTree,
    tree: ObjectTree
}

impl DMState {
    pub fn load<P: AsRef<Path>>(path: P) -> DMState {
        let mut at = AnnotationTree::default();
        let tree = {
            let context = Context::default();
            let preprocess = Preprocessor::new(&context, path.as_ref().to_owned()).unwrap();
            let indents = IndentProcessor::new::<Preprocessor>(&context, preprocess);
            let mut parser = Parser::new(&context, indents);
            parser.annotate_to(&mut at);
            parser.run();

            let sloppy = context.errors().iter().any(|p| p.severity() == Severity::Error);
            let mut tree = parser.take_tree();
            tree.finalize(&context, sloppy);
            tree
        };

        DMState {
            annotations: at,
            tree: tree
        }
    }

    pub fn get_tree(&self) -> &ObjectTree {
        &self.tree
    }

    pub fn get_annotations(&self, loc: Location) -> Iter {
        self.annotations.get_location(loc)
    }
}