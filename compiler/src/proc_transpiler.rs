//! The workhorse of the compiler.
//! Compiles the a BYOND AST proc into a CIL method.
use CompilerError;
use std::collections::HashMap;
use dm::ast::*;
use dm::annotation::Annotation;
use super::il::*;
use super::dmstate::DMState;
use super::compiler_state::*;

pub(crate) fn create_proc(the_proc: &Proc, class: &mut Class, proc_name: &str, is_static: bool,
                          state: &DMState, compiler_state: &CompilerState) -> Result<Method, CompilerError> {
    if let Some(code) = get_proc_body_details(the_proc, state) {
        let mut data = TranspilerData {
            total_locals: 1,
            locals: vec![HashMap::new()],
            uniques: 0,
            state,
            compiler_state,
            class,
            proc_name,
            is_static,
            loop_labels: vec![],
        };

        let mut ins = InstructionBlob::default();

        // Load up arguments into locals.
        // Not efficient but it makes the code simpler.
        for (i, param) in the_proc.parameters.iter().enumerate() {
            let local = data.add_local(&param.name);
            let arg = if is_static { i } else { i + 1 } as u16;
            ins.instruction(Instruction::ldarg(arg));
            ins.instruction(Instruction::stloc(local));
        }
        
        // Load null into . (default return value.)
        ins.instruction(Instruction::ldnull);
        ins.instruction(Instruction::stloc0);
        for statement in code {
            write_statement(statement, &mut data, &mut ins)?;
        }

        ins.instruction(Instruction::ret);

        let mut method = Method::new(proc_name.to_owned(), "object".into(), MethodAccessibility::Public, MethodVirtuality::NotVirtual, ins, is_static);

        for param in &the_proc.parameters {
            method.params.push(MethodParameter::new(&param.name, "object"));
        }

        for _ in 0..data.total_locals {
            method.locals.push("object".to_owned());
        }

        Ok(method)
    } else {
        Err(format!("Unable to find proc body: {}, {:?}", proc_name, the_proc).into())
    }
}

pub(crate) fn evaluate_initializer(expression: &Expression, class: &mut Class, proc_name: &str, dm_state: &DMState, compiler_state: &CompilerState, blob: &mut InstructionBlob) -> Result<VariableType, CompilerError> {
    let mut data = TranspilerData {
        total_locals: 0,
        locals: vec![],
        uniques: 0,
        state: dm_state,
        compiler_state,
        is_static: true,
        proc_name,
        class,
        loop_labels: vec![]
    };

    evaluate_expression(expression, false, &mut data, blob)
}

/// Shared data necessary across the entire proc transpile.
struct TranspilerData<'a> {
    pub total_locals: u16,
    pub locals: Vec<HashMap<String, u16>>,
    pub uniques: u16,
    pub state: &'a DMState,
    pub compiler_state: &'a CompilerState,
    pub class: &'a mut Class,
    pub proc_name: &'a str,
    pub is_static: bool,
    pub loop_labels: Vec<(String, String)>,
}

impl<'a> TranspilerData<'a> {
    pub fn get_meta_class(&mut self) -> &mut Class {
        let name = format!("<>_meta_{}", self.proc_name);
        if !self.class.has_child_class(&name) {
            let class = Class::new(name.clone(),
                                   ClassAccessibility::NestedPrivate,
                                   None,
                                   format!("{}/'{}'", self.class.get_full_name(), name),
                                   true);
            self.class.insert_child_class(class);
        }

        self.class.get_child_class_mut(&name).unwrap()
    }

    pub fn get_meta_field_name(&mut self) -> String {
        let name = format!("<>_{}", self.get_uniq());
        self.uniques += 1;
        name
    }

    pub fn get_uniq(&mut self) -> u16 {
        let val = self.uniques;
        self.uniques += 1;
        val
    }

    /// Adds a local variable with specified name to this scope.
    pub fn add_local(&mut self, name: &str) -> u16 {
        // NOTE: Local 0 is . (default return value).
        let top_pos = self.locals.len()-1;
        let top = &mut self.locals[top_pos];
        let new_local_id = self.total_locals;
        self.total_locals += 1;
        top.insert(name.to_owned(), new_local_id);
        new_local_id
    }

    /// Gets the ID of a local variable.
    pub fn get_local(&self, name: &str) -> Option<u16> {
        for locals in self.locals.iter().rev() {
            if let Some(id) = locals.get(name) {
                return Some(*id);
            }
        }

        None
    }

    #[allow(dead_code)]
    pub fn add_unnamed_local(&mut self) -> u16 {
        let new_local_id = self.total_locals;
        self.total_locals += 1;
        new_local_id
    }

    pub fn push_loop_scope(&mut self, repeat_label: String, exit_label: String) {
        self.push_scope();
        self.loop_labels.push((repeat_label, exit_label));
    }

    pub fn push_scope(&mut self) {
        self.locals.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        // TODO: Recycle locals.
        self.locals.pop();
    }

    pub fn pop_loop_scope(&mut self) {
        self.pop_scope();
        self.loop_labels.pop();
    }

    pub fn get_loop_exit_label(&self) -> Option<&str> {
        if self.loop_labels.len() == 0 {
            return None;
        }

        Some(&self.loop_labels[self.loop_labels.len()-1].1)
    }

    pub fn get_loop_repeat_label(&self) -> Option<&str> {
        if self.loop_labels.len() == 0 {
            return None;
        }

        Some(&self.loop_labels[self.loop_labels.len()-1].0)
    }
}

fn write_statement(statement: &Statement, data: &mut TranspilerData, ins: &mut InstructionBlob) -> Result<(), CompilerError> {
    // RULE: when this function is done, the stack is the same as before.
    match statement {
        Statement::Expr(exp) => {
            evaluate_expression(exp, true, data, ins)?;
        },
        Statement::Var(VarStatement { name, value, .. }) => {
            let idx = data.add_local(name);
            if let Some(initializer) = value {
                evaluate_expression(initializer, false, data, ins)?;
                ins.instruction(Instruction::stloc(idx));
            }
        },
        Statement::If(ifs, else_statements) => {
            let uniq = data.get_uniq();
            // The label AFTER the else clause.
            let end_label = format!("ip_{}", uniq);
            let else_label = format!("ie_{}", uniq);

            let ifcount = ifs.len();
            for (i, (expr, statements)) in ifs.iter().enumerate() {
                ins.label(format!("ic_{}_{}", uniq, i));
                // I should probably write this down *somewhere*.
                // I put a nop after most labels so that something if like evaluate_expression ALSO writes a label,
                // because 2 labels on the same opcode would break.
                ins.instruction(Instruction::nop);
                evaluate_expression(expr, false, data, ins)?;
                evaluate_truthy(ins);
                // There is another else if.
                if i != ifcount - 1 {
                    ins.instruction(Instruction::brfalse(format!("ic_{}_{}", uniq, i+1)));
                } else {
                    if else_statements.is_some() {
                        ins.instruction(Instruction::brfalse(else_label.clone()));
                    } else {
                        ins.instruction(Instruction::brfalse(end_label.clone()))
                    }
                }

                for statement in statements {
                    write_statement(statement, data, ins)?;
                }

                ins.instruction(Instruction::br(end_label.clone()));
            }

            if let Some(statements) = else_statements {
                ins.label(else_label);
                ins.instruction(Instruction::nop);
                
                for statement in statements {
                    write_statement(statement, data, ins)?;
                }
            }

            ins.label(end_label);
            ins.instruction(Instruction::nop);
        },
        Statement::While(exp, statements) => {
            let uniq = data.get_uniq();
            let test_label = format!("w_{}", uniq);
            let exit_label = format!("e_{}", uniq);
            data.push_loop_scope(test_label.clone(), exit_label.clone());

            ins.label(test_label.clone());
            println!("{:?}", &exp);
            evaluate_expression(exp, false, data, ins)?;
            evaluate_truthy(ins);
            ins.instruction(Instruction::brfalse(exit_label.clone()));

            for statement in statements {
                write_statement(statement, data, ins)?;
            }

            ins.instruction(Instruction::br(test_label));

            ins.label(exit_label);
            ins.instruction(Instruction::nop);

            data.pop_loop_scope();
        },
        Statement::DoWhile(statements, exp) => {
            let uniq = data.get_uniq();
            let test_label = format!("dw_{}", uniq);
            let exit_label = format!("de_{}", uniq);
            let repeat_label = format!("dr_{}", uniq);
            data.push_loop_scope(test_label.clone(), exit_label.clone());

            ins.label(repeat_label.clone());

            for statement in statements {
                write_statement(statement, data, ins)?;
            }

            ins.label(test_label);
            ins.instruction(Instruction::nop);
            evaluate_expression(exp, false, data, ins)?;
            ins.instruction(Instruction::call("bool class [DM]DM.DmInternal::Truthy(object)".to_owned()));
            ins.instruction(Instruction::brtrue(repeat_label));
            ins.label(exit_label);
            ins.instruction(Instruction::nop);
        },
        Statement::Break(Some(_)) | Statement::Continue(Some(_)) => {
            return Err("Labelled loop flow control is not implemented yet.".into());
        },
        Statement::Break(None) => {
            if let Some(label) = data.get_loop_exit_label() {
                ins.instruction(Instruction::br(label.to_owned()));
            }
            else
            {
                return Err("Encountered break outside loop".into());
            }
        },
        Statement::Continue(None) => {
            if let Some(label) = data.get_loop_repeat_label() {
                ins.instruction(Instruction::br(label.to_owned()));
            }
            else
            {
                return Err("Encountered break outside loop".into());
            }
        },
        Statement::Return(None) => {
            ins.instruction(Instruction::ldnull);
            ins.instruction(Instruction::ret);
        },
        Statement::Return(Some(expr)) => {
            evaluate_expression(&expr, false, data, ins)?;
            ins.instruction(Instruction::ret);
        }
        _ => {
            return Err(format!("unknown statement: {:?}", statement).into());
        }
    };

    Ok(())
}

fn evaluate_expression(expression: &Expression, will_be_discarded: bool, data: &mut TranspilerData, ins: &mut InstructionBlob) -> Result<VariableType, CompilerError> {
    // RULE: when this function is done, ONE extra value on the stack ONLY if will_be_discarded is false.
    match expression {
        Expression::Base { unary, term, follow } => {
            let mut term_blob = InstructionBlob::default();
            let mut term_type = evaluate_term(term, data, &mut term_blob)?;
            for follow in follow.iter().rev() {
                let old_blob = term_blob;
                term_blob = InstructionBlob::default();
                term_type = evaluate_follow(follow, will_be_discarded, term_type, old_blob, data, &mut term_blob)?;
            };

            for _unary in unary.iter().rev() {
                // TODO: Unary ops.
            }

            ins.absord(term_blob);
            if will_be_discarded {
                ins.instruction(Instruction::pop);
            }

            return Ok(term_type);
        },
        Expression::BinaryOp { op, lhs, rhs } => {
            match op {
                BinaryOp::Add
                | BinaryOp::Mul
                | BinaryOp::Sub
                | BinaryOp::Div
                | BinaryOp::Eq
                | BinaryOp::NotEq
                | BinaryOp::Greater
                | BinaryOp::GreaterEq 
                | BinaryOp::Less
                | BinaryOp::LessEq
                | BinaryOp::Mod => {
                    let mut arg_blob = InstructionBlob::default();
                    evaluate_expression(lhs, false, data, &mut arg_blob)?;
                    evaluate_expression(rhs, false, data, &mut arg_blob)?;
                    do_dynamic_invoke(DynamicInvokeType::BinaryOp(*op), arg_blob, data, ins);
                    if will_be_discarded {
                        ins.instruction(Instruction::pop);
                    } else {
                        match op {
                            BinaryOp::Eq
                            | BinaryOp::NotEq
                            | BinaryOp::Greater
                            | BinaryOp::GreaterEq 
                            | BinaryOp::Less
                            | BinaryOp::LessEq => {
                                ins.instruction(Instruction::unboxany("[mscorlib]System.Boolean".to_owned()));
                                bool_to_float(data, ins)
                            },
                            _ => {}
                        };
                    };
                },
                BinaryOp::And => {
                    let uniq = data.get_uniq();
                    let exit = format!("opand_lhs_false_{}", uniq);
                    evaluate_expression(lhs, false, data, ins)?;
                    ins.instruction(Instruction::dup);
                    evaluate_truthy(ins);
                    ins.instruction(Instruction::brfalse(exit.clone()));
                    ins.instruction(Instruction::pop);

                    evaluate_expression(rhs, false, data, ins)?;
                    ins.label(exit);
                    ins.instruction(Instruction::nop);
                    if will_be_discarded {
                        ins.instruction(Instruction::pop);
                    }
                },
                BinaryOp::Or => {
                    let uniq = data.get_uniq();
                    let exit = format!("opor_lhs_true_{}", uniq);
                    evaluate_expression(lhs, false, data, ins)?;
                    ins.instruction(Instruction::dup);
                    evaluate_truthy(ins);
                    ins.instruction(Instruction::brtrue(exit.clone()));
                    ins.instruction(Instruction::pop);

                    evaluate_expression(rhs, false, data, ins)?;
                    ins.label(exit);
                    ins.instruction(Instruction::nop);
                    if will_be_discarded {
                        ins.instruction(Instruction::pop);
                    }
                },
                BinaryOp::LShift => {
                    let mut arg_blob = InstructionBlob::default();
                    evaluate_expression(lhs, false, data, &mut arg_blob)?;
                    evaluate_expression(rhs, false, data, &mut arg_blob)?;
                    let invoke = DynamicInvokeType::MemberInvoke {
                        arg_count: 1,
                        expect_return: false,
                        method_name: "output".to_owned()
                    };
                    do_dynamic_invoke(invoke, arg_blob, data, ins);
                },
                _ => {
                    return Err(format!("Unknown op: {:?}", op).into());
                },
            };
        },
        Expression::AssignOp { op: AssignOp::Assign, lhs, rhs } => {
            if let Expression::Base { term: Term::Ident(varname), .. } = *lhs.clone() {
                if let Some(idx) = data.get_local(&varname) {
                    evaluate_expression(rhs, false, data, ins)?;
                    if !will_be_discarded {
                        ins.instruction(Instruction::dup);
                    }
                    ins.instruction(Instruction::stloc(idx));
                } else {
                    return Err(format!("Unknown variable: {}", &varname).into());
                }
            } else {
                return Err("That lvalue is too complex for me.".into());
            }
        },
        _ => {
            return Err(format!("Unable to handle expression type: {:?}", expression).into());
        }
    };

    Ok(VariableType::Unspecified)
}

fn evaluate_term(term: &Term, data: &mut TranspilerData, ins: &mut InstructionBlob) -> Result<VariableType, CompilerError> {
    // RULE: when this function is done, there is ONE extra value on the stack.
    match term {
        Term::Int(val) => {
            ins.instruction(Instruction::ldcr4(*val as f32));
            ins.instruction(Instruction::_box("[mscorlib]System.Single".to_owned()));
            Ok(VariableType::Unspecified)
        }
        Term::Float(val) => {
            ins.instruction(Instruction::ldcr4(*val));
            ins.instruction(Instruction::_box("[mscorlib]System.Single".to_owned()));
            Ok(VariableType::Unspecified)
        }
        Term::Null => {
            ins.instruction(Instruction::ldnull);
            Ok(VariableType::Unspecified)
        }
        Term::Ident(ident) => {
            if ident == "src" {
                ins.instruction(Instruction::ldarg0);
                Ok(VariableType::Unspecified)
            } else if let Some(idx) = data.get_local(ident) {
                ins.instruction(Instruction::ldloc(idx));
                Ok(VariableType::Unspecified)
            } else if data.compiler_state.global_vars.contains_key(ident) {
                let global = data.compiler_state.global_vars.get(ident).unwrap();
                ins.instruction(Instruction::ldsfld(format!("object byond_root::{}", ident)));
                match &global.var_type {
                    VariableType::Object(path) => {
                        ins.instruction(Instruction::castclass(byond_path_to_class(path)));
                    },
                    VariableType::Unspecified => {}
                };
                Ok(global.var_type.clone())
            } else {
                Err(format!("Unknown identifier: {}", &ident).into())
            }
        },
        Term::String(val) => {
            ins.instruction(Instruction::ldstr(val.to_owned()));
            Ok(VariableType::Unspecified)
        },
        Term::Expr(expr) => {
            evaluate_expression(expr, false, data, ins)
        },
        Term::ReturnValue => {
            ins.instruction(Instruction::ldloc0);
            Ok(VariableType::Unspecified)
        },
        Term::Call(name, args) => {
            if !data.is_static {
                return Err("Unscoped non-static calls are not implemented yet.".into());
            }
            let tree = data.state.get_tree();
            let root = tree.root();
            if let Some(proc) = root.get_proc(name) {
                let mut args_tok = String::new();
                if proc.parameters.len() != 0 {
                    args_tok.push_str("object");
                    for _ in 1..proc.parameters.len() {
                        args_tok.push_str(", object");
                    }
                }
                //println!("{:?}", proc);
                for expr in args {
                    evaluate_expression(expr, false, data, ins)?;
                }
                ins.instruction(Instruction::call(format!("object byond_root::{}({})", name, args_tok)));
                Ok(VariableType::Unspecified)
            } else {
                panic!(format!("Method does not exist: {}", name));
            }
        },
        t => {
            Err(format!("Unable to handle term: {:?}", t).into())
        }
    }
}

fn evaluate_follow(follow: &Follow, will_be_discarded: bool, term_type: VariableType, mut term_blob: InstructionBlob, data: &mut TranspilerData, ins: &mut InstructionBlob) -> Result<VariableType, CompilerError> {
    // When this function is done, there is an extra value on the stack IF !will_be_discarded 
    match follow {
        Follow::Call(_, method_name, args) => {
            for arg in args {
                evaluate_expression(arg, false, data, &mut term_blob)?;
            }

            match term_type {
                VariableType::Unspecified => {
                    do_dynamic_invoke(DynamicInvokeType::MemberInvoke {
                        arg_count: args.len() as u16,
                        expect_return: !will_be_discarded,
                        method_name: method_name.clone()
                    }, term_blob, data, ins);
                    Ok(VariableType::Unspecified)
                },
                VariableType::Object(path) => {
                    if !data.compiler_state.types.contains_key(&path) {
                        return Err("Unable to find type.".into());
                    }
                    let type_instance = &data.compiler_state.types[&path];
                    if !type_instance.procs.contains_key(method_name) {
                        return Err("Unable to find proc.".into());
                    }
                    
                    //let instance_proc = &type_instance.procs[method_name];
                    // oh shit we got it.
                    ins.absord(term_blob);
                    let arg_count = args.len();
                    let mut args = String::new();
                    if arg_count > 0 {
                        args.push_str("object");

                        if arg_count > 1 {
                            for _ in 1..arg_count {
                                args.push_str(", object");
                            }
                        }
                    }
                    ins.instruction(Instruction::call(format!("instance object byond_root{}::{}({})", path, method_name, args)));

                    Ok(VariableType::Unspecified)
                }
            }
        },
        a => {
            Err(format!("Non-call follows not implemented: {:?}", a).into())
        }
    }
}

// NOTE FROM THE PAST BUT RELATIVE TO THE BELOW THE FUTURE:
// This is no longer true.
// We have static typing now.

// ALRIGHT.
// So because DM has awful typing support AND I'm too lazy to implement type checking atm,
// everything is duck typed.
// So, we need to use C# dynamic. Dynamic does not exist at a CIL level.
// This monster of a method generates dynamic operations for everything we need.
// I recommend you to mess around with dynamic on sharplab.io to have the slightest of a grasp what's going on.
fn do_dynamic_invoke(invoke_type: DynamicInvokeType, subblob: InstructionBlob, data: &mut TranspilerData, ins: &mut InstructionBlob) {
    // RULE: when this function is done, there is an extra value on the stack IF the operation should've added one.
    // So basically it depends on what kinda operation's being invoked.

    let meta_field_name = data.get_meta_field_name();
    let post_init_label = format!("di_{}", data.get_uniq());
    let (call_type, arg_count, call_site_calltype, meta_field_name_full) = {
        // NLL when.
        let meta_class = data.get_meta_class();

        // call_type is like class [mscorlib]System.Action`3<class [System.Core]System.Runtime.CompilerServices.CallSite, object, object>
        // I'm gonna be honest, I'm pretty sure arg_count is off by one (hell, 2?).
        let (call_type, arg_count) = match &invoke_type {
            DynamicInvokeType::MemberInvoke { arg_count, expect_return, .. } => {
                let mut type_args_count = 1;
                let mut type_args = "class [System.Core]System.Runtime.CompilerServices.CallSite".to_owned();
                for _ in 0..=*arg_count {
                    type_args.push_str(", object");
                    type_args_count += 1;
                }

                if *expect_return {
                    type_args.push_str(", object");
                    type_args_count += 1;
                }

                (format!("class [mscorlib]System.{}`{}<{}>",
                        if *expect_return { "Func" } else { "Action" },
                        type_args_count, type_args), *arg_count)
            },
            DynamicInvokeType::BinaryOp(_) => {
                ("class [mscorlib]System.Func`4<class [System.Core]System.Runtime.CompilerServices.CallSite, object, object, object>".to_owned(), 1)
            }
        };
        // Callsite`1<call_type> type, because it's used a lot.
        let call_site_calltype = format!("class [System.Core]System.Runtime.CompilerServices.CallSite`1<{}>", call_type); 
        meta_class.insert_field(Field {
            name: meta_field_name.clone(),
            type_name: call_site_calltype.clone(),
            accessibility: FieldAccessibility::Public,
            is_static: true,
            is_initonly: false,
        });
        let meta_field_name_full = format!("{} {}::'{}'", call_site_calltype, meta_class.get_full_name(), meta_field_name);
        (call_type, arg_count, call_site_calltype, meta_field_name_full)
    };
    // Check if the CallSite`1 is already initialized.
    // If so, skip to the normal execution code.
    ins.instruction(Instruction::ldsfld(meta_field_name_full.clone()));
    ins.instruction(Instruction::brtrue(post_init_label.clone()));
    
    // We're not initialized. Hold onto yer butts.
    // Push CSharpBinderFlags.
    if let DynamicInvokeType::MemberInvoke { expect_return: false, .. } = &invoke_type {
        // ResultDiscarded.
        ins.instruction(Instruction::ldci4(256));
    } else {
        // None.
        ins.instruction(Instruction::ldci40);
    }

    match invoke_type {
        DynamicInvokeType::MemberInvoke { ref method_name, .. } => {
            ins.instruction(Instruction::ldstr(method_name.clone()));
            // No generics.
            ins.instruction(Instruction::ldnull);
        },
        DynamicInvokeType::BinaryOp(op) => {
            ins.instruction(Instruction::ldci4(match op {
                // These correspond to System.Linq.Expressions.ExpressionType.
                BinaryOp::Add => 0,
                BinaryOp::Sub => 42,
                BinaryOp::Mul => 26,
                BinaryOp::Div => 12,
                BinaryOp::Eq => 13,
                BinaryOp::NotEq => 35,
                BinaryOp::Greater => 15,
                BinaryOp::GreaterEq => 16,
                BinaryOp::Less => 20,
                BinaryOp::LessEq => 21,
                BinaryOp::Mod => 25,
                _ => panic!("Unsupported binary op!"),
            }));
        },

        /*
        ref a => {
            println!("{:?}", a);
            ins.not_implemented("Unimplemented invoke type");
        }
        */
    };

    // Push System.Type.
    ins.instruction(Instruction::ldtoken(data.class.get_full_name().to_owned()));
    ins.instruction(Instruction::call("class [mscorlib]System.Type [mscorlib]System.Type::GetTypeFromHandle(valuetype [mscorlib]System.RuntimeTypeHandle)".to_owned()));
    
    // Create CSharpArgumentInfo array.
    ins.instruction(Instruction::ldci4((arg_count as i32)+1));
    ins.instruction(Instruction::newarr("[Microsoft.CSharp]Microsoft.CSharp.RuntimeBinder.CSharpArgumentInfo".to_owned()));

    for i in 0..=arg_count {
        ins.instruction(Instruction::dup);
        // Index in array.
        ins.instruction(Instruction::ldci4(i as i32));
        // Args for CSharpArgumentInfo creation.
        ins.instruction(Instruction::ldci40);
        ins.instruction(Instruction::ldnull);
        ins.instruction(Instruction::call("class [Microsoft.CSharp]Microsoft.CSharp.RuntimeBinder.CSharpArgumentInfo [Microsoft.CSharp]Microsoft.CSharp.RuntimeBinder.CSharpArgumentInfo::Create(valuetype [Microsoft.CSharp]Microsoft.CSharp.RuntimeBinder.CSharpArgumentInfoFlags, string)".to_owned()));
        // Set in array.
        ins.instruction(Instruction::stelemref);
    }

    match invoke_type {
        DynamicInvokeType::MemberInvoke { .. } => {
            // That's 500 columns long.
            ins.instruction(Instruction::call("class [System.Core]System.Runtime.CompilerServices.CallSiteBinder [Microsoft.CSharp]Microsoft.CSharp.RuntimeBinder.Binder::InvokeMember(valuetype [Microsoft.CSharp]Microsoft.CSharp.RuntimeBinder.CSharpBinderFlags, string, class [mscorlib]System.Collections.Generic.IEnumerable`1<class [mscorlib]System.Type>, class [mscorlib]System.Type, class [mscorlib]System.Collections.Generic.IEnumerable`1<class [Microsoft.CSharp]Microsoft.CSharp.RuntimeBinder.CSharpArgumentInfo>)".to_owned()))
        },
        DynamicInvokeType::BinaryOp(_) => {
            ins.instruction(Instruction::call("class [System.Core]System.Runtime.CompilerServices.CallSiteBinder [Microsoft.CSharp]Microsoft.CSharp.RuntimeBinder.Binder::BinaryOperation(valuetype [Microsoft.CSharp]Microsoft.CSharp.RuntimeBinder.CSharpBinderFlags, valuetype [System.Core]System.Linq.Expressions.ExpressionType, class [mscorlib]System.Type, class [mscorlib]System.Collections.Generic.IEnumerable`1<class [Microsoft.CSharp]Microsoft.CSharp.RuntimeBinder.CSharpArgumentInfo>)".to_owned()))
        }
    };

    // Create call site and assign it to the meta field.
    ins.instruction(Instruction::call(format!("class [System.Core]System.Runtime.CompilerServices.CallSite`1<!0> {}::Create(class [System.Core]System.Runtime.CompilerServices.CallSiteBinder)", call_site_calltype)));
    ins.instruction(Instruction::stsfld(meta_field_name_full.clone()));

    // Cool we're going to call it now.
    ins.label(post_init_label);
    ins.instruction(Instruction::ldsfld(meta_field_name_full.clone()));
    ins.instruction(Instruction::ldfld(format!("!0 {}::Target", call_site_calltype)));
    ins.instruction(Instruction::ldsfld(meta_field_name_full.clone()));
    
    // Load up arguments however the caller wants.
    ins.absord(subblob);
    
    match invoke_type {
        DynamicInvokeType::MemberInvoke { expect_return, arg_count, .. } => {
            let ret_type = if expect_return {
                format!("!{}", arg_count+1)
            } else {
                "void".to_owned()
            };

            let invoke_args = (0..=arg_count+1).map(|i| format!("!{}", i)).collect::<Vec<String>>().join(", ");
            ins.instruction(Instruction::callvirt(format!("instance {} {}::Invoke({})", ret_type, call_type, invoke_args)));
        }
        DynamicInvokeType::BinaryOp(_) => {
            ins.instruction(Instruction::callvirt(format!("instance !3 {}::Invoke(!0, !1, !2)", call_type)));
        }
    }
}

#[derive(Debug)]
enum DynamicInvokeType {
    MemberInvoke {
        arg_count: u16,
        expect_return: bool,
        method_name: String,
    },
    BinaryOp(BinaryOp),
}

fn get_proc_body_details<'a>(the_proc: &Proc, state: &'a DMState) -> Option<&'a[Statement]> {
    let loc = match the_proc.source {
        ProcSource::Code(loc) => loc,
        _ => panic!("Don't use proc body details on STD procs!"),
    };
    for anno in state.get_annotations(loc) {
        if let (range, Annotation::ProcHeader(..)) = anno {
            let mut end = range.end;
            end.column += 1;
            for anno in state.get_annotations(end) {
                if let (_, Annotation::ProcBodyDetails(code)) = anno {
                    //println!("{:?}", anno);
                    return Some(code);
                }
            }
        }
    }

    None
}

fn evaluate_truthy(ins: &mut InstructionBlob) {
    ins.instruction(Instruction::call("bool class [DM]DM.DmInternal::Truthy(object)".to_owned()));
}

/// Writes in a conversion from a bool to a float.
/// Because float is the numeric type in BYOND, but stuff such as equality returns bool.
fn bool_to_float(data: &mut TranspilerData, ins: &mut InstructionBlob) {
    let uniq = data.get_uniq();
    let true_label = format!("btf_{}_t", uniq);
    let escape_label = format!("btf_{}_e", uniq);
    // Effectively compiles "x ? 1f : 0f"
    ins.instruction(Instruction::brtrue(true_label.clone()));
    ins.instruction(Instruction::ldcr4(0f32));
    ins.instruction(Instruction::br(escape_label.clone()));
    ins.label(true_label);
    ins.instruction(Instruction::ldcr4(1f32));
    ins.label(escape_label);
    ins.instruction(Instruction::_box("[mscorlib]System.Single".to_owned()));
}

pub fn byond_path_to_class(path: &ByondPath) -> String {
    assert!(path.is_rooted());

    format!("byond_root{}", path)
}