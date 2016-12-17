use std::ops::{Add, Sub, Mul, Div, Rem};
use ::value::Value;
use ::interpreter::Interpreter;

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
    ($fn_name:expr, $type_name:expr, $unwrap_fn:path, $t:expr) => ({
        match $unwrap_fn($t) {
            Some(x) => x,
            None => {
                let s = format!("{} expected {}, got: {}", $fn_name, $type_name, $t);
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

    let s = try_unwrap_type!("define", "symbol", Value::get_symbol, &args[0]);
    let item = interpreter.evaluate(&args[1]);
    interpreter.current_scope.add_symbol(s, item);
    args[0].clone()
}

pub fn set(interpreter: &mut Interpreter, args: &mut [Value]) -> Value {
    check_arity!("set!", args.len(), 2);

    let s = try_unwrap_type!("set!", "symbol", Value::get_symbol, &args[0]);
    assert_or_condition!(
        interpreter.current_scope.lookup_symbol_with_string(s).is_some(),
        format!("set!: unknown identifier {}", args[0])
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
        raise_condition!(format!("if: argument mismatch: expected bool, got: {}", condition));
    }
}

pub fn lambda(interpreter: &mut Interpreter, args: &mut [Value]) -> Value {
    check_arity!("lambda", args.len(), 2);
    let binding_list = try_unwrap_type!("lambda", "list", Value::get_list, &args[0]);
    let mut bindings: Vec<String> = Vec::with_capacity(binding_list.len());
    for v in binding_list.iter() {
        // type check
        bindings.push(try_unwrap_type!("lambda", "symbol", Value::get_symbol, v).into());
    }

    Value::new_proc(interpreter.current_scope.clone(), bindings, args[1].clone())
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
    )
}

// Polymorphic equality
eval_args!(fn eq_(args: &mut [Value]) -> Value {
    check_arity!("eq?", args.len(), min => 2);
    Value::new_bool(args.windows(2).all(|window| window[0] == window[1]))
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
type_checker!(procedure_, "procedure?", get_native_fn_ptr);

// Type conversions
macro_rules! type_conversion {
    ($func:ident, $lisp_name:expr, $conversion_fn:ident) =>
    (eval_args!(fn $func(args: &mut [Value]) -> Value {
        check_arity!($lisp_name, args.len(), 1);
        Value::$conversion_fn(&args[0])
    }););
}

type_conversion!(char_integer, "char->integer", from_char_to_integer);
type_conversion!(integer_char, "integer->char", from_integer_to_char);
type_conversion!(number_string, "number->string", from_number_to_string);
type_conversion!(string_number, "string->number", from_string_to_number);
type_conversion!(symbol_string, "symbol->string", from_symbol_to_string);
type_conversion!(string_symbol, "string->symbol", from_string_to_symbol);

// Arithmetic operators
macro_rules! arithmetic_operator {
    ($func:ident, $operator:path, $default:expr) =>
    (eval_args!(fn $func(args: &mut [Value]) -> Value {
        let mut res = if args.len() < 2 {
            $default as i64
        } else {
            match args[0].get_integer() {
                Some(i) => i,
                None => raise_condition!(format!("expected integer, got: {}", &args[0]))
            }
        };
        for x in args[1..].iter() {
            if let Some(i) = x.get_integer() {
                res = $operator(res, i);
            } else {
                raise_condition!(format!("expected integer, got: {}", x))
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
    (eval_args!(fn $func(args: &mut [Value]) -> Value {
        check_arity!($lisp_name, args.len(), 1);

        let mut res = true;
        let compared_element = match args[0].get_integer() {
            Some(i) => i,
            None => raise_condition!(format!("expected integer, got: {}", &args[0]))
        };

        for x in &args[1..] {
            let num = match x.get_integer() {
                Some(i) => i,
                None => raise_condition!(format!("expected integer, got: {}", x))
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
eval_args!(fn car(args: &mut [Value]) -> Value {
    check_arity!("car", args.len(), 1);

    if let Some((a, _)) = args[0].get_pair() {
        a.clone()
    } else {
        raise_condition!(format!("expected pair, got {}", &args[0]));
    }
});

eval_args!(fn cdr(args: &mut [Value]) -> Value {
    check_arity!("cdr", args.len(), 1);

    if let Some((_, b)) = args[0].get_pair() {
        b.clone()
    } else {
        raise_condition!(format!("expected pair, got {}", &args[0]));
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
            let quoted = Value::new_list(&[Value::new_symbol("quote"), new_pair]);
            define(interpreter, &mut [f.clone(), quoted])
        } else {
            raise_condition!(format!("expected pair, got {}", old_pair));
        }
    } else {
        raise_condition!(format!("expected symbol, got {}", f));
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
            let quoted = Value::new_list(&[Value::new_symbol("quote"), new_pair]);
            define(interpreter, &mut [f.clone(), quoted])
        } else {
            raise_condition!(format!("expected pair, got {}", old_pair))
        }
    } else {
        raise_condition!(format!("expected symbol, got {}", f));
    }
}

pub fn symbol_space(interpreter: &mut Interpreter, args: &mut [Value]) -> Value {
    check_arity!("symbol-space", args.len(), 0);

    let symbols: Vec<Value> = interpreter.current_scope.symbol_strings()
    .into_iter()
    .map(|s| Value::new_symbol(s))
    .collect();

    Value::new_list(&symbols)
}
