use std::rc::Rc;
use std::borrow::Cow;
use std::mem;
use std::char;
use super::value_data::*;
use ::value::{Proc};
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
    pub fn new_condition(x: Value) -> Self { Self::new_with(ValueData::Condition(x)) }
    pub fn empty_list() -> Self { Self::new_with(ValueData::EmptyList) }
    pub fn new_native_proc(f: fn(&mut Interpreter, &mut [Value]) -> Value) -> Self {
        let raw: *const () = f as *const ();
        Self::new_with(ValueData::NativeProc(raw))
    }
    pub fn new_proc(name: Option<String>, parent_scope: Scope, bindings: Vec<u64>, code: Vec<Value>) -> Self {
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
            &ValueData::List(ref values) => {
                Some(values.clone())
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
        Value::new_with(ValueData::List(elements.iter().cloned().collect()))
    }

    pub fn to_string(&self, interner: &StringInterner) -> String {
        self.data().to_string(interner)
    }
}
