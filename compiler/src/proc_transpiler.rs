use std::collections::HashMap;
use dm::ast::{Statement, Expression, Term};
use dm::objtree::TypeProc;
use dm::annotation::Annotation;
use super::il::*;
use super::dmstate::DMState;

pub(crate) fn create_proc(procdef: &TypeProc, class: &mut Class, proc_name: &str, is_static: bool, state: &DMState) -> Method {
    println!("{}: {:?}", proc_name, procdef);
    if let Some(code) = get_proc_body_details(procdef, state) {
        let mut im_data = ProcTranspiler {
            instructions: InstructionBlob::default(),
            locals: HashMap::new(),
            uniques: 0,
            state,
            class,
            proc_name,
            is_static,
        };
        for statement in code {
            write_statement(statement, &mut im_data)
        }
    }


    Method::new(proc_name.to_owned(), "void".to_owned(), MethodAccessibility::Public, MethodVirtuality::NotVirtual, InstructionBlob::default(), is_static)
}

/// Shared data necessary across the entire proc transpile.
struct ProcTranspiler<'a> {
    pub instructions: InstructionBlob,
    pub locals: HashMap<String, u16>,
    pub uniques: u16,
    pub state: &'a DMState,
    pub class: &'a mut Class,
    pub proc_name: &'a str,
    pub is_static: bool,
}

fn write_statement(statement: &Statement, im_data: &mut ProcTranspiler) {
    // RULE: when this function is done, the stack is the same as before.
    match statement {
        Statement::Expr(exp) => {
            evaluate_expression(exp, im_data);
            // Pop the expression off again. Don't need it.
            im_data.instructions.instruction(Instruction::pop);
        },
        _ => {}
    }
}

fn evaluate_expression(expression: &Expression, im_data: &mut ProcTranspiler) {
    // RULE: when this function is done, there is ONE extra value on the stack.
    match expression {
        Expression::Base { unary, term, follow } => {
            evaluate_term(term, im_data);
        },
        Expression::BinaryOp { op, lhs, rhs } => {
            
        },
        _ => {
            im_data.instructions.not_implemented("Unable to handle expression type.");
        }
    }
}

fn evaluate_term(term: &Term, im_data: &mut ProcTranspiler) {
    // RULE: when this function is done, there is ONE extra value on the stack.
    match term {
        Term::Int(val) => {
            im_data.instructions.instruction(Instruction::ldcr4(*val as f32));
        }
        Term::Float(val) => {
            im_data.instructions.instruction(Instruction::ldcr4(*val));
        },
        Term::Null => {
            im_data.instructions.instruction(Instruction::ldnull);
        },
        Term::Ident(ident) => {
            if ident == "world" {
                im_data.instructions.instruction(Instruction::ldsfld("object byond_root::world".to_owned()));
            } else {
                im_data.instructions.not_implemented("Identifier lookup not done yet.");
            }
        }
        _ => {
            im_data.instructions.not_implemented("Unable to handle term.");
        }
    }
}

fn get_proc_body_details<'a>(procdef: &TypeProc, state: &'a DMState) -> Option<&'a[Statement]> {
    for anno in state.get_annotations(procdef.value.location) {
        if let (range, Annotation::ProcHeader(_)) = anno {
            let mut end = range.end;
            end.column += 1;
            for anno in state.get_annotations(end) {
                if let (_, Annotation::ProcBodyDetails(code)) = anno {
                    println!("{:?}", anno);
                    return Some(code);
                }
            }
        }
    }

    None
}
