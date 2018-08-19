use super::il::*;

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

pub fn create_std_proc(name: &str) -> Method {
    let mut method = Method::new(name.to_owned(), "object".to_owned(), MethodAccessibility::Public, MethodVirtuality::NotVirtual, InstructionBlob::default(), true);

    match name {
        "abs" => {
            method.code.instruction(Instruction::ldarg0);
            method.code.instruction(Instruction::unboxany("[mscorlib]System.Single".to_owned()));
            method.code.instruction(Instruction::call("float32 [mscorlib]System.Math::Abs(float32)".to_owned()));
            method.code.instruction(Instruction::_box("[mscorlib]System.Single".to_owned()));
            method.code.instruction(Instruction::ret);

            method.params.push(MethodParameter::new("A", "object"));
            method.maxstack = 1;
        },
        _ => {
            method.code.not_implemented("std proc not implemented.");
        }
    };

    method
}