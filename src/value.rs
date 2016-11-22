use std::fmt;
use std::rc::Rc;
use std::borrow::Cow;
use std::mem;
use super::scope::Scope;
use super::interpreter::Interpreter;
use super::native;

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
    pub fn new_ident<'a, T: 'a + Into<Cow<'a, str>>>(x: T) -> Self { Self::new_with(ValueData::Ident(x.into().into_owned())) }
    pub fn new_string<'a, T: 'a + Into<Cow<'a, str>>>(x: T) -> Self { Self::new_with(ValueData::String(x.into().into_owned())) }
    pub fn new_pair(a: Value, b: Value) -> Self { Self::new_with(ValueData::Pair(a,b)) }
    pub fn empty_list() -> Self { Self::new_with(ValueData::EmptyList) }
    pub fn new_native_proc(f: fn(&mut Interpreter, &mut [Value]) -> Value) -> Self {
        let raw: *const () = f as *const ();
        Self::new_with(ValueData::NativeProc(raw))
    }

    pub fn data(&self) -> &ValueData { &*self.val_ptr }
    pub fn get_fn_ptr(&self) -> Option<fn(&mut Interpreter, &mut [Value]) -> Value> {
        match self.data() {
            &ValueData::NativeProc(f) => Some(unsafe { mem::transmute(f) }),
            _ => None,
        }
    }

    fn is_pair(&self) -> bool {
        if let &ValueData::Pair(_, _) = self.data() { true } else { false }
    }

    pub fn is_list(&self) -> bool {
        match self.data() {
            &ValueData::Pair(_, ref b) => b.is_pair() || b.is_empty_list(),
            &ValueData::EmptyList => true,
            _ => false
        }
    }

    fn is_empty_list(&self) -> bool {
        if let &ValueData::EmptyList = self.data() { true } else { false }
    }

    pub fn get_ident(&self) -> Option<&str> {
        match self.data() {
            &ValueData::Ident(ref s) => Some(&*s),
            _ => None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.val_ptr.fmt(f)
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
            &ValueData::Pair(ref a, ref b) if b.is_pair() || b.is_empty_list() => {
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
pub enum ValueData {
    Bool(bool),
    Char(char),
    Integer(i64),
    Ident(String),
    String(String),
    Pair(Value, Value),
    EmptyList,
    NativeProc(*const ()),
}

impl fmt::Display for ValueData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &ValueData::Bool(x) => write!(f, "{}", x),
            &ValueData::Char(x) => write!(f, "{}", x),
            &ValueData::Integer(x) => write!(f, "{}", x),
            &ValueData::Ident(ref x) => write!(f, "{}", x),
            &ValueData::String(ref x) => write!(f, "\"{}\"", x),
            &ValueData::Pair(ref a, ref b) if b.is_pair() || b.is_empty_list() => {
                let iter = ListIter::new(b);
                let mut res = write!(f, "({}", a);
                for x in iter {
                    res = res.and(write!(f, " {}", x));
                }
                res.and(write!(f, ")"))
            },
            &ValueData::Pair(ref a, ref b) => write!(f, "({} . {})", a, b),
            &ValueData::EmptyList => write!(f, "()"),
            &ValueData::NativeProc(x) => write!(f, "[NATIVE_PROC: {:?}]", x),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Value, ValueData};

    #[test]
    fn pair_format() {
        fn v(x: i64) -> Value { Value::new_integer(x) }
        let empty = Value::empty_list();

        let a = Value::new_pair(v(4), empty.clone());
        let b = Value::new_pair(v(3), a.clone());
        let c = Value::new_pair(v(2), b.clone());
        let d = Value::new_pair(v(1), c.clone());

        assert_eq!(format!("{}", empty), "()");
        assert_eq!(format!("{}", a), "(4)");
        assert_eq!(format!("{}", b), "(3 4)");
        assert_eq!(format!("{}", c), "(2 3 4)");
        assert_eq!(format!("{}", d), "(1 2 3 4)");

        let x = Value::new_pair(v(3), v(4));
        let y = Value::new_pair(v(2), x.clone());
        let z = Value::new_pair(v(1), y.clone());

        assert_eq!(format!("{}", x), "(3 . 4)");
        assert_eq!(format!("{}", y), "(2 (3 . 4))");
        assert_eq!(format!("{}", z), "(1 2 (3 . 4))");

        let r = Value::new_pair(v(4), empty.clone());
        let s = Value::new_pair(v(2), v(3));
        let t = Value::new_pair(s.clone(), r.clone());
        let u = Value::new_pair(v(1), t.clone());
        let v = Value::new_pair(v(0), u.clone());

        assert_eq!(format!("{}", r), "(4)");
        assert_eq!(format!("{}", s), "(2 . 3)");
        assert_eq!(format!("{}", t), "((2 . 3) 4)");
        assert_eq!(format!("{}", u), "(1 (2 . 3) 4)");
        assert_eq!(format!("{}", v), "(0 1 (2 . 3) 4)");
    }
}
