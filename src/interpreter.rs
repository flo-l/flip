use super::value::{Value, ValueData, ListIter};
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
        self.current_scope.add_ident("define", Value::new_native_proc(native::define));
        self.current_scope.add_ident("set!", Value::new_native_proc(native::set));
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
        } else {
            match value.data() {
                &ValueData::Ident(ref s) => {
                    self.current_scope.lookup_ident(s)
                    .unwrap_or_else(|| panic!("undefined ident: {}", s))
                },
                _ => value.clone()
            }
        }
    }
}
