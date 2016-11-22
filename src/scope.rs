use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use siphasher::sip::SipHasher24 as SipHasher;

use super::value::Value;

#[derive(Debug)]
pub struct Scope {
    idents: HashMap<u64, Value>,
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

    pub fn lookup_ident(&self, ident: &str) -> Option<Value> {
        let id = intern_ident(ident);
        self.lookup_id(id)
    }

    fn lookup_id(&self, id: u64) -> Option<Value> {
        self.idents.get(&id).cloned()
        .or_else(|| self.parent.as_ref().and_then(|p| p.lookup_id(id)))
    }

    pub fn add_ident(&mut self, ident: &str, value: Value) {
        let id = intern_ident(ident);
        self.idents.insert(id, value);
    }
}

pub fn intern_ident(ident: &str) -> u64 {
    let mut s = SipHasher::new();
    ident.hash(&mut s);
    s.finish()
}
