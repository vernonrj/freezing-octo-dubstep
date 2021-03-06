use std::rand::Rng;
use std::rand::os::OSRng;

use types::Element;
use types::{Function, FuncPrimitive};


pub struct RustFunc {
    f: fn(&[Element]) -> Element,
    tag: u64
}

impl RustFunc {
    pub fn new(f: fn (&[Element]) -> Element) -> Element {
        let mut rng = OSRng::new();
        FuncPrimitive(~RustFunc {
            f: f,
            tag: rng.gen::<u64>()
        })
    }

}

impl Eq for RustFunc {
    fn eq(&self, other: &RustFunc) -> bool {
        self.tag == other.tag
    }
}

impl Clone for RustFunc {
    fn clone(&self) -> RustFunc {
        RustFunc {
            f: self.f,
            tag: self.tag
        }
    }
}

impl ToStr for RustFunc {
    fn to_str(&self) -> ~str {
        format!("@builtin{:u}", self.tag)
    }
}

#[deriving(Clone, Eq)]
pub enum ArgBinding {
    Variable(~str),
    Variadic(~str)
}

impl ToStr for ArgBinding {
    fn to_str(&self) -> ~str {
        match self.clone() {
            Variable(s) => s,
            Variadic(s) => format!("&{:s}", s)
        }
    }
}

#[deriving(Clone, Eq)]
pub struct BoundFn {
    bindings: ~[ArgBinding],
    f: Element,
    is_macro: bool
}

impl BoundFn {
    fn create_fn(bindings: &[~str], func: Element, is_macro: bool) -> Element {
        let newbindings = bindings.map(|x| {
            Variable(x.to_owned())
        });
        Function(~BoundFn {
            bindings: newbindings,
            f: func,
            is_macro: is_macro
        })
    }
    pub fn new(bindings: &[~str], func: Element) -> Element {
        BoundFn::create_fn(bindings, func, false)
    }
    #[allow(dead_code)]
    pub fn new_macro(bindings: &[~str], func: Element) -> Element {
        BoundFn::create_fn(bindings, func, true)
    }
}

impl ToStr for BoundFn {
    fn to_str(&self) -> ~str {
        format!("(fn {:s} {:s})", self.bindings.to_str(), self.f.to_str())
    }
}


#[test]
fn test_boundfn_to_str() {
    let list = ~[::types::Symbol(~"+"), ::types::Symbol(~"x"), ::types::Number(1)];
    assert!(BoundFn::new([~"x"], ::types::List(list)).to_str() == ~"(fn [x] (+ x 1))");
}
