use super::il::*;
use super::compiler_state::*;

pub fn create_world_class(parent: &mut Class) {
    let mut class = Class::new("world".to_owned(), ClassAccessibility::NestedPublic, Some("byond_root".to_owned()), "byond_root/world".to_owned(), false);
    
    let ctor = create_stock_ctor("byond_root");
    class.insert_method(ctor);

    let mut output_code: InstructionBlob = InstructionBlob::default();
    output_code.instruction(Instruction::ldarg1);
    output_code.instruction(Instruction::call("void [mscorlib]System.Console::WriteLine(object)".to_owned()));
    output_code.instruction(Instruction::ret);

    let mut output = Method::new("output".to_owned(), "void".to_owned(), MethodAccessibility::Public, MethodVirtuality::NotVirtual, output_code, false);
    output.params.push(MethodParameter {custom_attributes: vec![], name: "obj".to_owned(), type_name: "object".to_owned() });
    class.insert_method(output);

    parent.insert_child_class(class);
}

pub fn create_global_cctor() -> Method {
    let mut code = InstructionBlob::default();
    code.instruction(Instruction::newobj("instance void byond_root/world::'.ctor' ()".to_owned()));
    code.instruction(Instruction::stsfld("object byond_root::world".to_owned()));
    code.instruction(Instruction::ret);
    let mut cctor = Method::new(".cctor".to_owned(), "void".to_owned(), MethodAccessibility::Public, MethodVirtuality::NotVirtual, code, true);
    cctor.is_rt_special_name = true;
    cctor.is_special_name = true;
    cctor.maxstack = 1;

    cctor
}

pub fn create_stock_ctor(parent_name: &str) -> Method {
    let mut code = InstructionBlob::default();
    code.instruction(Instruction::ldarg0);
    code.instruction(Instruction::call(format!("instance void {}::.ctor()", parent_name)));
    code.instruction(Instruction::ret);

    let mut ctor = Method::new(".ctor".to_owned(), "void".to_owned(), MethodAccessibility::Public, MethodVirtuality::NotVirtual, code, false);
    ctor.is_rt_special_name = true;
    ctor.is_special_name = true;
    ctor.maxstack = 1;

    ctor
}

pub fn create_std_proc(std_proc: &StdProc) -> Method {
    match std_proc {
        StdProc::Abs => {
            let mut method = Method::new("abs".into(), "object".to_owned(), MethodAccessibility::Public, MethodVirtuality::NotVirtual, InstructionBlob::default(), true);
            method.code.instruction(Instruction::ldarg0);
            method.code.instruction(Instruction::unboxany("[mscorlib]System.Single".to_owned()));
            method.code.instruction(Instruction::call("float32 [mscorlib]System.Math::Abs(float32)".to_owned()));
            method.code.instruction(Instruction::_box("[mscorlib]System.Single".to_owned()));
            method.code.instruction(Instruction::ret);

            method.params.push(MethodParameter::new("A", "object"));
            method.maxstack = 1;
            method
        },
        StdProc::WorldOutput => {
            panic!()
        }, /*
        "min" => {
            method.code.instruction(Instruction::ldarg0);
            method.code.instruction(Instruction::unboxany("[mscorlib]System.Single".to_owned()));
            method.code.instruction(Instruction::ldarg1);
            method.code.instruction(Instruction::unboxany("[mscorlib]System.Single".to_owned()));
            method.code.instruction(Instruction::call("float32 [mscorlib]System.Math::Min(float32, float32)".to_owned()));
            method.code.instruction(Instruction::_box("[mscorlib]System.Single".to_owned()));
            method.code.instruction(Instruction::ret);

            method.params.push(MethodParameter::new("A", "object"));
            method.params.push(MethodParameter::new("B", "object"));
            method.maxstack = 2;
        },
        "max" => {
            method.code.instruction(Instruction::ldarg0);
            method.code.instruction(Instruction::unboxany("[mscorlib]System.Single".to_owned()));
            method.code.instruction(Instruction::ldarg1);
            method.code.instruction(Instruction::unboxany("[mscorlib]System.Single".to_owned()));
            method.code.instruction(Instruction::call("float32 [mscorlib]System.Math::Max(float32, float32)".to_owned()));
            method.code.instruction(Instruction::_box("[mscorlib]System.Single".to_owned()));
            method.code.instruction(Instruction::ret);

            method.params.push(MethodParameter::new("A", "object"));
            method.params.push(MethodParameter::new("B", "object"));
            method.maxstack = 2;
        },*/
        StdProc::Unimplemented(name) => {
            let mut method = Method::new(name.clone(), "object".to_owned(), MethodAccessibility::Public, MethodVirtuality::NotVirtual, InstructionBlob::default(), true);
            method.code.not_implemented("std proc not implemented.");
            method
        }
    }
}

pub fn create_std(state: &mut CompilerState) {
    // Create global procs.
    {
        let mut proc_abs = Proc::new("abs", ProcSource::Std(StdProc::Abs));
        proc_abs.parameters.push(ProcParameter::new("A", VariableType::Unspecified));
        state.global_procs.insert(proc_abs.name.clone(), proc_abs);
    }

    {
        let mut proc_min = Proc::new("min", ProcSource::Std(StdProc::Abs));
        proc_min.var_arg = true;
        state.global_procs.insert(proc_min.name.clone(), proc_min);
    }

    {
        let mut proc_max = Proc::new("max", ProcSource::Std(StdProc::Abs));
        proc_max.var_arg = true;
        state.global_procs.insert(proc_max.name.clone(), proc_max);
    }

    // Create world.
    {
        let world_path = "/world".into();
        let mut world_type = CompilerType::new(&world_path);
        
        let mut output_proc = Proc::new("output", ProcSource::Std(StdProc::WorldOutput));
        output_proc.parameters.push(ProcParameter::new("O", VariableType::Unspecified));
        world_type.procs.insert("output".into(), output_proc);

        state.types.insert(world_path.clone(), world_type);

        let world_var = GlobalVar { name: "world".into(), var_type: VariableType::Object(world_path)};
        state.global_vars.insert("world".into(), world_var);
    }
}