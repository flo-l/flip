use ::value::Value;
use ::interpreter::Interpreter;

pub fn quote(_: &mut Interpreter, args: &mut [Value]) -> Value {
    check_arity!("quote", args.len(), 1);
    args[0].clone()
}

pub fn define(interpreter: &mut Interpreter, args: &mut [Value]) -> Value {
    check_arity!("define", args.len(), 2);

    let s = try_unwrap_type!("define", "symbol", Value::get_symbol, &args[0], interpreter);
    let item = interpreter.evaluate(&args[1]);
    interpreter.current_scope.add_symbol(s, item);
    args[0].clone()
}

pub fn if_(interpreter: &mut Interpreter, args: &mut [Value]) -> Value {
    check_arity!("if", args.len(), 3);

    let condition = interpreter.evaluate(&args[0]);
    if let Some(b) = condition.get_bool() {
        if b {
            interpreter.evaluate(&args[1])
        } else {
            interpreter.evaluate(&args[2])
        }
    } else {
        raise_condition!(format!("if: argument mismatch: expected bool, got: {}", condition.to_string(&interpreter.interner)));
    }
}

pub fn lambda(interpreter: &mut Interpreter, args: &mut [Value]) -> Value {
    check_arity!("lambda", args.len(), 2, 3);

    let name;
    let binding_list;
    let code;
    if args.len() == 2 {
        name = None;
        binding_list = try_unwrap_type!("lambda", "list", Value::get_list, &args[0], interpreter);
        code = args[1].clone();
    } else {
        // type check name
        let name_id = try_unwrap_type!("lambda", "list", Value::get_symbol, &args[0], interpreter);
        name = interpreter.interner.lookup(name_id).map(Into::into);
        binding_list = try_unwrap_type!("lambda", "list", Value::get_list, &args[1], interpreter);
        code = args[2].clone();
    }

    let mut bindings: Vec<u64> = Vec::with_capacity(binding_list.len());
    for v in binding_list.iter() {
        // type check
        bindings.push(try_unwrap_type!("lambda", "symbol", Value::get_symbol, v, interpreter));
    }

    Value::new_proc(name, interpreter.current_scope.clone(), bindings, code)
}

// This behaves like let* in clojure
pub fn let_(interpreter: &mut Interpreter, args: &mut [Value]) -> Value {
    check_arity!("let*", args.len(), 2);
    let binding_list = try_unwrap_type!("let*", "list", Value::get_list, &args[0], interpreter);
    assert_or_condition!(binding_list.len() % 2 == 0, "bindings must be a list with an even number of objects");

    // replace interpreter scope with fresh child scope
    let parent_scope = interpreter.current_scope.clone();
    interpreter.current_scope = parent_scope.new_child();

    // evaluate binding sequentially in fresh scope
    for binding in binding_list.chunks(2) {
        let binding_name = try_unwrap_type!("let*", "symbol", Value::get_symbol, &binding[0], interpreter);
        let binding_value = interpreter.evaluate(&binding[1]);
        interpreter.current_scope.add_symbol(binding_name, binding_value);
    }

    // evaluate body with new scope and bindings
    let res = interpreter.evaluate(&args[1]);

    // restore old scope
    interpreter.current_scope = parent_scope;

    res
}

// This behaves like loop in clojure
pub fn loop_(interpreter: &mut Interpreter, args: &mut [Value]) -> Value {
    check_arity!("loop", args.len(), 2);
    let binding_list: Vec<Value> = try_unwrap_type!("loop", "list", Value::get_list, &args[0], interpreter);
    assert_or_condition!(binding_list.len() % 2 == 0, "bindings must be a list with an even number of objects");

    let mut binding_names = Vec::with_capacity(binding_list.len() / 2);
    let mut binding_values = Vec::with_capacity(binding_list.len() / 2);
    for binding in binding_list.chunks(2) {
        let binding_name = try_unwrap_type!("loop", "symbol", Value::get_symbol, &binding[0], interpreter);
        let binding_value = interpreter.evaluate(&binding[1]);
        binding_names.push(binding_name);
        binding_values.push(binding_value);
    }

    // replace interpreter scope with fresh child scope
    let parent_scope = interpreter.current_scope.clone();
    interpreter.current_scope = parent_scope.new_child();

    let mut res;
    loop {
        // bind values in fresh scope
        for (&binding_name, binding_value) in binding_names.iter().zip(binding_values.iter()) {
            interpreter.current_scope.add_symbol(binding_name, binding_value.clone());
        }

        // evaluate body with new scope and bindings
        res = interpreter.evaluate(&args[1]);

        // check for recursion
        if let Some(args) = res.get_recur() {
            interpreter.recur_lock = false;
            check_arity!("loop", args.len(), binding_values.len() as u32);
            binding_values = args.iter().cloned().collect();
            continue;
        }

        break;
    }

    // restore old scope
    interpreter.current_scope = parent_scope;

    res
}

// Begin form, for now just evaluates its arguments sequentially and returns the last one
eval_args!(fn begin(args: &mut [Value]) -> Value {
    check_arity!("begin", args.len(), min => 1);
    args.last().cloned().unwrap()
});

// Creates a recur with the supplied arguments
eval_args!(fn recur(interpreter: &mut Interpreter, args: &mut [Value]) -> Value {
    interpreter.recur_lock = true;
    Value::new_recur(args.iter().cloned().collect())
});
