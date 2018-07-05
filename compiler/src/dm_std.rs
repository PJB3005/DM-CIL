use super::il::*;

pub fn create_world_class(parent: &mut Class) {
    let mut class = Class::new("world".to_owned(), ClassAccessibility::NestedPublic, Some("byond_root".to_owned()), "byond_root/world".to_owned(), false);
    
    let mut ctor_code: InstructionBlob = InstructionBlob::default();
    ctor_code.instruction(Instruction::ldarg0);
    ctor_code.instruction(Instruction::call("instance void [mscorlib]System.Object::.ctor()".to_owned()));
    ctor_code.instruction(Instruction::ret);

    let mut ctor = Method::new(".ctor".to_owned(), "void".to_owned(), MethodAccessibility::Public, MethodVirtuality::NotVirtual, ctor_code, false);
    ctor.is_rt_special_name = true;
    ctor.is_special_name = true;
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

    cctor
}