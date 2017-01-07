use std::ops::{Add, Sub, Mul, Div, Rem};
use ::value::Value;
use ::interpreter::Interpreter;
use ::grammar;

macro_rules! check_arity {
    ($name:expr, $len:expr, $exact:expr) => ({
        let len = $len as u32;
        if len != $exact {
            raise_condition!(format!("arity mismatch for {}: expected: {}, got: {}", $name, $exact, len));
        }
    });

    ($name:expr, $len:expr, $lo:expr, $hi:expr) => ({
        let len = $len as u32;
        if len < $lo || len > $hi {
            raise_condition!(format!("arity mismatch for {}: expected: {}..{}, got: {}", $name, $lo, $hi, len));
        }
    });

    ($name:expr, $len:expr, min => $min:expr) => ({
        let len = $len as u32;
        if len < $min {
            raise_condition!(format!("arity mismatch for {}: expected: {}.., got: {}", $name, $min, len));
        }
    });
}

macro_rules! try_unwrap_type {
    ($fn_name:expr, $type_name:expr, $unwrap_fn:path, $t:expr, $interpreter:expr) => ({
        match $unwrap_fn($t) {
            Some(x) => x,
            None => {
                let s = format!("{} expected {}, got: {}", $fn_name, $type_name, $t.to_string(&$interpreter.interner));
                return Value::new_condition(Value::new_string(s));
            }
        }
    });
}

macro_rules! new_condition {
    ($msg:expr) => (
        Value::new_condition(Value::new_string($msg))
    )
}

macro_rules! raise_condition {
    ($msg:expr) => (
        return new_condition!($msg);
    )
}

macro_rules! assert_or_condition {
    ($b:expr, $msg:expr) => ({
        if !$b {
            raise_condition!($msg)
        }
    })
}

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

pub fn set(interpreter: &mut Interpreter, args: &mut [Value]) -> Value {
    check_arity!("set!", args.len(), 2);

    let s = try_unwrap_type!("set!", "symbol", Value::get_symbol, &args[0], interpreter);
    assert_or_condition!(
        interpreter.current_scope.lookup_symbol(s).is_some(),
        format!("set!: unknown identifier {}", args[0].to_string(&interpreter.interner))
    );
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

// the let impls can hardly be deduplicated, because the code modifies different scopes for
// different lets, which makes writing a macro hard because of aliasing &mut pointers
pub fn let_(interpreter: &mut Interpreter, args: &mut [Value]) -> Value {
    check_arity!("let", args.len(), 2);
    let binding_list = try_unwrap_type!("let", "list", Value::get_list, &args[0], interpreter);

    // create fresh scope
    let parent_scope = interpreter.current_scope.clone();
    let mut new_scope = interpreter.current_scope.new_child();

    // evaluate binding sequentially in parent scope
    for binding in binding_list.iter() {
        let binding = try_unwrap_type!("let", "list", Value::get_list, binding, interpreter);
        assert_or_condition!(binding.len() == 2, "binding for let must be a list with length 2");
        let binding_name = try_unwrap_type!("let", "symbol", Value::get_symbol, &binding[0], interpreter);
        let binding_value = interpreter.evaluate(&binding[1]);
        new_scope.add_symbol(binding_name, binding_value);
    }

    // evaluate body with new scope (and bindings)
    interpreter.current_scope = new_scope;
    let res = interpreter.evaluate(&args[1]);

    // restore old scope
    interpreter.current_scope = parent_scope;

    res
}


pub fn let_dash(interpreter: &mut Interpreter, args: &mut [Value]) -> Value {
    check_arity!("let*", args.len(), 2);
    let binding_list = try_unwrap_type!("let*", "list", Value::get_list, &args[0], interpreter);

    // replace interpreter scope with fresh child scope
    let parent_scope = interpreter.current_scope.clone();
    interpreter.current_scope = parent_scope.new_child();

    // evaluate binding sequentially in fresh scope
    for binding in binding_list.iter() {
        let binding = try_unwrap_type!("let*", "list", Value::get_list, binding, interpreter);
        assert_or_condition!(binding.len() == 2, "binding for let* must be a list with length 2");
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

macro_rules! eval_args {
    (fn $func:ident($args:ident : $arg_ty:ty) -> $ret_ty:ty $blk:block) =>
    (
        pub fn $func(interpreter: &mut Interpreter, args: $arg_ty) -> $ret_ty {
            fn inner($args: $arg_ty) -> $ret_ty $blk;
            for x in args.iter_mut() {
                *x = interpreter.evaluate(x);
            }
            inner(args)
        }
    );

    (fn $func:ident($interpreter:ident : &mut Interpreter, $args:ident : $arg_ty:ty) -> $ret_ty:ty $blk:block) =>
    (
        pub fn $func(interpreter: &mut Interpreter, args: $arg_ty) -> $ret_ty {
            fn inner($interpreter: &mut Interpreter, $args: $arg_ty) -> $ret_ty $blk;
            for x in args.iter_mut() {
                *x = interpreter.evaluate(x);
            }
            inner(interpreter, args)
        }
    );
}

// Polymorphic equality
eval_args!(fn poly_eq(args: &mut [Value]) -> Value {
    check_arity!("eq?", args.len(), min => 2);
    Value::new_bool(args.windows(2).all(|window| window[0] == window[1]))
});

// Begin form, for now just evaluates its arguments sequentially and returns the last one
eval_args!(fn begin(args: &mut [Value]) -> Value {
    check_arity!("begin", args.len(), min => 1);
    args.last().cloned().unwrap()
});

// Creates a recur with the supplied arguments
eval_args!(fn recur(args: &mut [Value]) -> Value {
    Value::new_recur(args.iter().cloned().collect())
});

// Type checking
macro_rules! type_checker {
    ($func:ident, $lisp_name:expr, $checking_fn:ident) =>
    (eval_args!(fn $func(args: &mut [Value]) -> Value {
        check_arity!($lisp_name, args.len(), 1);
        Value::new_bool(args[0].$checking_fn().is_some())
    }););
}

type_checker!(null_, "null?", get_empty_list);
type_checker!(boolean_, "boolean?", get_bool);
type_checker!(symbol_, "symbol?", get_symbol);
type_checker!(integer_, "integer?", get_integer);
type_checker!(char_, "char?", get_char);
type_checker!(string_, "string?", get_string);
type_checker!(pair_, "pair?", get_pair);

eval_args!(fn procedure_(args: &mut [Value]) -> Value {
    check_arity!("procedure?", args.len(), 1);
    let is_proc = Value::get_native_fn_ptr(&args[0]).is_some() || Value::get_proc(&args[0]).is_some();
    Value::new_bool(is_proc)
});

// Type conversions
macro_rules! type_conversion {
    ($func:ident, $lisp_name:expr, $type_name:expr, $get_fn:path, $conversion_fn:expr, $new_fn:path) =>
    (eval_args!(fn $func(interpreter: &mut Interpreter, args: &mut [Value]) -> Value {
        check_arity!($lisp_name, args.len(), 1);
        let conversion_fn = $conversion_fn;
        let rust_value = try_unwrap_type!($lisp_name, $type_name, $get_fn, &args[0], interpreter);
        let converted = conversion_fn(rust_value);
        $new_fn(converted)
    }););
}



eval_args!(fn char_integer(interpreter: &mut Interpreter, args: &mut [Value]) -> Value {
    check_arity!("char->integer", args.len(), 1);
    let c = try_unwrap_type!("char->integer", "char", Value::get_char, &args[0], interpreter);
    Value::new_integer(c as i64)
});

eval_args!(fn integer_char(interpreter: &mut Interpreter, args: &mut [Value]) -> Value {
    use std::u32;
    use std::char;
    check_arity!("integer->char", args.len(), 1);
    let i = try_unwrap_type!("integer->char", "integer", Value::get_integer, &args[0], interpreter);
    if i > 0 && i < u32::MAX as i64 {
        let u = i as u32;
        if let Some(c) = char::from_u32(u) {
            return Value::new_char(c)
        }
    }
    raise_condition!("integer is not a valid char");
});

eval_args!(fn number_string(interpreter: &mut Interpreter, args: &mut [Value]) -> Value {
    check_arity!("number->string", args.len(), 1);
    let i = try_unwrap_type!("number->string", "integer", Value::get_integer, &args[0], interpreter);
    Value::new_string(format!("{}", i))
});

eval_args!(fn string_number(interpreter: &mut Interpreter, args: &mut [Value]) -> Value {
    check_arity!("string->number", args.len(), 1);
    let s = try_unwrap_type!("string->number", "string", Value::get_string, &args[0], interpreter);
    if let Ok(v) = grammar::parse_integer(s) {
        return v;
    }
    raise_condition!(format!("string is not a valid integer: {:?}", s));
});

pub fn symbol_string(interpreter: &mut Interpreter, args: &mut [Value]) -> Value {
    check_arity!("symbol->string", args.len(), 1);
    let evaled = interpreter.evaluate(&args[0]);
    let id = try_unwrap_type!("symbol->string", "symbol", Value::get_symbol, &evaled, interpreter);
    if let Some(string) = interpreter.interner.lookup(id) {
        Value::new_string(string)
    } else {
        raise_condition!("internal error: invalid symbol")
    }
}

pub fn string_symbol(interpreter: &mut Interpreter, args: &mut [Value]) -> Value {
    check_arity!("string->symbol", args.len(), 1);
    let evaled = interpreter.evaluate(&args[0]);
    let string = try_unwrap_type!("string->symbol", "string", Value::get_string, &evaled, interpreter);
    let id = interpreter.interner.intern(string);
    Value::new_symbol(id)
}

// Arithmetic operators
macro_rules! arithmetic_operator {
    ($func:ident, $operator:path, $default:expr) =>
    (eval_args!(fn $func(interpreter: &mut Interpreter, args: &mut [Value]) -> Value {
        let mut res = if args.len() < 2 {
            $default as i64
        } else {
            match args[0].get_integer() {
                Some(i) => i,
                None => raise_condition!(format!("expected integer, got: {}", &args[0].to_string(&interpreter.interner)))
            }
        };
        for x in args[1..].iter() {
            if let Some(i) = x.get_integer() {
                res = $operator(res, i);
            } else {
                raise_condition!(format!("expected integer, got: {}", x.to_string(&interpreter.interner)))
            }
        }
        Value::new_integer(res)
    }););
}

arithmetic_operator!(plus, Add::add, 0);
arithmetic_operator!(minus, Sub::sub, 0);
arithmetic_operator!(multiply, Mul::mul, 1);
arithmetic_operator!(quotient, Div::div, 1);
arithmetic_operator!(remainder, Rem::rem, 1);

// Comparison Operators
macro_rules! comparison_operator {
    ($func:ident, $lisp_name:expr, $operator:path) =>
    (eval_args!(fn $func(interpreter: &mut Interpreter, args: &mut [Value]) -> Value {
        check_arity!($lisp_name, args.len(), min => 2);

        let mut res = true;
        let compared_element = match args[0].get_integer() {
            Some(i) => i,
            None => raise_condition!(format!("expected integer, got: {}", &args[0].to_string(&interpreter.interner)))
        };

        for x in &args[1..] {
            let num = match x.get_integer() {
                Some(i) => i,
                None => raise_condition!(format!("expected integer, got: {}", x.to_string(&interpreter.interner)))
            };
            res = res && $operator(&compared_element, &num);
        }
        Value::new_bool(res)
    }););
}

comparison_operator!(eq, "=", PartialEq::eq);
comparison_operator!(lt, "<", PartialOrd::lt);
comparison_operator!(le, "<=", PartialOrd::le);
comparison_operator!(gt, ">", PartialOrd::gt);
comparison_operator!(ge, ">=", PartialOrd::ge);

// List operations:
eval_args!(fn car(interpreter: &mut Interpreter, args: &mut [Value]) -> Value {
    check_arity!("car", args.len(), 1);

    if let Some((a, _)) = args[0].get_pair() {
        a.clone()
    } else {
        raise_condition!(format!("expected pair, got {}", &args[0].to_string(&interpreter.interner)));
    }
});

eval_args!(fn cdr(interpreter: &mut Interpreter, args: &mut [Value]) -> Value {
    check_arity!("cdr", args.len(), 1);

    if let Some((_, b)) = args[0].get_pair() {
        b.clone()
    } else {
        raise_condition!(format!("expected pair, got {}", &args[0].to_string(&interpreter.interner)));
    }
});

eval_args!(fn cons(args: &mut [Value]) -> Value {
    check_arity!("cons", args.len(), 2);
    Value::new_pair(args[0].clone(), args[1].clone())
});

eval_args!(fn list(args: &mut [Value]) -> Value {
    Value::new_list(args)
});

pub fn set_car_(interpreter: &mut Interpreter, args: &mut [Value]) -> Value {
    check_arity!("set-car!", args.len(), 2);

    let (f, elem) = args.split_at(1);
    let (f, elem) = (&f[0], &elem[0]);
    if f.get_symbol().is_some() {
        let old_pair = interpreter.evaluate(f);
        if let Some((_, b)) = old_pair.get_pair() {
            let new_pair = Value::new_pair(elem.clone(), b.clone());
            let quote_id = interpreter.interner.intern("quote");
            let quoted = Value::new_list(&[Value::new_symbol(quote_id), new_pair]);
            define(interpreter, &mut [f.clone(), quoted])
        } else {
            raise_condition!(format!("expected pair, got {}", old_pair.to_string(&interpreter.interner)));
        }
    } else {
        raise_condition!(format!("expected symbol, got {}", f.to_string(&interpreter.interner)));
    }
}

pub fn set_cdr_(interpreter: &mut Interpreter, args: &mut [Value]) -> Value {
    check_arity!("set-cdr!", args.len(), 2);

    let (f, elem) = args.split_at(1);
    let (f, elem) = (&f[0], &elem[0]);
    if f.get_symbol().is_some() {
        let old_pair = interpreter.evaluate(f);
        if let Some((a, _)) = old_pair.get_pair() {
            let new_pair = Value::new_pair(a.clone(), elem.clone());
            let quote_id = interpreter.interner.intern("quote");
            let quoted = Value::new_list(&[Value::new_symbol(quote_id), new_pair]);
            define(interpreter, &mut [f.clone(), quoted])
        } else {
            raise_condition!(format!("expected pair, got {}", old_pair.to_string(&interpreter.interner)))
        }
    } else {
        raise_condition!(format!("expected symbol, got {}", f.to_string(&interpreter.interner)));
    }
}

pub fn symbol_space(interpreter: &mut Interpreter, args: &mut [Value]) -> Value {
    check_arity!("symbol-space", args.len(), 0);

    let symbols: Vec<Value> = interpreter.current_scope.symbol_ids()
    .into_iter()
    .map(|s| Value::new_symbol(s))
    .collect();

    Value::new_list(&symbols)
}
