mod linked_list;

use std::collections::hash_map::HashMap;
use std::borrow::Cow;
use std::cell::RefCell;
use ::value::{Value, intern_symbol};
use self::linked_list::List;

// The list holds ScopeData structs, which store the actual data (say bindings etc. etc.)
// The front of the list is the lastly created ScopeData. The back is the last ScopeData that
// should be searched when trying to match a binding.
#[derive(Debug, Clone, PartialEq)]
pub struct Scope {
    list: List<RefCell<ScopeData>>,
}

impl Scope {
    pub fn new() -> Self {
        Scope::with_scope_data(RefCell::new(ScopeData::new()))
    }

    fn with_scope_data(scope_data: RefCell<ScopeData>) -> Self {
        Scope {
            list: List::new().append(scope_data),
        }
    }

    // creates a child scope linked with it's parents
    pub fn new_child(&self) -> Self {
        let fresh_data = RefCell::new(ScopeData::new());
        Scope {
            list: self.list.append(fresh_data)
        }
    }

    pub fn lookup_symbol_with_string(&self, s: &str) -> Option<Value> {
        let id = intern_symbol(s);
        self.lookup_symbol(id)
    }

    pub fn lookup_symbol<'a>(&'a self, id: u64) -> Option<Value> {
        for scope_data in self.list.iter().map(RefCell::borrow) {
            match scope_data.lookup_symbol(id) {
                Some(&(_, ref v)) => return Some(v.clone()),
                None => continue,
            }
        }
        None
    }

    pub fn add_symbol<'a, T: 'a + Into<Cow<'a, str>>>(&mut self, ident: T, value: Value) {
        let ident = ident.into();
        let id = intern_symbol(&ident);
        self.list.head().expect("internal error: scope without data")
        .borrow_mut().bindings.insert(id, (ident.into_owned(), value));
    }

    // horribly inefficient but for now that has to be enough
    // when symbols become numbers this will be fast again
    pub fn symbol_strings<'a>(&'a self) -> Vec<String> {
        let mut symbol_strings: Vec<String> = vec![];
        for scope in self.list.iter().map(RefCell::borrow) {
            for string in scope.symbols().map(|&(ref s, _)| s) {
                symbol_strings.push(string.clone());
            }
        }
        symbol_strings
    }
}

#[derive(Debug, PartialEq)]
pub struct ScopeData {
    bindings: HashMap<u64, (String, Value)>,
}

impl ScopeData {
    fn new() -> Self {
        ScopeData {
            bindings: HashMap::new(),
        }
    }

    fn lookup_symbol(&self, id: u64) -> Option<&(String, Value)> {
        self.bindings.get(&id)
    }

    fn symbols<'a>(&'a self) -> impl Iterator<Item=&'a (String, Value)> {
        self.bindings.iter().map(|(_, pair)| pair)
    }
}
