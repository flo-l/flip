use std::ops::{Add, Sub, Mul, Div, Rem};
use super::value::Value;
use super::interpreter::Interpreter;
use super::scope::SymbolIterator;

pub fn quote(_: &mut Interpreter, args: &mut [Value]) -> Value {
    if args.len() != 1 {
        panic!("quote accepts exactly 1 argument");
    }
    args[0].clone()
}

pub fn define(interpreter: &mut Interpreter, args: &mut [Value]) -> Value {
    if args.len() != 2 {
        panic!("define accepts exactly 2 arguments");
    }

    if let Some(x) = args[0].get_symbol() {
        let item = interpreter.evaluate(&args[1]);
        interpreter.current_scope.add_symbol(&*x, item);
        args[0].clone()
    } else {
        panic!("first arg of define has to be an ident, got: {}", args[0]);
    }
}

pub fn set(interpreter: &mut Interpreter, args: &mut [Value]) -> Value {
    if args.len() != 2 {
        panic!("set! accepts exactly 2 arguments");
    }

    if let Some(x) = args[0].get_symbol() {
        if interpreter.current_scope.lookup_symbol_string(x).is_some() {
            let item = interpreter.evaluate(&args[1]);
            interpreter.current_scope.add_symbol(x, item);
            args[0].clone()
        } else {
            panic!("ident undefined: {}", args[0]);
        }
    } else {
        panic!("first arg of set! has to be an ident, got: {}", args[0]);
    }
}

pub fn if_(interpreter: &mut Interpreter, args: &mut [Value]) -> Value {
    if args.len() != 3 {
        panic!("if accepts exactly 3 arguments");
    }

    let condition = interpreter.evaluate(&args[0]);
    if let Some(b) = condition.get_bool() {

        if b {
            interpreter.evaluate(&args[1])
        } else {
            interpreter.evaluate(&args[2])
        }
    } else {
        panic!("first arg of if has to evaluate to bool, got: {}", condition);
    }
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

// Type checking
macro_rules! type_checker {
    ($func:ident, $lisp_name:expr, $checking_fn:ident) =>
    (eval_args!(fn $func(args: &mut [Value]) -> Value {
        if args.len() != 1 {
            panic!("{} accepts exactly 1 argument", $lisp_name);
        }
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
type_checker!(procedure_, "procedure?", get_fn_ptr);

// Type conversions
macro_rules! type_conversion {
    ($func:ident, $lisp_name:expr, $conversion_fn:ident) =>
    (eval_args!(fn $func(args: &mut [Value]) -> Value {
        if args.len() != 1 {
            panic!("{} accepts exactly 1 argument", $lisp_name);
        }
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
            args[0].get_integer().unwrap_or_else(|| panic!("expected integer, got: {}", &args[0]))
        };
        for x in args[1..].iter() {
            if let Some(i) = x.get_integer() {
                res = $operator(res, i);
            } else {
                panic!("expected intege, got: {}", x);
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
        if args.len() < 1 {
            panic!("{} accepts 1 or more arguments", $lisp_name);
        }

        let mut res = true;
        let compared_element = &args[0].get_integer()
        .unwrap_or_else(|| panic!("expected integer, found {}", &args[0]));

        for x in &args[1..] {
            let num = x.get_integer()
            .unwrap_or_else(|| panic!("expected integer, found {}", &args[0]));
            res = res && $operator(compared_element, &num)
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
    if args.len() != 1 {
        panic!("car accepts exactly 1 argument");
    }

    if let Some((a, _)) = args[0].get_pair() {
        a.clone()
    } else {
        panic!("expected pair, got {}", &args[0])
    }
});

eval_args!(fn cdr(args: &mut [Value]) -> Value {
    if args.len() != 1 {
        panic!("cdr accepts exactly 1 argument");
    }

    if let Some((_, b)) = args[0].get_pair() {
        b.clone()
    } else {
        panic!("expected pair, got {}", &args[0])
    }
});

eval_args!(fn cons(args: &mut [Value]) -> Value {
    if args.len() != 2 {
        panic!("cons accepts exactly 2 argument");
    }

    Value::new_pair(args[0].clone(), args[1].clone())
});

eval_args!(fn list(args: &mut [Value]) -> Value {
    Value::new_list(args)
});

pub fn set_car_(interpreter: &mut Interpreter, args: &mut [Value]) -> Value {
    if args.len() != 2 {
        panic!("set_car! accepts exactly 2 argument");
    }

    let (f, elem) = args.split_at(1);
    let (f, elem) = (&f[0], &elem[0]);
    if f.get_symbol().is_some() {
        let old_pair = interpreter.evaluate(f);
        if let Some((_, b)) = old_pair.get_pair() {
            let new_pair = Value::new_pair(elem.clone(), b.clone());
            let quoted = Value::new_list(&[Value::new_symbol("quote"), new_pair]);
            define(interpreter, &mut [f.clone(), quoted])
        } else {
            panic!("expected pair, got {}", old_pair)
        }
    } else {
        panic!("expected symbol, got {}", f)
    }
}

pub fn set_cdr_(interpreter: &mut Interpreter, args: &mut [Value]) -> Value {
    if args.len() != 2 {
        panic!("set_cdr! accepts exactly 2 argument");
    }

    let (f, elem) = args.split_at(1);
    let (f, elem) = (&f[0], &elem[0]);
    if f.get_symbol().is_some() {
        let old_pair = interpreter.evaluate(f);
        if let Some((a, _)) = old_pair.get_pair() {
            let new_pair = Value::new_pair(a.clone(), elem.clone());
            let quoted = Value::new_list(&[Value::new_symbol("quote"), new_pair]);
            define(interpreter, &mut [f.clone(), quoted])
        } else {
            panic!("expected pair, got {}", old_pair)
        }
    } else {
        panic!("expected symbol, got {}", f)
    }
}

pub fn symbol_space(interpreter: &mut Interpreter, args: &mut [Value]) -> Value {
    if args.len() != 0 {
        panic!("symbol-space accepts no arguments");
    }

    let symbols: Vec<Value> = SymbolIterator::new(&interpreter.current_scope)
    .map(|(_, &(ref s, _))| Value::new_symbol(s.clone()))
    .collect();

    Value::new_list(&symbols)
}
