use std::hashmap::HashMap;
use std::iter::Iterator;

use tokenizer::tokenize;

use types::Element;
use types::Symbol;
use types::{Number, String, nil};
use types::{List, Vec};
use types::{Function, FuncPrimitive};
use types::EvalError;

use primitives::{add, sub, mul, div, modfn, equal, if_fn, concat};

use functypes::{RustFunc, BoundFn, Variable};


pub struct Bindings {
    bindings: ~[HashMap<~str, Element>]
}

impl Bindings {
    pub fn new() -> Bindings {
        let mut binding: HashMap<~str, Element> = HashMap::new();
        binding.insert(~"+", RustFunc::new(add));
        binding.insert(~"-", RustFunc::new(sub));
        binding.insert(~"*", RustFunc::new(mul));
        binding.insert(~"/", RustFunc::new(div));
        binding.insert(~"%", RustFunc::new(modfn));
        binding.insert(~"=", RustFunc::new(equal));
        binding.insert(~"concat", RustFunc::new(concat));
        binding.insert(~"if", RustFunc::new(if_fn));
        binding.insert(~"inc", BoundFn::new([~"x"], tokenize("(+ x 1)")));
        binding.insert(~"dec", BoundFn::new([~"x"], tokenize("(- x 1)")));
        Bindings { bindings: ~[binding] }
    }
    pub fn push(&self) -> Bindings {
        Bindings { bindings: ~[HashMap::new()] + self.bindings }
    }
    pub fn insert(&mut self, key: &str, value: Element) -> bool {
        self.bindings[0].insert(key.to_owned(), value)
    }
    pub fn get(&self, e: &str) -> Element {
        let s = e.to_owned();
        for map in self.bindings.iter() {
            if map.contains_key(&s) {
                return map.get(&s).clone();
            }
        }
        return EvalError(~"Not in scope");
    }
    pub fn contains_key(&self, e: &str) -> bool {
        for map in self.bindings.iter() {
            if map.contains_key(&e.to_owned()) {
                return true;
            }
        }
        return false;
    }
    fn eval_form(&self, form: &[Element]) -> Element
    {
        //println!("eval_form({:u}): {:?}", self.bindings.len(), form);
        let mut b = self.push();
        if form.len() == 0 {
            return List(form.to_owned());
        }
        let vals: ~[Element] = form.slice_from(1).to_owned();
        let vals_expanded = vals.map(|x| b.eval(x.clone()));
        //println!("eval({:u}): finished expanding: {:?}", self.bindings.len(), vals_expanded);
        match form[0] {
            Symbol(ref sym) => {
                // lookup in bindings
                if b.contains_key(sym.to_owned()) {
                    let bound = b.get(sym.to_owned()).clone();
                    //println!("eval({:u}): sym {:?} resolves to {:?}",
                    //        self.bindings.len(), sym, bound);
                    self.eval_form(~[bound] + vals_expanded)
                } else {
                    EvalError(~"Symbol Not defined")
                }
            },
            List(ref l) => {
                let newform = ~[b.eval_form(*l)] + vals_expanded;
                b.eval_form(newform)
            },
            FuncPrimitive(ref fptr) => {
                // lookup name, pass the rest of the list in
                let f = fptr.f;
                f(vals_expanded)
            },
            Function(ref fptr) => {
                for (arg, val) in fptr.bindings.iter().zip(vals.iter()) {
                    match arg {
                        &Variable(ref s) => b.insert(s.to_owned(), val.clone()),
                        _ => return EvalError(~"Variadic not implemented")
                    };
                }
                b.eval(fptr.f.clone())
            }
            _ => EvalError(~"Failed to evaluate form")

        }
    }
    pub fn eval(&self, form: Element) -> Element
    {
        //println!("eval({:u}): {:?}", self.bindings.len(), form);
        match form {
            List(l) => self.eval_form(l),
            Vec(v) => Vec(v.map(|x| self.eval(x.clone()))),
            Symbol(ref sym) => {
                // lookup in bindings
                if self.contains_key(sym.to_owned()) {
                    let bound = self.get(sym.to_owned()).clone();
                    //println!("eval({:u}): sym {:?} resolves to {:?}",
                    //        self.bindings.len(), sym, bound);
                    bound
                } else {
                    EvalError(~"Symbol Not defined")
                }
            }
            _ => form
        }
    }
}


#[allow(dead_code)]
pub fn eval(s: &str) -> Element
{
    let bindings = Bindings::new();
    let parsed = tokenize(s);
    bindings.eval(parsed)
}


#[test]
fn test_basic_eval() {
    assert!(eval("1") == Number(1));
    assert!(eval("1\n") == Number(1));
    assert!(eval("") == nil);
    assert!(eval("\n") == nil);
    assert!(eval("()") == List(~[]));
    assert!(eval("()\n") == List(~[]));
    assert!(eval("[]") == Vec(~[]));
    assert!(eval("\"\"") == String(~""));
    assert!(eval("\"test string\"") == String(~"test string"));
    assert!(eval("\"(+ 1 1)\"") == String(~"(+ 1 1)"));
    assert!(eval("[(+ 1 1)]") == List(~[Number(2)]));
}


