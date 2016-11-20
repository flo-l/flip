use std::fmt;
use std::rc::Rc;
use std::borrow::Cow;

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
    pub fn new_pair(a: Value, b: Value) -> Self { Self::new_with(ValueData::Pair(a,b)) }
    pub fn empty_list() -> Self { Self::new_with(ValueData::EmptyList) }
    pub fn new_list<'a, T: 'a + Into<Cow<'a, [Value]>>>(x: T) -> Self { Self::new_with(ValueData::List(x.into().into_owned())) }
    pub fn new_native_plus() -> Self { Self::new_with(ValueData::NativePlus) }
    pub fn new_native_define() -> Self { Self::new_with(ValueData::NativeDefine) }

    pub fn data(&self) -> &ValueData { &*self.val_ptr }
    // tries to move out data, clones if rc count is > 1
    pub fn into_data(self) -> ValueData {
        match Rc::try_unwrap(self.val_ptr) {
            Ok(x) => x,
            Err(rc) => (*rc).clone(),
        }
    }

    fn is_pair(&self) -> bool {
        if let &ValueData::Pair(_, _) = self.data() { true } else { false }
    }

    fn is_empty_list(&self) -> bool {
        if let &ValueData::EmptyList = self.data() { true } else { false }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.val_ptr.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueData {
    Bool(bool),
    Char(char),
    Integer(i64),
    Ident(String),
    Pair(Value, Value),
    EmptyList,
    List(Vec<Value>),
    NativePlus,
    NativeDefine,
}

impl fmt::Display for ValueData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &ValueData::Bool(x) => write!(f, "{}", x),
            &ValueData::Char(x) => write!(f, "{}", x),
            &ValueData::Integer(x) => write!(f, "{}", x),
            &ValueData::Ident(ref x) => write!(f, "{}", x),
            &ValueData::Pair(ref a, ref b) if b.is_pair() => {
                let mut current = b;
                let mut res = write!(f, "({}", a);

                loop {
                    if let &ValueData::EmptyList = current.data() {
                        res = res.and(write!(f, ")"));
                        return res;
                    }

                    if let &ValueData::Pair(ref x, ref y) = current.data() {
                        if y.is_pair() || y.is_empty_list() {
                            res = res.and(write!(f, " {}", x));
                            current = y;
                            continue;
                        } else {
                            res = res.and(write!(f, " {})", current));
                            return res;
                        }
                    }
                }
            },
            &ValueData::Pair(ref a, ref b) if b.is_empty_list() => write!(f, "({})", a),
            &ValueData::Pair(ref a, ref b) => write!(f, "({} . {})", a, b),
            &ValueData::EmptyList => write!(f, "()"),
            &ValueData::List(ref vec) => {
                let last = vec.len() - 1;
                let mut res = write!(f, "(");
                for x in &vec[..last] {
                    res = res.and(write!(f, "{} ", x));
                }
                res.and(write!(f, "{})", vec[last]))
            },
            &ValueData::NativePlus => write!(f, "[NATIVE_PROC]+"),
            &ValueData::NativeDefine => write!(f, "[NATIVE_PROC]define"),
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
