use std::collections::hash_map::{HashMap, Iter};
use std::iter::Fuse;
use std::borrow::Cow;

use super::value::{Value, intern_symbol};

#[derive(Debug)]
pub struct Scope {
    defined_symbols: HashMap<u64, (String, Value)>,
    parent: Option<Box<Scope>>,
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            defined_symbols: HashMap::new(),
            parent: None,
        }
    }

    pub fn new_child<'b>(self) -> Self {
        Scope {
            defined_symbols: HashMap::new(),
            parent: Some(Box::new(self)),
        }
    }

    pub fn lookup_symbol_string(&self, s: &str) -> Option<&(String, Value)> {
        let id = intern_symbol(s);
        self.lookup_symbol(id)
    }

    pub fn lookup_symbol(&self, id: u64) -> Option<&(String, Value)> {
        self.defined_symbols.get(&id)
        .or_else(|| self.parent.as_ref().and_then(|p| p.lookup_symbol(id)))
    }

    pub fn add_symbol<'a, T: 'a + Into<Cow<'a, str>>>(&mut self, ident: T, value: Value) {
        let ident = ident.into();
        let id = intern_symbol(&ident);
        self.defined_symbols.insert(id, (ident.into_owned(), value));
    }
}

pub struct SymbolIterator<'a> {
    current_scope: Option<&'a Scope>,
    current_iter: Fuse<Iter<'a, u64, (String, Value)>>
}

impl<'a> SymbolIterator <'a> {
    pub fn new(scope: &'a Scope) -> Self {
        SymbolIterator {
            current_scope: Some(scope),
            current_iter: scope.defined_symbols.iter().fuse()
        }
    }
}

impl<'a> Iterator for SymbolIterator<'a> {
    type Item = (&'a u64, &'a (String, Value));

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_scope.is_none() {
            return None;
        }

        if let Some(x) = self.current_iter.next() {
            Some(x)
        } else {
            if let Some(ref scope) = self.current_scope.unwrap().parent {
                self.current_scope = Some(scope);
                self.current_iter = scope.defined_symbols.iter().fuse();
                self.current_iter.next()
            } else {
                self.current_scope = None;
                None
            }
        }
    }
}
