// this is used to automate arity checking of native functions
// name is always the functions name as string, eg. for define name is "define"
// len is always the number of arguments
macro_rules! check_arity {
    // exact arity
    ($name:expr, $len:expr, $exact:expr) => ({
        let len = $len as u32;
        if len != $exact {
            raise_condition!(format!("arity mismatch for {}: expected: {}, got: {}", $name, $exact, len));
        }
    });

    // range
    ($name:expr, $len:expr, $lo:expr, $hi:expr) => ({
        let len = $len as u32;
        if len < $lo || len > $hi {
            raise_condition!(format!("arity mismatch for {}: expected: {}..{}, got: {}", $name, $lo, $hi, len));
        }
    });

    // minimum
    ($name:expr, $len:expr, min => $min:expr) => ({
        let len = $len as u32;
        if len < $min {
            raise_condition!(format!("arity mismatch for {}: expected: {}.., got: {}", $name, $min, len));
        }
    });
}

// this automates type checking. If the expected type is not found, a condition is returned early
// name: functions name
// type_name: name of type, eg. "string" or "list"
// unwrap_fn: a fn taking a value and returning Option<T>, where T is the rust type that is expected from the value.
// value: the value to unwrap
// interpreter: &mut Interpreter
macro_rules! try_unwrap_type {
    ($fn_name:expr, $type_name:expr, $unwrap_fn:path, $value:expr, $interpreter:expr) => ({
        match $unwrap_fn($value) {
            Some(x) => x,
            None => {
                let s = format!("{} expected {}, got: {}", $fn_name, $type_name, $value.to_string(&$interpreter.interner));
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

// This automates the evaluation of arguments supplied to functions
// arguments are just evaluated in order
macro_rules! eval_args {
    // args is an array of already evaluated args
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

    // same as above, but with interpreter available
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
