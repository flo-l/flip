use super::value::Value;
use super::interpreter::Interpreter;

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

    if let Some(x) = args[0].get_ident() {
        let item = interpreter.evaluate(&args[1]);
        interpreter.current_scope.add_ident(x, item);
        args[0].clone()
    } else {
        panic!("first arg of define has to be an ident, got: {}", args[0]);
    }
}

pub fn set(interpreter: &mut Interpreter, args: &mut [Value]) -> Value {
    if args.len() != 2 {
        panic!("set! accepts exactly 2 arguments");
    }

    if let Some(x) = args[0].get_ident() {
        if interpreter.current_scope.lookup_ident(x).is_some() {
            let item = interpreter.evaluate(&args[1]);
            interpreter.current_scope.add_ident(x, item);
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
type_checker!(symbol_, "symbol?", get_ident);
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
