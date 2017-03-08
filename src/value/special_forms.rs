use ::value::Value;
use ::interpreter::Interpreter;

#[derive(Debug, PartialEq, Clone)]
pub enum SpecialForm {
    Begin(Begin),
    Define(Define),
    If(If),
    Lambda(Lambda),
    Let(LetLoop),
    Loop(LetLoop),
    RecurForm(RecurForm),
    Quote(Quote),
}

impl SpecialForm {
    pub fn evaluate(&self, interpreter: &mut Interpreter) -> Value {
        match self {
            &SpecialForm::Begin(ref x) => x.evaluate(interpreter),
            &SpecialForm::Define(ref x) => x.evaluate(interpreter),
            &SpecialForm::If(ref x) => x.evaluate(interpreter),
            &SpecialForm::Lambda(ref x) => x.evaluate(interpreter),
            &SpecialForm::Let(ref x) => x.evaluate_let(interpreter),
            &SpecialForm::Loop(ref x) => x.evaluate_loop(interpreter),
            &SpecialForm::RecurForm(ref x) => x.evaluate(interpreter),
            &SpecialForm::Quote(ref x) => x.evaluate(interpreter),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct If {
    condition: Value,
    then: Value,
    or_else: Value,
}

impl If {
    pub fn new(condition: Value, then: Value, or_else: Value) -> Self {
        If {
            condition: condition,
            then: then,
            or_else: or_else,
        }
    }

    fn evaluate(&self, interpreter: &mut Interpreter) -> Value {
        let condition = interpreter.evaluate(&self.condition);
        match condition.get_bool() {
            Some(true)  => interpreter.evaluate(&self.then),
            Some(false) => interpreter.evaluate(&self.or_else),
            None => new_condition!(format!("if expected bool, found: {}", condition.to_string(&interpreter.interner)))
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Define {
    symbol_id: u64,
    expression: Value,
}

impl Define {
    pub fn new(symbol_id: u64, expression: Value) -> Self {
        Define {
            symbol_id: symbol_id,
            expression: expression,
        }
    }

    fn evaluate(&self, interpreter: &mut Interpreter) -> Value {
        let expr = interpreter.evaluate(&self.expression);
        interpreter.current_scope.add_symbol(self.symbol_id, expr);
        Value::new_symbol(self.symbol_id)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Quote {
    expression: Value,
}

impl Quote {
    pub fn new(expression: Value) -> Self {
        Quote {
            expression: expression,
        }
    }

    fn evaluate(&self, _: &mut Interpreter) -> Value {
        self.expression.clone()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Lambda {
    name: Option<String>,
    bindings: Vec<u64>,
    code: Vec<Value>,
}

impl Lambda {
    pub fn new(name: Option<String>, bindings: Vec<u64>, code: Vec<Value>) -> Self {
        Lambda {
            name: name,
            bindings: bindings,
            code: code,
        }
    }

    fn evaluate(&self, interpreter: &mut Interpreter) -> Value {
        Value::new_proc(self.name.clone(), interpreter.current_scope.clone(), self.bindings.clone(), self.code.clone())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct LetLoop {
    bindings: Vec<(u64, Value)>,
    code: Vec<Value>,
}

impl LetLoop {
    pub fn new(bindings: Vec<(u64, Value)>, code: Vec<Value>) -> Self {
        LetLoop {
            bindings: bindings,
            code: code,
        }
    }

    fn evaluate_let(&self, interpreter: &mut Interpreter) -> Value {
        // replace interpreter scope with fresh child scope
        let parent_scope = interpreter.current_scope.clone();
        interpreter.current_scope = parent_scope.new_child();

        // evaluate bindings sequentially in fresh scope
        for &(binding_name, ref binding_value) in &self.bindings {
            let binding_value = interpreter.evaluate(binding_value);
            interpreter.current_scope.add_symbol(binding_name, binding_value);
        }

        // evaluate body with new scope and bindings
        let mut res = Value::empty_list();
        for body in &self.code {
            res = interpreter.evaluate(body);
        }

        // restore old scope
        interpreter.current_scope = parent_scope;

        res
    }

    // pub because this is also used for procs
    pub fn evaluate_loop(&self, interpreter: &mut Interpreter) -> Value {
        // replace interpreter scope with fresh child scope
        let parent_scope = interpreter.current_scope.clone();
        interpreter.current_scope = parent_scope.new_child();

        // evaluate bindings sequentially in fresh scope
        for &(binding_name, ref binding_value) in &self.bindings {
            let binding_value = interpreter.evaluate(binding_value);
            interpreter.current_scope.add_symbol(binding_name, binding_value);
        }

        let mut res = Value::empty_list();
        loop {
            // evaluate body with new scope and bindings
            for body in &self.code {
                res = interpreter.evaluate(body);
            }

            // check for recursion
            if let Some(args) = res.get_recur() {
                check_arity!("loop", args.len(), self.bindings.len() as u32);

                // recreate a new scope
                interpreter.current_scope = parent_scope.new_child();

                // bind values from recur
                for (binding_name, binding_value) in self.bindings.iter().map(|&(name, _)| name).zip(args.iter()) {
                    interpreter.current_scope.add_symbol(binding_name, binding_value.clone());
                }

                continue;
            }

            // restore old scope
            interpreter.current_scope = parent_scope;
            return res;
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct RecurForm {
    bindings: Vec<Value>,
}

impl RecurForm {
    pub fn new(bindings: Vec<Value>) -> Self {
        RecurForm {
            bindings: bindings,
        }
    }

    fn evaluate(&self, interpreter: &mut Interpreter) -> Value {
        let evaluated_bindings = self.bindings.iter().map(|b| interpreter.evaluate(b)).collect();
        Value::new_recur(evaluated_bindings)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Begin {
    code: Vec<Value>,
}

impl Begin {
    pub fn new(code: Vec<Value>) -> Self {
        Begin {
            code: code,
        }
    }

    fn evaluate(&self, interpreter: &mut Interpreter) -> Value {
        let evaluated_code = self.code.iter().map(|b| interpreter.evaluate(b)).last();
        evaluated_code.unwrap_or(Value::empty_list())
    }
}
