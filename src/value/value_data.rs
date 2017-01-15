use ::value::{Value, ListIter, Proc};
use ::string_interner::StringInterner;
use grammar::escape_char;
use itertools::Itertools;

#[derive(Debug, Clone, PartialEq)]
pub enum ValueData {
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
    pub fn to_string(&self, interner: &StringInterner) -> String {
        match self {
            &ValueData::Bool(x) => format!("{}", x),
            &ValueData::Char(x) => {
                if let Some(c) = escape_char(x) {
                    format!("#\\\\{}", c)                    
                } else {
                    format!("#\\{}", x)
                }
            },
            &ValueData::Integer(x) => format!("{}", x),
            &ValueData::Symbol(id) => format!("{}", interner.lookup(id).unwrap_or(&format!("[SYMBOL: {}]", id.to_string()))),
            &ValueData::String(ref x) => format!("\"{}\"", x),
            &ValueData::Pair(ref a, ref b) if b.get_empty_list().is_some() => {
                format!("({})", a.to_string(interner))
            },
            &ValueData::Pair(ref a, ref b) if b.get_pair().is_some() => {
                let rest = ListIter::new(b)
                .map(|x| x.to_string(interner))
                .join(" ");

                format!("({} {})", a.to_string(interner), rest)
            },
            &ValueData::Pair(ref a, ref b) => format!("({} . {})", a.to_string(interner), b.to_string(interner)),
            &ValueData::Condition(ref x) => format!("[CONDITION: {:?}]", x),
            &ValueData::EmptyList => format!("()"),
            &ValueData::NativeProc(x) => format!("[NATIVE_PROC: {:?}]", x),
            &ValueData::Proc(ref p) => format!("[PROC: {}]", p.to_string(interner)),
            &ValueData::Recur(ref p) => format!("[RECUR: {}]", Value::new_list(&p).to_string(interner)),
        }
    }
}
