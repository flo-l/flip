use ::value::{Value, Proc, SpecialForm};
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
    EmptyList,
    List(Vec<Value>),
    Condition(Value),
    NativeProc(*const ()),
    Proc(Proc),
    Recur(Vec<Value>),
    SpecialForm(SpecialForm),
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
            &ValueData::Condition(ref x) => format!("[CONDITION: {:?}]", x),
            &ValueData::EmptyList => format!("()"),
            &ValueData::List(ref values) => format!("({})", values.iter().map(|v| v.to_string(interner)).join(" ")),
            &ValueData::NativeProc(x) => format!("[NATIVE_PROC: {:?}]", x),
            &ValueData::Proc(ref p) => format!("[PROC: {}]", p.to_string(interner)),
            &ValueData::Recur(ref p) => format!("[RECUR: {}]", Value::new_list(&p).to_string(interner)),
            &ValueData::SpecialForm(ref s) => s.to_string(interner),
        }
    }
}
