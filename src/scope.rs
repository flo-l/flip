use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::borrow::Cow;
use siphasher::sip::SipHasher24 as SipHasher;

use super::value::Value;

#[derive(Debug)]
pub struct Scope {
    idents: HashMap<u64, (String, Value)>,
    parent: Option<Box<Scope>>,
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            idents: HashMap::new(),
            parent: None,
        }
    }

    pub fn new_child<'b>(self) -> Self {
        Scope {
            idents: HashMap::new(),
            parent: Some(Box::new(self)),
        }
    }

    pub fn lookup_ident(&self, id: u64) -> Option<&Value> {
        self.idents.get(&id)
        .map(|&(ident, value)| &value)
        .or_else(|| self.parent.as_ref().and_then(|p| p.lookup_id(id)))
    }

    pub fn add_ident<'a, T: 'a + Into<Cow<'a, str>>>(&mut self, ident: T, value: Value) {
        let ident = ident.into();
        let id = intern_ident(&ident);
        self.idents.insert(id, (ident.into_owned(), value));
    }
}

pub fn intern_ident(ident: &str) -> u64 {
    let mut s = SipHasher::new();
    ident.hash(&mut s);
    s.finish()
}
