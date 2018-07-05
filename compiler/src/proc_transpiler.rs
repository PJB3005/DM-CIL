//! The workhorse of the compiler.
//! Compiles the a BYOND AST proc into a CIL method.
use std::collections::HashMap;
use dm::ast::*;
use dm::objtree::TypeProc;
use dm::annotation::Annotation;
use super::il::*;
use super::dmstate::DMState;

pub(crate) fn create_proc(procdef: &TypeProc, class: &mut Class, proc_name: &str, is_static: bool, state: &DMState) -> Method {
    println!("{}: {:?}", proc_name, procdef);
    
    let is_entry_point = proc_name == "EntryPoint";
    let return_type = if is_entry_point {
        "void".to_owned()
    } else {
        "object".to_owned()
    };

    if let Some(code) = get_proc_body_details(procdef, state) {
        let mut data = TranspilerData {
            total_locals: 1,
            locals: vec![HashMap::new()],
            uniques: 0,
            state,
            class,
            proc_name,
            is_static,
        };
        let mut ins = InstructionBlob::default();
        // Load null into . (default return value.)
        ins.instruction(Instruction::ldnull);
        ins.instruction(Instruction::stloc0);
        for statement in code {
            write_statement(statement, &mut data, &mut ins)
        }

        if !is_entry_point {
            ins.instruction(Instruction::ldloc0);
        }
        ins.instruction(Instruction::ret);

        let mut method = Method::new(proc_name.to_owned(), return_type, MethodAccessibility::Public, MethodVirtuality::NotVirtual, ins, is_static);

        for _ in 0..data.total_locals {
            method.locals.push("object".to_owned());
        }

        return method;
    }

    let mut blob = InstructionBlob::default();
    blob.not_implemented("Unable to find proc body.");

    Method::new(proc_name.to_owned(), return_type, MethodAccessibility::Public, MethodVirtuality::NotVirtual, blob, is_static)
}

/// Shared data necessary across the entire proc transpile.
struct TranspilerData<'a> {
    pub total_locals: u16,
    pub locals: Vec<HashMap<String, u16>>,
    pub uniques: u16,
    pub state: &'a DMState,
    pub class: &'a mut Class,
    pub proc_name: &'a str,
    pub is_static: bool,
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

    pub fn push_scope(&mut self) {
        self.locals.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        // TODO: Recycle locals.
        self.locals.pop();
    }
}

fn write_statement(statement: &Statement, data: &mut TranspilerData, ins: &mut InstructionBlob) {
    // RULE: when this function is done, the stack is the same as before.
    match statement {
        Statement::Expr(exp) => {
            evaluate_expression(exp, true, data, ins);
            // Pop the expression off again. Don't need it.
            ins.instruction(Instruction::pop);
        },
        Statement::Var { name, value, .. } => {
            let idx = data.add_local(name);
            if let Some(initializer) = value {
                evaluate_expression(initializer, false, data, ins);
                ins.instruction(Instruction::stloc(idx));
            }
        },
        Statement::Setting(name, SettingMode::Assign, exp) => {
            if let Some(idx) = data.get_local(name) {
                evaluate_expression(exp, false, data, ins);
                ins.instruction(Instruction::stloc(idx));
            }
        }
        _ => {
            ins.not_implemented("Unknown Statement.");
        }
    }
}

fn evaluate_expression(expression: &Expression, will_be_discarded: bool, data: &mut TranspilerData, ins: &mut InstructionBlob) {
    // RULE: when this function is done, there is ONE extra value on the stack.
    match expression {
        Expression::Base { unary, term, follow } => {
            let mut term_blob = InstructionBlob::default();
            evaluate_term(term, data, &mut term_blob);
            for follow in follow.iter().rev() {
                let old_blob = term_blob;
                term_blob = InstructionBlob::default();
                evaluate_follow(follow, will_be_discarded, old_blob, data, &mut term_blob);
            };

            for unary in unary.iter().rev() {
                // TODO: Unary ops.
            }

            ins.absord(term_blob);
            if will_be_discarded {
                ins.instruction(Instruction::ldnull);
            }
        },
        Expression::BinaryOp { op, lhs, rhs } => {
            let mut arg_blob = InstructionBlob::default();
            evaluate_expression(lhs, false, data, &mut arg_blob);
            evaluate_expression(rhs, false, data, &mut arg_blob);
            do_dynamic_invoke(DynamicInvokeType::BinaryOp(*op), arg_blob, data, ins);
        },
        _ => {
            ins.not_implemented("Unable to handle expression type.");
        }
    }
}

fn evaluate_term(term: &Term, data: &mut TranspilerData, ins: &mut InstructionBlob) {
    // RULE: when this function is done, there is ONE extra value on the stack.
    match term {
        Term::Int(val) => {
            ins.instruction(Instruction::ldcr4(*val as f32));
            ins.instruction(Instruction::_box("[mscorlib]System.Single".to_owned()));
        }
        Term::Float(val) => {
            ins.instruction(Instruction::ldcr4(*val));
            ins.instruction(Instruction::_box("[mscorlib]System.Single".to_owned()));
        },
        Term::Null => {
            ins.instruction(Instruction::ldnull);
        },
        Term::Ident(ident) => {
            if ident == "world" {
                ins.instruction(Instruction::ldsfld("object byond_root::world".to_owned()));
            } else if ident == "src" {
                ins.instruction(Instruction::ldarg0);
            } else if let Some(idx) = data.get_local(ident) {
                ins.instruction(Instruction::ldloc(idx));
            } else {
                ins.not_implemented("Unknown identifier.");
            }
        },
        Term::String(val) => {
            ins.instruction(Instruction::ldstr(val.to_owned()));
        },
        Term::Expr(expr) => {
            evaluate_expression(expr, false, data, ins);
        },
        Term::ReturnValue => {
            ins.instruction(Instruction::ldloc0);
        }
        _ => {
            ins.not_implemented("Unable to handle term.");
        }
    }
}

fn evaluate_follow(follow: &Follow, will_be_discarded: bool, mut term_blob: InstructionBlob, data: &mut TranspilerData, ins: &mut InstructionBlob) {
    // RULE: term_callback will insert ONE element into the stack, which is the object to call on.
    // When this function is done, there is an extra value on the stack IF !will_be_discarded 
    match follow {
        Follow::Call(_, method_name, args) => {
            for arg in args {
                evaluate_expression(arg, false, data, &mut term_blob)
            }

            do_dynamic_invoke(DynamicInvokeType::MemberInvoke {
                arg_count: args.len() as u16,
                expect_return: !will_be_discarded,
                method_name: method_name.clone()
                }, term_blob, data, ins);
        },
        _ => {
            ins.not_implemented("Non-call follows not implemented.");
        }
    }
}

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
        DynamicInvokeType::BinaryOp(BinaryOp::Add) => {
            ins.instruction(Instruction::ldci40);
        },
        DynamicInvokeType::BinaryOp(BinaryOp::Sub) => {
            ins.instruction(Instruction::ldci4(42));
        },
        DynamicInvokeType::BinaryOp(BinaryOp::Mul) => {
            ins.instruction(Instruction::ldci4(26));
        },
        DynamicInvokeType::BinaryOp(BinaryOp::Div) => {
            ins.instruction(Instruction::ldci4(12));
        },

        ref a => {
            println!("{:?}", a);
            ins.not_implemented("Unimplemented invoke type");
        }
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