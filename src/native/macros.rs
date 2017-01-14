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
