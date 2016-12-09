use std::fmt;
use std::rc::Rc;
use std::borrow::Cow;
use std::mem;
use std::char;
use std::hash::{Hash, Hasher};
use siphasher::sip::SipHasher24 as SipHasher;
use ::interpreter::Interpreter;
use ::grammar;

#[derive(Debug, Clone, PartialEq)]
pub struct Value {
    val_ptr: Rc<ValueData>
}

macro_rules! check_type {
    ($unwrap_fn:path, $e:expr, $type_name:expr) =>
    ({
        let expected = $unwrap_fn($e);
        expected.unwrap_or_else(|| panic!("expected {}, got {:?}", $type_name, $e))
    });
}

impl Value {
    fn new_with(data: ValueData) -> Self {
        Value { val_ptr: Rc::new(data) }
    }

    pub fn new_bool(x: bool) -> Self { Self::new_with(ValueData::Bool(x)) }
    pub fn new_char(x: char) -> Self { Self::new_with(ValueData::Char(x)) }
    pub fn new_integer(x: i64) -> Self { Self::new_with(ValueData::Integer(x)) }
    pub fn new_symbol<'a, T: 'a + Into<Cow<'a, str>>>(s: T) -> Self { Self::new_with(ValueData::Symbol(s.into().into_owned())) }
    pub fn new_string<'a, T: 'a + Into<Cow<'a, str>>>(x: T) -> Self { Self::new_with(ValueData::String(x.into().into_owned())) }
    pub fn new_pair(a: Value, b: Value) -> Self { Self::new_with(ValueData::Pair(a,b)) }
    pub fn empty_list() -> Self { Self::new_with(ValueData::EmptyList) }
    pub fn new_native_proc(f: fn(&mut Interpreter, &mut [Value]) -> Value) -> Self {
        let raw: *const () = f as *const ();
        Self::new_with(ValueData::NativeProc(raw))
    }

    fn data(&self) -> &ValueData {
        &*self.val_ptr
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

    pub fn is_empty_list(&self) -> bool {
        self.get_empty_list().is_some()
    }

    pub fn get_empty_list(&self) -> Option<()> {
        if let &ValueData::EmptyList = self.data() { Some(()) } else { None }
    }

    pub fn get_symbol(&self) -> Option<&str> {
        match self.data() {
            &ValueData::Symbol(ref s) => Some(&s),
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

    pub fn get_fn_ptr(&self) -> Option<fn(&mut Interpreter, &mut [Value]) -> Value> {
        match self.data() {
            &ValueData::NativeProc(f) => Some(unsafe { mem::transmute(f) }),
            _ => None,
        }
    }

    pub fn from_char_to_integer(a: &Value) -> Value {
        let c = check_type!(Value::get_char, a, "char");
        Value::new_integer(c as i64)
    }

    pub fn from_integer_to_char(a: &Value) -> Value {
        let i = check_type!(Value::get_integer, a, "integer");
        let c = char::from_u32(i as u32);
        if i > char::MAX as i64 || c.is_none() {
            panic!("{} is no valid char", i);
        } else {
            Value::new_char(c.unwrap())
        }
    }

    pub fn from_number_to_string(a: &Value) -> Value {
        let i = check_type!(Value::get_integer, a, "integer");
        Value::new_string(format!("{}", i))
    }

    pub fn from_string_to_number(a: &Value) -> Value {
        let s = check_type!(Value::get_string, a, "string");
        let number = grammar::parse(s);
        number.unwrap_or_else(|_| panic!("{} is no valid integer", s))
    }

    pub fn from_string_to_symbol(a: &Value) -> Value {
        let s = check_type!(Value::get_string, a, "string");
        Value::new_symbol(s.clone())
    }

    pub fn from_symbol_to_string(a: &Value) -> Value {
        let s = check_type!(Value::get_symbol, a, "symbol");
        Value::new_string(s.clone())
    }

    pub fn new_list(elements: &[Value]) -> Value {
        if elements.len() == 0 { return Value::empty_list(); }
        let mut iter = elements.into_iter().rev();
        let last = iter.next().unwrap(); // safe because list len must be >= 1
        iter.fold(Value::new_pair(last.clone(), Value::empty_list()), |prev_pair, value| Value::new_pair(value.clone(), prev_pair))
    }
}

pub fn intern_symbol(ident: &str) -> u64 {
    let mut s = SipHasher::new();
    ident.hash(&mut s);
    s.finish()
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
enum ValueData {
    Bool(bool),
    Char(char),
    Integer(i64),
    Symbol(String),
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
            &ValueData::Symbol(ref s) => write!(f, "{}", s),
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
    use super::Value;

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
