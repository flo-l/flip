use std::rc::Rc;
use std::borrow::Cow;
use std::mem;
use std::char;
use ::interpreter::Interpreter;
use ::scope::Scope;
use ::string_interner::StringInterner;

#[derive(Debug, Clone, PartialEq)]
pub struct Value {
    val_ptr: Rc<ValueData>
}

impl Value {
    fn new_with(data: ValueData) -> Self {
        Value { val_ptr: Rc::new(data) }
    }

    pub fn new_bool(x: bool) -> Self { Self::new_with(ValueData::Bool(x)) }
    pub fn new_char(x: char) -> Self { Self::new_with(ValueData::Char(x)) }
    pub fn new_integer(x: i64) -> Self { Self::new_with(ValueData::Integer(x)) }
    pub fn new_symbol(id: u64) -> Self { Self::new_with(ValueData::Symbol(id)) }
    pub fn new_string<'a, T: 'a + Into<Cow<'a, str>>>(x: T) -> Self { Self::new_with(ValueData::String(x.into().into_owned())) }
    pub fn new_pair(a: Value, b: Value) -> Self { Self::new_with(ValueData::Pair(a,b)) }
    pub fn new_condition(x: Value) -> Self { Self::new_with(ValueData::Condition(x)) }
    pub fn empty_list() -> Self { Self::new_with(ValueData::EmptyList) }
    pub fn new_native_proc(f: fn(&mut Interpreter, &mut [Value]) -> Value) -> Self {
        let raw: *const () = f as *const ();
        Self::new_with(ValueData::NativeProc(raw))
    }
    pub fn new_proc(name: Option<String>, parent_scope: Scope, bindings: Vec<u64>, code: Value) -> Self {
        let procedure = Proc {
            name: name,
            parent_scope: parent_scope,
            bindings: bindings,
            code: code,
        };
        Self::new_with(ValueData::Proc(procedure))
    }
    pub fn new_recur(args: Vec<Value>) -> Self { Self::new_with(ValueData::Recur(args)) }

    fn data(&self) -> &ValueData {
        &*self.val_ptr
    }

    pub fn get_empty_list(&self) -> Option<()> {
        if let &ValueData::EmptyList = self.data() { Some(()) } else { None }
    }

    pub fn get_list(&self) -> Option<Vec<Value>> {
        match self.data() {
            &ValueData::Pair(_, ref b) if b.get_pair().is_some() || b.get_empty_list().is_some() => {
                Some(ListIter::new(self).cloned().collect())
            },
            &ValueData::EmptyList => Some(vec![]),
            _ => None
        }
    }

    pub fn get_symbol(&self) -> Option<u64> {
        match self.data() {
            &ValueData::Symbol(id) => Some(id),
            _ => None,
        }
    }

    pub fn get_bool(&self) -> Option<bool> {
        match self.data() {
            &ValueData::Bool(b) => Some(b),
            _ => None,
        }
    }

    pub fn get_integer(&self) -> Option<i64> {
        match self.data() {
            &ValueData::Integer(i) => Some(i),
            _ => None,
        }
    }

    pub fn get_char(&self) -> Option<char> {
        match self.data() {
            &ValueData::Char(c) => Some(c),
            _ => None,
        }
    }

    pub fn get_string(&self) -> Option<&str> {
        match self.data() {
            &ValueData::String(ref s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn get_pair(&self) -> Option<(&Value, &Value)> {
        match self.data() {
            &ValueData::Pair(ref a, ref b) => Some((a, b)),
            _ => None,
        }
    }

    pub fn get_condition(&self) -> Option<&Value> {
        match self.data() {
            &ValueData::Condition(ref x) => Some(x),
            _ => None,
        }
    }

    pub fn get_native_fn_ptr(&self) -> Option<fn(&mut Interpreter, &mut [Value]) -> Value> {
        match self.data() {
            &ValueData::NativeProc(f) => Some(unsafe { mem::transmute(f) }),
            _ => None,
        }
    }

    pub fn get_proc(&self) -> Option<&Proc> {
        match self.data() {
            &ValueData::Proc(ref p) => Some(p),
            _ => None,
        }
    }

    pub fn get_recur(&self) -> Option<&[Value]> {
        match self.data() {
            &ValueData::Recur(ref args) => Some(&*args),
            _ => None,
        }
    }
    pub fn new_list(elements: &[Value]) -> Value {
        if elements.len() == 0 { return Value::empty_list(); }
        let mut iter = elements.into_iter().rev();
        let last = iter.next().unwrap(); // safe because list len must be >= 1
        iter.fold(Value::new_pair(last.clone(), Value::empty_list()), |prev_pair, value| Value::new_pair(value.clone(), prev_pair))
    }

    pub fn to_string(&self, interner: &StringInterner) -> String {
        self.data().to_string(interner)
    }
}

pub struct ListIter<'a> {
    current: &'a Value,
}

impl<'a> ListIter<'a> {
    pub fn new(val: &'a Value) -> Self {
        ListIter { current: val }
    }
}

impl<'a> Iterator for ListIter<'a> {
    type Item = &'a Value;
    fn next(&mut self) -> Option<Self::Item> {
        match self.current.data() {
            &ValueData::Pair(ref a, ref b) if b.get_pair().is_some() || b.get_empty_list().is_some() => {
                self.current = b;
                Some(a)
            },
            &ValueData::Pair(ref a, _) => {
                let ret = self.current;
                self.current = a; // dummy
                Some(ret)
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ValueData {
    Bool(bool),
    Char(char),
    Integer(i64),
    Symbol(u64),
    String(String),
    Pair(Value, Value),
    EmptyList,
    Condition(Value),
    NativeProc(*const ()),
    Proc(Proc),
    Recur(Vec<Value>),
}

impl ValueData {
    fn to_string(&self, interner: &StringInterner) -> String {
        match self {
            &ValueData::Bool(x) => format!("{}", x),
            &ValueData::Char(x) => format!("{}", x),
            &ValueData::Integer(x) => format!("{}", x),
            &ValueData::Symbol(id) => format!("{}", interner.lookup(id).unwrap_or(&format!("[SYMBOL: {}]", id.to_string()))),
            &ValueData::String(ref x) => format!("\"{}\"", x),
            &ValueData::Pair(ref a, ref b) if b.get_pair().is_some() || b.get_empty_list().is_some() => {
                let iter = ListIter::new(b);
                let mut res = format!("({}", a.to_string(interner));
                for x in iter {
                    res.push_str(&format!(" {}", x.to_string(interner)));
                }
                res.push(')');
                res
            },
            &ValueData::Condition(ref x) => format!("[CONDITION: {:?}]", x),
            &ValueData::Pair(ref a, ref b) => format!("({} . {})", a.to_string(interner), b.to_string(interner)),
            &ValueData::EmptyList => format!("()"),
            &ValueData::NativeProc(x) => format!("[NATIVE_PROC: {:?}]", x),
            &ValueData::Proc(ref p) => format!("[PROC: {}]", p.to_string(interner)),
            &ValueData::Recur(ref p) => format!("[RECUR: {}]", Value::new_list(&p).to_string(interner)),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Proc {
    name: Option<String>,
    parent_scope: Scope,
    bindings: Vec<u64>,
    code: Value,
}

impl Proc {
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
            res = interpreter.evaluate(&self.code);

            // check for recursion
            if let Some(args) = res.get_recur() {
                check_arity!("recur", args.len(), evaluated_args.len() as u32);
                evaluated_args = args.iter().cloned().collect();
                continue;
            }

            break;
        }

        interpreter.current_scope = old_scope;
        res
    }

    fn to_string(&self, interner: &StringInterner) -> String {
        let mut bindings: String = "(".into();
        let bindings_iter = &mut self.bindings.iter().flat_map(|&b| interner.lookup(b));
        for b in bindings_iter.take(self.bindings.len() - 1) {
            bindings.push_str(b);
            bindings.push(' ');
        }
        bindings.push_str(bindings_iter.next().unwrap_or(&String::new()));
        bindings.push(')');

        let name = self.name.as_ref().map(|x| &**x).unwrap_or("lambda");
        format!("({} {} {})", name, bindings, self.code.to_string(interner))
    }
}

#[cfg(test)]
mod test {
    use ::value::Value;
    use ::string_interner::StringInterner;


    #[test]
    fn pair_format() {
        fn v(x: i64) -> Value { Value::new_integer(x) }
        let interner = &mut StringInterner::new();
        let empty = Value::empty_list();

        let a = Value::new_pair(v(4), empty.clone());
        let b = Value::new_pair(v(3), a.clone());
        let c = Value::new_pair(v(2), b.clone());
        let d = Value::new_pair(v(1), c.clone());

        assert_eq!(empty.to_string(interner), "()");
        assert_eq!(a.to_string(interner), "(4)");
        assert_eq!(b.to_string(interner), "(3 4)");
        assert_eq!(c.to_string(interner), "(2 3 4)");
        assert_eq!(d.to_string(interner), "(1 2 3 4)");

        let x = Value::new_pair(v(3), v(4));
        let y = Value::new_pair(v(2), x.clone());
        let z = Value::new_pair(v(1), y.clone());

        assert_eq!(x.to_string(interner), "(3 . 4)");
        assert_eq!(y.to_string(interner), "(2 (3 . 4))");
        assert_eq!(z.to_string(interner), "(1 2 (3 . 4))");

        let r = Value::new_pair(v(4), empty.clone());
        let s = Value::new_pair(v(2), v(3));
        let t = Value::new_pair(s.clone(), r.clone());
        let u = Value::new_pair(v(1), t.clone());
        let v = Value::new_pair(v(0), u.clone());

        assert_eq!(r.to_string(interner), "(4)");
        assert_eq!(s.to_string(interner), "(2 . 3)");
        assert_eq!(t.to_string(interner), "((2 . 3) 4)");
        assert_eq!(u.to_string(interner), "(1 (2 . 3) 4)");
        assert_eq!(v.to_string(interner), "(0 1 (2 . 3) 4)");
    }
}
