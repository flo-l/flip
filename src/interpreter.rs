use ::value::Value;
use ::scope::Scope;
use ::native;
use ::string_interner::StringInterner;

pub struct Interpreter {
    string_interner: Option<StringInterner>,
    pub current_scope: Scope,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut interpreter = Interpreter {
            string_interner: Some(StringInterner::new()),
            current_scope: Scope::new(),
        };
        interpreter.init();
        interpreter
    }

    fn init(&mut self) {
        self.add_str_to_current_scope("quote", Value::new_native_proc(native::quote));
        self.add_str_to_current_scope("define", Value::new_native_proc(native::define));
        self.add_str_to_current_scope("set!", Value::new_native_proc(native::set));
        self.add_str_to_current_scope("if", Value::new_native_proc(native::if_));
        self.add_str_to_current_scope("eq?", Value::new_native_proc(native::eq_));
        self.add_str_to_current_scope("lambda", Value::new_native_proc(native::lambda));

        self.add_str_to_current_scope("null?", Value::new_native_proc(native::null_));
        self.add_str_to_current_scope("boolean?", Value::new_native_proc(native::boolean_));
        self.add_str_to_current_scope("symbol?", Value::new_native_proc(native::symbol_));
        self.add_str_to_current_scope("integer?", Value::new_native_proc(native::integer_));
        self.add_str_to_current_scope("char?", Value::new_native_proc(native::char_));
        self.add_str_to_current_scope("string?", Value::new_native_proc(native::string_));
        self.add_str_to_current_scope("pair?", Value::new_native_proc(native::pair_));
        self.add_str_to_current_scope("procedure?", Value::new_native_proc(native::procedure_));

        self.add_str_to_current_scope("char->integer", Value::new_native_proc(native::char_integer));
        self.add_str_to_current_scope("integer->char", Value::new_native_proc(native::integer_char));
        self.add_str_to_current_scope("number->string", Value::new_native_proc(native::number_string));
        self.add_str_to_current_scope("string->number", Value::new_native_proc(native::string_number));
        self.add_str_to_current_scope("symbol->string", Value::new_native_proc(native::symbol_string));
        self.add_str_to_current_scope("string->symbol", Value::new_native_proc(native::string_symbol));

        self.add_str_to_current_scope("+", Value::new_native_proc(native::plus));
        self.add_str_to_current_scope("-", Value::new_native_proc(native::minus));
        self.add_str_to_current_scope("*", Value::new_native_proc(native::multiply));
        self.add_str_to_current_scope("quotient", Value::new_native_proc(native::quotient));
        self.add_str_to_current_scope("remainder", Value::new_native_proc(native::remainder));

        self.add_str_to_current_scope("=", Value::new_native_proc(native::eq));
        self.add_str_to_current_scope(">", Value::new_native_proc(native::gt));
        self.add_str_to_current_scope(">=", Value::new_native_proc(native::ge));
        self.add_str_to_current_scope("<", Value::new_native_proc(native::lt));
        self.add_str_to_current_scope("<=", Value::new_native_proc(native::le));

        self.add_str_to_current_scope("cons", Value::new_native_proc(native::cons));
        self.add_str_to_current_scope("list", Value::new_native_proc(native::list));
        self.add_str_to_current_scope("car", Value::new_native_proc(native::car));
        self.add_str_to_current_scope("cdr", Value::new_native_proc(native::cdr));
        self.add_str_to_current_scope("set-car!", Value::new_native_proc(native::set_car_));
        self.add_str_to_current_scope("set-cdr!", Value::new_native_proc(native::set_cdr_));

        self.add_str_to_current_scope("symbol-space", Value::new_native_proc(native::symbol_space));
    }

    pub fn evaluate(&mut self, value: &Value) -> Value {
        let res: Value;
        if let Some(mut list) = value.get_list() {
            if list.len() > 0 {
                let (func, mut args) = list.split_at_mut(1);
                let func = self.evaluate(&func[0]);

                if let Some(f) = func.get_native_fn_ptr() {
                    res = f(self, &mut args)
                } else if let Some(p) = func.get_proc() {
                    res = p.evaluate(self, &args);
                } else {
                    res = Value::new_condition(Value::new_string(format!("tried to call {}, which is not possible", func.to_string(self.get_interner()))));
                }
            } else {
                res = Value::new_condition(Value::new_string(format!("tried to evaluate ()")));
            };
        } else if let Some(symbol) = value.get_symbol() {
            res = self.current_scope
            .lookup_symbol(symbol)
            .unwrap_or(Value::new_condition(Value::new_string(format!("undefined ident: {}", value.to_string(self.get_interner())))));
        } else {
            res = value.clone();
        }

        // TODO handle condition properly
        match res.get_condition() {
            Some(x) => panic!("{}", x.to_string(self.get_interner())),
            _ => (),
        };
        res
    }

    fn add_str_to_current_scope(&mut self, s: &str, value: Value) {
        let id = self.get_interner().intern(s);
        self.current_scope.add_symbol(id, value);
    }

    pub fn get_interner(&mut self) -> &mut StringInterner {
        self.string_interner.as_mut().expect("internal error: string interner uninitialized")
    }

    pub fn move_interner(&mut self) -> StringInterner {
        self.string_interner.take().expect("internal error: string interner uninitialized")
    }

    pub fn set_interner(&mut self, interner: StringInterner) {
        self.string_interner = Some(interner);
    }
}
