use ::value::{Value, LetLoop};
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
        // every function's body is enclosed in an implicit loop
        let bindings: Vec<(u64, Value)> = self.bindings.iter().cloned().zip(args.iter().cloned()).collect();
        let implicit_loop = LetLoop::new(bindings, self.code.clone());
        implicit_loop.evaluate_loop(interpreter)
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
