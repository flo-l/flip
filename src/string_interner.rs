use std::collections::hash_map::{HashMap, Entry};
use std::borrow::Cow;
use std::hash::{Hash, Hasher};
use siphasher::sip::SipHasher24 as SipHasher;


pub struct StringInterner {
    map: HashMap<u64, String>,
}

impl StringInterner {
    pub fn new() -> Self {
        StringInterner { map: HashMap::new() }
    }

    pub fn intern<'a, T: 'a + Into<Cow<'a, str>>>(&mut self, s: T) -> u64 {
        let string = s.into();
        let mut h = SipHasher::new();
        string.hash(&mut h);
        let id = h.finish();
        match self.map.entry(id) {
            Entry::Occupied(o) => debug_assert!(o.get() == &string),
            Entry::Vacant(o) => { o.insert(string.into_owned()); },
        }
        id
    }

    pub fn lookup(&self, id: u64) -> Option<&str> {
        self.map.get(&id).map(|x| &**x)
    }
}
