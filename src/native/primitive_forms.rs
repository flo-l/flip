use std::ops::{Add, Sub, Mul, Div, Rem};
use ::value::Value;
use ::interpreter::Interpreter;
use ::grammar;
use super::special_forms::define;

// Polymorphic equality
eval_args!(fn poly_eq(args: &mut [Value]) -> Value {
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
