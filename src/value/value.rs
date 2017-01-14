use std::rc::Rc;
use std::borrow::Cow;
use std::mem;
use std::char;
use super::value_data::*;
use ::value::{Proc};
use ::interpreter::Interpreter;
use ::scope::Scope;
use ::string_interner::StringInterner;
use ::tail_calls::check_tail_calls;

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
    pub fn new_proc(name: Option<String>, parent_scope: Scope, bindings: Vec<u64>, code: Vec<Value>) -> Self {
        assert_or_condition!(check_tail_calls(&*code), "recur in non-tail position");

        let procedure = Proc::new(name, parent_scope, bindings, code);
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
