use std::fmt;

#[derive(Debug, Clone)]
pub enum IR {
    Bool(bool),
    Char(char),
    Integer(i64),
    Ident(String),
    List(Vec<IR>),
    NativePlus,
}

impl fmt::Display for IR {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &IR::Bool(x) => write!(f, "{}", x),
            &IR::Char(x) => write!(f, "{}", x),
            &IR::Integer(x) => write!(f, "{}", x),
            &IR::Ident(ref x) => write!(f, "{}", x),
            &IR::List(ref vec) => {
                let last = vec.len() - 1;
                let mut res = write!(f, "(");
                for x in &vec[..last] {
                    res = res.and(write!(f, "{} ", x));
                }
                res.and(write!(f, "{})", vec[last]))
            },
            &IR::NativePlus => write!(f, "[NATIVE_PROC]+"),
        }
    }
}
