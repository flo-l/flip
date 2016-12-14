use super::value::{Value, ListIter};
use super::scope::Scope;
use super::native;

pub struct Interpreter {
    pub current_scope: Scope,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut interpreter = Interpreter {
            current_scope: Scope::new()
        };
        interpreter.init();
        interpreter
    }

    fn init(&mut self) {
        self.current_scope.add_symbol("quote", Value::new_native_proc(native::quote));
        self.current_scope.add_symbol("define", Value::new_native_proc(native::define));
        self.current_scope.add_symbol("set!", Value::new_native_proc(native::set));
        self.current_scope.add_symbol("if", Value::new_native_proc(native::if_));

        self.current_scope.add_symbol("null?", Value::new_native_proc(native::null_));
        self.current_scope.add_symbol("boolean?", Value::new_native_proc(native::boolean_));
        self.current_scope.add_symbol("symbol?", Value::new_native_proc(native::symbol_));
        self.current_scope.add_symbol("integer?", Value::new_native_proc(native::integer_));
        self.current_scope.add_symbol("char?", Value::new_native_proc(native::char_));
        self.current_scope.add_symbol("string?", Value::new_native_proc(native::string_));
        self.current_scope.add_symbol("pair?", Value::new_native_proc(native::pair_));
        self.current_scope.add_symbol("procedure?", Value::new_native_proc(native::procedure_));

        self.current_scope.add_symbol("char->integer", Value::new_native_proc(native::char_integer));
        self.current_scope.add_symbol("integer->char", Value::new_native_proc(native::integer_char));
        self.current_scope.add_symbol("number->string", Value::new_native_proc(native::number_string));
        self.current_scope.add_symbol("string->number", Value::new_native_proc(native::string_number));
        self.current_scope.add_symbol("symbol->string", Value::new_native_proc(native::symbol_string));
        self.current_scope.add_symbol("string->symbol", Value::new_native_proc(native::string_symbol));

        self.current_scope.add_symbol("+", Value::new_native_proc(native::plus));
        self.current_scope.add_symbol("-", Value::new_native_proc(native::minus));
        self.current_scope.add_symbol("*", Value::new_native_proc(native::multiply));
        self.current_scope.add_symbol("quotient", Value::new_native_proc(native::quotient));
        self.current_scope.add_symbol("remainder", Value::new_native_proc(native::remainder));

        self.current_scope.add_symbol("=", Value::new_native_proc(native::eq));
        self.current_scope.add_symbol(">", Value::new_native_proc(native::gt));
        self.current_scope.add_symbol(">=", Value::new_native_proc(native::ge));
        self.current_scope.add_symbol("<", Value::new_native_proc(native::lt));
        self.current_scope.add_symbol("<=", Value::new_native_proc(native::le));

        self.current_scope.add_symbol("cons", Value::new_native_proc(native::cons));
        self.current_scope.add_symbol("list", Value::new_native_proc(native::list));
        self.current_scope.add_symbol("car", Value::new_native_proc(native::car));
        self.current_scope.add_symbol("cdr", Value::new_native_proc(native::cdr));
        self.current_scope.add_symbol("set-car!", Value::new_native_proc(native::set_car_));
        self.current_scope.add_symbol("set-cdr!", Value::new_native_proc(native::set_cdr_));

        self.current_scope.add_symbol("symbol-space", Value::new_native_proc(native::symbol_space));
    }

    pub fn evaluate(&mut self, value: &Value) -> Value {
        let res: Value;
        if value.is_list() {
            let mut list: Vec<Value> = ListIter::new(value).cloned().collect();
            if list.len() > 0 {
                let (func, mut args) = list.split_at_mut(1);
                let func = self.evaluate(&func[0]);

                if let Some(f) = func.get_fn_ptr() {
                    res = f(self, &mut args)
                } else {
                    res = Value::new_condition(Value::new_string(format!("tried to call {}, which is not possible", func)));
                }
            } else {
                res = Value::new_condition(Value::new_string(format!("tried to evaluate ()")));
            };
        } else if let Some(symbol) = value.get_symbol() {
            res = self.current_scope
            .lookup_symbol_string(symbol)
            .map(|&(_, ref value)| value.clone())
            .unwrap_or(Value::new_condition(Value::new_string(format!("undefined ident: {}", value))));
        } else {
            res = value.clone();
        }

        // TODO handle condition properly
        match res.get_condition() {
            Some(x) => panic!("{}", x),
            _ => (),
        };
        res
    }
}
