use ::value::Value;
use ::scope::Scope;
use ::interpreter::Interpreter;
use ::string_interner::StringInterner;
use itertools::Itertools;

#[derive(Debug, PartialEq, Clone)]
pub struct Proc {
    name: Option<String>,
    parent_scope: Scope,
    bindings: Vec<u64>,
    code: Vec<Value>,
}

impl Proc {
    pub fn new(name: Option<String>, parent_scope: Scope, bindings: Vec<u64>, code: Vec<Value>) -> Self {
        Proc {
            name: name,
            parent_scope: parent_scope,
            bindings: bindings,
            code: code,
        }
    }

    pub fn evaluate(&self, interpreter: &mut Interpreter, args: &[Value]) -> Value {
        if self.bindings.len() != args.len() {
            let name = self.name.clone().unwrap_or("lambda".into());
            return Value::new_condition(Value::new_string(
                format!("arity mismatch for {}: expected: {}, got: {}", name, self.bindings.len(), args.len())));
        }

        let mut res;

        // evaluate args in current scope
        let mut evaluated_args: Vec<Value> = args.iter().map(|x| interpreter.evaluate(x)).collect();

        // backup current scope
        let old_scope = interpreter.current_scope.clone(); // this is just one Rc::clone

        // create new scope for fn from fns parent scope
        interpreter.current_scope = self.parent_scope.new_child();

        // loop for recur
        loop {
            // add args to fn scope
            for (&binding, value) in self.bindings.iter().zip(evaluated_args.iter()) {
                interpreter.current_scope.add_symbol(binding, value.clone());
            }

            // evaluate code in fn scope
            for body in self.code.iter().take(self.code.len() - 1) {
                let res = interpreter.evaluate(body);

                // check for invalid recursion
                if let Some(_) = res.get_recur() {
                    interpreter.recur_lock = false;
                    raise_condition!("recur in non-tail position")
                }
            }

            res = interpreter.evaluate(&self.code.last().unwrap()); // safe because of arity check

            // check for recursion
            if let Some(args) = res.get_recur() {
                interpreter.recur_lock = false;
                check_arity!("recur", args.len(), evaluated_args.len() as u32);
                evaluated_args = args.iter().cloned().collect();
                continue;
            }

            break;
        }

        interpreter.current_scope = old_scope;
        res
    }

    pub fn to_string(&self, interner: &StringInterner) -> String {
        let name = self.name.as_ref().map(|x| &**x).unwrap_or("lambda");

        let bindings = self.bindings.iter()
        .flat_map(|&b| interner.lookup(b))
        .join(" ");

        let code = self.code.iter()
        .map(|x| x.to_string(interner))
        .join(" ");

        format!("({} ({}) {})", name, bindings, code)
    }
}
