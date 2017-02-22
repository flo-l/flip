use ::value::Value;
use ::interpreter::Interpreter;

#[derive(Debug, PartialEq, Clone)]
pub enum SpecialForm {
    Define(Define),
    If(If),
    Lambda(Lambda),
    Let(Let),
    Quote(Quote),
}

impl SpecialForm {
    pub fn evaluate(&self, interpreter: &mut Interpreter, args: &[Value]) -> Value {
        match self {
            &SpecialForm::Define(ref x) => x.evaluate(interpreter, args),
            &SpecialForm::If(ref x) => x.evaluate(interpreter, args),
            &SpecialForm::Lambda(ref x) => x.evaluate(interpreter, args),
            &SpecialForm::Let(ref x) => x.evaluate(interpreter, args),
            &SpecialForm::Quote(ref x) => x.evaluate(interpreter, args),
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

    fn evaluate(&self, interpreter: &mut Interpreter, args: &[Value]) -> Value {
        assert!(args.len() == 0);
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

    fn evaluate(&self, interpreter: &mut Interpreter, args: &[Value]) -> Value {
        assert!(args.len() == 0);
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

    fn evaluate(&self, _: &mut Interpreter, args: &[Value]) -> Value {
        assert!(args.len() == 0);
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

    fn evaluate(&self, interpreter: &mut Interpreter, args: &[Value]) -> Value {
        assert!(args.len() == 0);
        Value::new_proc(self.name.clone(), interpreter.current_scope.clone(), self.bindings.clone(), self.code.clone())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Let {
    bindings: Vec<(u64, Value)>,
    code: Vec<Value>,
}

impl Let {
    pub fn new(bindings: Vec<(u64, Value)>, code: Vec<Value>) -> Self {
        Let {
            bindings: bindings,
            code: code,
        }
    }

    fn evaluate(&self, interpreter: &mut Interpreter, args: &[Value]) -> Value {
        assert!(args.len() == 0);
        // replace interpreter scope with fresh child scope
        let parent_scope = interpreter.current_scope.clone();
        interpreter.current_scope = parent_scope.new_child();

        // evaluate binding sequentially in fresh scope
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
}
