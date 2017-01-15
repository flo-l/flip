use itertools::Itertools;
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
    check_arity!("lambda", args.len(), min => 2);

    let name: Option<String>;
    let binding_list: &Value;
    let code: &[Value];

    if let Some(id) = Value::get_symbol(&args[0]) {
        // if there is a name the arity must be at least 3
        check_arity!("lambda", args.len(), min => 3);

        let name_id = id;
        name = interpreter.interner.lookup(name_id).map(Into::into);

        binding_list = &args[1];
        code = &args[2..];
    } else {
        name = None;
        binding_list = &args[0];
        code = &args[1..];
    }

    let binding_list = try_unwrap_type!("lambda", "list", Value::get_list, binding_list, interpreter);

    let mut bindings: Vec<u64> = Vec::with_capacity(binding_list.len());
    for v in binding_list.iter() {
        // type check
        bindings.push(try_unwrap_type!("lambda", "symbol", Value::get_symbol, v, interpreter));
    }

    Value::new_proc(name, interpreter.current_scope.clone(), bindings, code.iter().cloned().collect())
}

// This behaves like let* in clojure
pub fn let_(interpreter: &mut Interpreter, args: &mut [Value]) -> Value {
    check_arity!("let*", args.len(), min => 2);
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
    let mut res = Value::empty_list();
    for body in &args[1..] {
        res = interpreter.evaluate(body);
    }

    // restore old scope
    interpreter.current_scope = parent_scope;

    res
}

// This behaves like loop in clojure
pub fn loop_(interpreter: &mut Interpreter, args: &mut [Value]) -> Value {
    check_arity!("loop", args.len(), min => 2);
    let binding_list: Vec<Value> = try_unwrap_type!("loop", "list", Value::get_list, &args[0], interpreter);
    assert_or_condition!(binding_list.len() % 2 == 0, "bindings must be a list with an even number of objects");


    // type check binding names and creates a vec with their ids
    let binding_ids = binding_list.iter()
    .step(2)
    .map(|name| {
        if let Some(id) = Value::get_symbol(name) {
            Ok(id)
        } else {
            Err(new_condition!(format!("loop expected symbol, got: {}", name.to_string(&interpreter.interner))))
        }
    })
    .collect::<Result<Vec<u64>, Value>>();

    // if any name was not a symbol, return the condition raised
    let binding_ids = match binding_ids {
        Ok(ids) => ids,
        Err(condition) => return condition,
    };

    // values are elements 1, 3, 5, etc.
    let binding_values = binding_list.iter().skip(1).step(2);

    // replace interpreter scope with fresh child scope
    let parent_scope = interpreter.current_scope.clone();
    interpreter.current_scope = parent_scope.new_child();

    // bind values in fresh scope
    for (&binding_id, binding_value) in binding_ids.iter().zip(binding_values) {
        let evaluated_value = interpreter.evaluate(binding_value);
        interpreter.current_scope.add_symbol(binding_id, evaluated_value);
    }

    let mut res;
    loop {
        // evaluate each body with new scope and bindings
        for body in args[1..].iter().take(args.len() - 2) {
            let res = interpreter.evaluate(body);

            // check for invalid recursion
            if let Some(_) = res.get_recur() {
                interpreter.recur_lock = false;
                raise_condition!("recur in non-tail position")
            }
        }

        res = interpreter.evaluate(&args.last().unwrap()); // safe because of arity check

        // check for recursion
        if let Some(args) = res.get_recur() {
            interpreter.recur_lock = false;
            check_arity!("loop", args.len(), binding_ids.len() as u32);

            // bind new values from recur
            for (&binding_id, binding_value) in binding_ids.iter().zip(args.iter()) {
                interpreter.current_scope.add_symbol(binding_id, binding_value.clone());
            }

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
