mod linked_list;

use std::collections::hash_map::HashMap;
use std::cell::RefCell;
use ::value::Value;
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

    pub fn lookup_symbol(&self, id: u64) -> Option<Value> {
        for scope_data in self.list.iter().map(RefCell::borrow) {
            match scope_data.lookup_symbol(id) {
                // has to clone because of refcell
                Some(v) => return Some(v.clone()),
                None => continue,
            }
        }
        None
    }

    pub fn add_symbol(&mut self, id: u64, value: Value) {
        self.list.head().expect("internal error: scope without data")
        .borrow_mut().bindings.insert(id, value);
    }

    pub fn symbol_ids<'a>(&'a self) -> Vec<u64> {
        let mut symbol_strings: Vec<u64> = vec![];
        for scope in self.list.iter().map(RefCell::borrow) {
            for id in scope.symbol_ids() {
                symbol_strings.push(id);
            }
        }
        symbol_strings
    }
}

#[derive(Debug, PartialEq)]
pub struct ScopeData {
    bindings: HashMap<u64, Value>,
}

impl ScopeData {
    fn new() -> Self {
        ScopeData {
            bindings: HashMap::new(),
        }
    }

    fn lookup_symbol(&self, id: u64) -> Option<&Value> {
        self.bindings.get(&id)
    }

    fn symbol_ids<'a>(&'a self) -> impl Iterator<Item=u64> + 'a {
        self.bindings.keys().map(|x| *x)
    }
}
