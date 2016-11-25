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
        self.current_scope.add_ident("quote", Value::new_native_proc(native::quote));
        self.current_scope.add_ident("define", Value::new_native_proc(native::define));
        self.current_scope.add_ident("set!", Value::new_native_proc(native::set));
        self.current_scope.add_ident("if", Value::new_native_proc(native::if_));

        self.current_scope.add_ident("null?", Value::new_native_proc(native::null_));
        self.current_scope.add_ident("boolean?", Value::new_native_proc(native::boolean_));
        self.current_scope.add_ident("symbol?", Value::new_native_proc(native::symbol_));
        self.current_scope.add_ident("integer?", Value::new_native_proc(native::integer_));
        self.current_scope.add_ident("char?", Value::new_native_proc(native::char_));
        self.current_scope.add_ident("string?", Value::new_native_proc(native::string_));
        self.current_scope.add_ident("pair?", Value::new_native_proc(native::pair_));
        self.current_scope.add_ident("procedure?", Value::new_native_proc(native::procedure_));

        self.current_scope.add_ident("char->integer", Value::new_native_proc(native::char_integer));
        self.current_scope.add_ident("integer->char", Value::new_native_proc(native::integer_char));
        self.current_scope.add_ident("number->string", Value::new_native_proc(native::number_string));
        self.current_scope.add_ident("string->number", Value::new_native_proc(native::string_number));
        self.current_scope.add_ident("symbol->string", Value::new_native_proc(native::symbol_string));
        self.current_scope.add_ident("string->symbol", Value::new_native_proc(native::string_symbol));

        self.current_scope.add_ident("+", Value::new_native_proc(native::plus));
        self.current_scope.add_ident("-", Value::new_native_proc(native::minus));
        self.current_scope.add_ident("*", Value::new_native_proc(native::multiply));
        self.current_scope.add_ident("quotient", Value::new_native_proc(native::quotient));
        self.current_scope.add_ident("remainder", Value::new_native_proc(native::remainder));

        self.current_scope.add_ident("=", Value::new_native_proc(native::eq));
        self.current_scope.add_ident(">", Value::new_native_proc(native::gt));
        self.current_scope.add_ident(">=", Value::new_native_proc(native::ge));
        self.current_scope.add_ident("<", Value::new_native_proc(native::lt));
        self.current_scope.add_ident("<=", Value::new_native_proc(native::le));

        self.current_scope.add_ident("cons", Value::new_native_proc(native::cons));
        self.current_scope.add_ident("list", Value::new_native_proc(native::list));
        self.current_scope.add_ident("car", Value::new_native_proc(native::car));
        self.current_scope.add_ident("cdr", Value::new_native_proc(native::cdr));
        self.current_scope.add_ident("set-car!", Value::new_native_proc(native::set_car_));
        self.current_scope.add_ident("set-cdr!", Value::new_native_proc(native::set_cdr_));
    }

/*
    pub fn lookup_ident(ident: &str) -> Value {
        if let Some(val) = self.scopes.last().lookup_ident(ident) {
            val
        } else {
            println!("Searched Ident: {:?}", scope::intern_ident(ident));
            println!("Known Idents: ");
            for x in &self.scopes {
                for (hash, value) in &x.idents {
                    println!("({:?}) => {:?}", hash, value);
                }
            }
            panic!("Ident: {:?} not found.", ident);
        }
    }*/

    pub fn evaluate(&mut self, value: &Value) -> Value {
        if value.is_list() {
            let mut list: Vec<Value> = ListIter::new(value).cloned().collect();
            let (func, mut args) = list.split_at_mut(1);
            let func = if func.len() == 1 { self.evaluate(&func[0]) } else { panic!("tried to evaluate ()") };

            if let Some(f) = func.get_fn_ptr() {
                f(self, &mut args)
            } else {
                panic!("tried to call {}, which is not possible", func)
            }
        } else if let Some(ident) = value.get_ident() {
            self.current_scope
            .lookup_ident(ident)
            .cloned()
            .unwrap_or_else(|| panic!("undefined ident: {}", value))
        } else {
            value.clone()
        }
    }
}
