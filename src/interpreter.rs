use super::ir::IR;
use std::collections::BTreeMap;
use std::hash::{Hash, SipHasher, Hasher};
use std::mem;

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
        self.scopes.push(Scope::new());
        self.add_ident("+", &IR::NativePlus);
        self.add_ident("define", &IR::NativeDefine);
    }

    fn evaluate(&mut self, ir: &IR) -> IR {
        match ir {
            &IR::List(ref vec) => {
                if vec.len() == 0 { panic!("Tried to evaluate empty list") }
                let (first, rest) = vec.split_at(1);

                let first = self.evaluate(&first[0]);
                let rest = rest.iter();
                self.call(&first, rest)
            }
            &IR::Ident(ref ident) => self.lookup_ident(ident).clone(),
            x => x.clone(),
        }
    }

    fn call<'a, I>(&mut self, ir: &IR, mut args: I) -> IR
    where I: 'a + Iterator<Item=&'a IR>,
    {
        match ir {
            &IR::NativePlus => {
                let sum = args
                .map(|ir| self.evaluate(&ir))
                .fold(0, |acc, i|{
                    match i {
                        IR::Integer(x) => acc + x,
                        x => panic!("Tried to sum {:?}", x),
                    }
                });
                IR::Integer(sum)
            },
            &IR::NativeDefine => {
                let (ident, expr) = {
                    let mut args = args.map(|ir| self.evaluate(&ir));
                    let ident = match args.next() {
                        Some(IR::Ident(s)) => s,
                        Some(ir) => panic!("expected: (define <ident> <expr>), got {} as <ident>", ir),
                        None => panic!("expected: (define <ident> <expr>), missing <ident> and <expr>")
                    };
                    let expr = match args.next() {
                        Some(ir) => ir,
                        None => panic!("expected: (define <ident> <expr>), missing <expr>")
                    };
                    let v: Vec<IR> = args.collect();
                    if v.len() != 0 {
                        panic!("expected: (define <ident> <expr>), got too many arguments: {:?}", v)
                    }
                    (ident, expr)
                };

                self.add_ident(&ident, &expr);
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

    fn lookup_ident(&self, ident: &str) -> &IR {
        let id = self.intern_ident(ident);
        for scope in self.scopes.iter().rev() {
            if let Some(ir) = scope.lookup_ident(id) {
                return ir;
            }
        }
        panic!("Ident: {:?} not found.", ident);
    }

    fn add_ident(&mut self, ident: &str, ir: &IR) {
        let id = self.intern_ident(ident);
        self.scopes.last_mut().unwrap().add_ident(id, ir.clone());
    }
}
