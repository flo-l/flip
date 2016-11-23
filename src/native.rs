use super::value::Value;
use super::interpreter::Interpreter;

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
