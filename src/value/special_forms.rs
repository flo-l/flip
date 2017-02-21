use ::value::Value;
use ::interpreter::Interpreter;
use ::string_interner::StringInterner;

#[derive(Debug, PartialEq, Clone)]
pub enum SpecialForm {
    Define(Define),
    If(If),
    Quote(Quote),
}

impl SpecialForm {
    pub fn evaluate(&self, interpreter: &mut Interpreter, args: &[Value]) -> Value {
        match self {
            &SpecialForm::Define(ref x) => x.evaluate(interpreter, args),
            &SpecialForm::If(ref x) => x.evaluate(interpreter, args),
            &SpecialForm::Quote(ref x) => x.evaluate(interpreter, args),
        }
    }

    pub fn to_string(&self, interner: &StringInterner) -> String {
        match self {
            &SpecialForm::Define(ref x) => x.to_string(interner),
            &SpecialForm::If(ref x) => x.to_string(interner),
            &SpecialForm::Quote(ref x) => x.to_string(interner),
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

    fn to_string(&self, interner: &StringInterner) -> String {
        let condition = self.condition.to_string(interner);
        let then = self.then.to_string(interner);
        let or_else = self.or_else.to_string(interner);

        format!("(if {} {} {})", condition, then, or_else)
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

    fn to_string(&self, interner: &StringInterner) -> String {
        let symbol = Value::new_symbol(self.symbol_id).to_string(interner);
        let expr = self.expression.to_string(interner);

        format!("(define {} {})", symbol, expr)
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

    fn to_string(&self, interner: &StringInterner) -> String {
        let expr = self.expression.to_string(interner);
        format!("(quote {})", expr)
    }
}
