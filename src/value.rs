use std::fmt;
use std::rc::Rc;

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
    pub fn new_ident(x: String) -> Self { Self::new_with(ValueData::Ident(x)) }
    pub fn new_list(x: Vec<Value>) -> Self { Self::new_with(ValueData::List(x)) }
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
