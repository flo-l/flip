use super::ir::IR;
use std::collections::BTreeMap;
use std::hash::{Hash, SipHasher, Hasher};

#[derive(Debug)]
struct Scope {
    idents: BTreeMap<u64, IR>,
}

impl Scope {
    fn new() -> Self{
        Scope {
            idents: BTreeMap::new()
        }
    }

    fn lookup_ident(&self, id: u64) -> Option<&IR> {
        self.idents.get(&id)
    }

    fn add_ident(&mut self, id: u64, ir: IR) {
        self.idents.insert(id, ir);
    }
}

pub struct Interpreter {
    ir: Option<IR>,
    scopes: Vec<Scope>,
}

impl Interpreter {
    pub fn new(ir: IR) -> Self {
        Interpreter{
            ir: Some(ir),
            scopes: vec![],
        }
    }

    pub fn start(&mut self) -> IR {
        self.init();
        let mut ir = self.ir.take().unwrap();
        self.evaluate(&mut ir)
    }

    fn init(&mut self) {
        let mut scope = Scope::new();
        scope.add_ident(self.intern_ident("+"), IR::NativePlus);
        self.scopes.push(scope);
    }

    fn evaluate(&mut self, ir: &IR) -> IR {
        match ir {
            &IR::List(ref vec) => {
                if vec.len() == 0 {
                    panic!("tried to evaluate empty list");
                }

                let (first, rest) = vec.split_at(1);
                match &first[0] {
                    &IR::Ident(ref s) => { self.lookup_ident(s).call(rest) },
                    x => x.call(rest),
                }
            }
            x => x.clone(),
        }
    }

    fn intern_ident(&self, ident: &str) -> u64 {
        let mut s = SipHasher::new();
        ident.hash(&mut s);
        s.finish()
    }

    fn lookup_ident(&self, ident: &str) -> &IR {
        let id = self.intern_ident(ident);
        for scope in self.scopes.iter().rev() {
            if let Some(ir) = scope.lookup_ident(id) {
                return ir;
            }
        }
        panic!("Ident: {:?} not found.", ident);
    }
}

impl IR {
    fn call(&self, args: &[IR]) -> IR {
        match self {
            &IR::NativePlus => {
                let sum = args.iter().fold(0, |acc, i|{
                    match i {
                        &IR::Integer(x) => acc + x,
                        x => panic!("Tried to sum {:?}", x),
                    }
                });
                IR::Integer(sum)
            },
            x => panic!("Tried to call {:?}", x),
        }
    }
}
