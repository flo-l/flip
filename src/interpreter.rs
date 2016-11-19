use super::value::{Value, ValueData};
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use siphasher::sip::SipHasher24 as SipHasher;

#[derive(Debug)]
struct Scope {
    idents: BTreeMap<u64, Value>,
}

impl Scope {
    fn new() -> Self{
        Scope {
            idents: BTreeMap::new()
        }
    }

    fn lookup_ident(&self, id: u64) -> Option<Value> {
        self.idents.get(&id).cloned()
    }

    fn add_ident(&mut self, id: u64, value: Value) {
        self.idents.insert(id, value);
    }

    fn idents(&self) -> &BTreeMap<u64, Value> {
        &self.idents
    }
}

pub struct Interpreter {
    starting_point: Option<Value>,
    scopes: Vec<Scope>,
}

impl Interpreter {
    pub fn new(value: Value) -> Self {
        Interpreter{
            starting_point: Some(value),
            scopes: vec![],
        }
    }

    pub fn start(&mut self) -> Value {
        self.init();
        let mut value = self.starting_point.take().unwrap();
        self.evaluate(&mut value)
    }

    fn init(&mut self) {
        self.scopes.push(Scope::new());
        self.add_ident("+", Value::new_native_plus());
        self.add_ident("define", Value::new_native_define());
    }

    fn evaluate(&mut self, value: &Value) -> Value {
        match value.data() {
            &ValueData::List(ref vec) => {
                if vec.len() == 0 { panic!("Tried to evaluate empty list") }
                let (first, rest) = vec.split_at(1);

                let first = self.evaluate(&first[0]);
                let rest = rest.iter();
                self.call(&first, rest)
            }
            &ValueData::Ident(ref ident) => self.lookup_ident(ident).clone(),
            _ => value.clone(),
        }
    }

    fn call<'a, I>(&mut self, value: &Value, args: I) -> Value
    where I: 'a + Iterator<Item=&'a Value>,
    {
        match value.data() {
            &ValueData::NativePlus => {
                let sum = args
                .map(|value| self.evaluate(&value))
                .fold(0, |acc, i|{
                    match i.data() {
                        &ValueData::Integer(x) => acc + x,
                        x => panic!("Tried to sum {:?}", x),
                    }
                });
                Value::new_integer(sum)
            },
            &ValueData::NativeDefine => {
                println!("hi");
                let (ident, expr) = {
                    let mut args = args.map(|value| self.evaluate(&value));
                    let ident = match args.next().map(|x| x.into_data()) {
                        Some(ValueData::Ident(s)) => s,
                        Some(value) => panic!("expected: (define <ident> <expr>), got {} as <ident>", value),
                        None => panic!("expected: (define <ident> <expr>), missing <ident> and <expr>")
                    };
                    let expr = match args.next() {
                        Some(value) => value,
                        None => panic!("expected: (define <ident> <expr>), missing <expr>")
                    };
                    let v: Vec<Value> = args.collect();
                    if v.len() != 0 {
                        panic!("expected: (define <ident> <expr>), got too many arguments: {:?}", v)
                    }
                    (ident, expr)
                };

                self.add_ident(&ident, expr.clone());
                expr
            }
            x => panic!("Tried to call {:?}", x),
        }
    }

    fn intern_ident(&self, ident: &str) -> u64 {
        let mut s = SipHasher::new();
        ident.hash(&mut s);
        s.finish()
    }

    fn lookup_ident(&self, ident: &str) -> Value {
        let id = self.intern_ident(ident);
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.lookup_ident(id) {
                return value.clone();
            }
        }
        println!("Searched Ident: {:?}", id);
        println!("Known Idents: ");
        for x in &self.scopes {
            for (hash, value) in &x.idents {
                println!("({:?}) => {:?}", hash, value);
            }
        }
        panic!("Ident: {:?} not found.", ident);
    }

    fn add_ident(&mut self, ident: &str, value: Value) {
        let id = self.intern_ident(ident);
        self.scopes.last_mut().unwrap().add_ident(id, value);
    }
}
