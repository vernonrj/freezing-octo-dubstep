use std::hashmap::HashMap;
use std::iter::Iterator;

use tokenizer::tokenize;

use types::Element;
use types::{Symbol, Boolean, nil};
use types::{List, Vec};
use types::{Function, FuncPrimitive};
use types::EvalError;

use primitives::{add, sub, mul, div, modfn, equal, concat};

use functypes::{RustFunc, BoundFn, Variable};

mod types;

#[allow(dead_code)]
pub struct Bindings {
    bindings: ~[HashMap<~str, Element>]
}

impl Bindings {
    #[allow(dead_code)]
    pub fn new() -> Bindings {
        let mut binding: HashMap<~str, Element> = HashMap::new();
        binding.insert(~"+", RustFunc::new(add));
        binding.insert(~"-", RustFunc::new(sub));
        binding.insert(~"*", RustFunc::new(mul));
        binding.insert(~"/", RustFunc::new(div));
        binding.insert(~"%", RustFunc::new(modfn));
        binding.insert(~"=", RustFunc::new(equal));
        binding.insert(~"concat", RustFunc::new(concat));
        //binding.insert(~"not", BoundFn::new_macro([~"x"], tokenize("(if x false true)")));
        //binding.insert(~"if-not", BoundFn::new_macro([~"test", ~"then", ~"else"],
        //    tokenize("(if (not test) then else)")));
        binding.insert(~"inc", BoundFn::new([~"x"], tokenize("(+ x 1)")));
        binding.insert(~"dec", BoundFn::new([~"x"], tokenize("(- x 1)")));
        Bindings { bindings: ~[binding] }
    }
    #[allow(dead_code)]
    pub fn push(&self) -> Bindings {
        Bindings { bindings: ~[HashMap::new()] + self.bindings }
    }
    #[allow(dead_code)]
    pub fn insert(&mut self, key: &str, value: Element) -> bool {
        self.bindings[0].insert(key.to_owned(), value)
    }
    #[allow(dead_code)]
    pub fn get(&self, e: &str) -> Element {
        let s = e.to_owned();
        for map in self.bindings.iter() {
            if map.contains_key(&s) {
                return map.get(&s).clone();
            }
        }
        return EvalError(~"Not in scope");
    }
    #[allow(dead_code)]
    pub fn contains_key(&self, e: &str) -> bool {
        for map in self.bindings.iter() {
            if map.contains_key(&e.to_owned()) {
                return true;
            }
        }
        return false;
    }
    #[allow(dead_code)]
    fn eval_form(&mut self, form: &[Element]) -> Element
    {
        //println!("eval_form({:u}): {:?}", self.bindings.len(), form);
        let mut b = self.push();
        if form.len() == 0 {
            return List(form.to_owned());
        }
        let vals: ~[Element] = form.slice_from(1).to_owned();
        //println!("eval({:u}): finished expanding: {:?}", self.bindings.len(), vals_expanded);
        match form[0] {
            Symbol(ref sym) => {
                // lookup in bindings
                let symclone = sym.clone();
                if symclone == ~"if" {
                    // if is a special case
                    self.if_fn(vals)
                } else if symclone == ~"def" {
                    // bind to toplevel
                    if vals.len() != 2 {
                        EvalError(~"expected 2 args")
                    } else {
                        let (name, form) = (vals[0].clone(), b.eval_elem(vals[1].clone()));
                        match name {
                            Symbol(s) => {
                                let toplevel = self.bindings.len() - 1;
                                self.bindings[toplevel].insert(s, form);
                                nil
                            },
                            _ => EvalError(~"first arg not of type symbol")
                        }
                    }
                } else if symclone == ~"fn" {
                    // create an fn
                    // TODO: when defmacro is defined, use that instead
                    if vals.len() != 2 {
                        EvalError(~"expected 2 args")
                    } else {
                        let (args_wrapped, form) = (vals[0].clone(), vals[1].clone());
                        let args_v: ~[Element] = match args_wrapped {
                            Vec(v) => v,
                            _ => return EvalError(~"args must be in a vector")
                        };
                        let mut args: ~[~str] = ~[];
                        for i in args_v.iter() {
                            match i.clone() {
                                Symbol(s) => args.push(s.clone()),
                                _ => return EvalError(~"args must be symbols")
                            }
                        }
                        BoundFn::new(args, form)
                    }
                } else if symclone == ~"defmacro" {
                    // create a macro
                    if vals.len() != 3 {
                        EvalError(~"expected 3 args")
                    } else {
                        let name = match vals[0].clone() {
                            Symbol(s) => s,
                            _ => return EvalError(~"name must be a symbol")
                        };
                        let args_v = match vals[1].clone() {
                            Vec(v) => v,
                            _ => return EvalError(~"args must be a vector")
                        };
                        let mut args: ~[~str] = ~[];
                        for i in args_v.iter() {
                            match i.clone() {
                                Symbol(s) => args.push(s.clone()),
                                _ => return EvalError(~"args must be symbols")
                            }
                        }
                        let form = vals[2].clone();
                        let toplevel = self.bindings.len() - 1;
                        self.bindings[toplevel].insert(name, BoundFn::new_macro(args, form));
                        nil
                    }
                } else if b.contains_key(sym.to_owned()) {
                    let bound = b.get(sym.to_owned()).clone();
                    //println!("eval({:u}): sym {:?} resolves to {:?}",
                    //        self.bindings.len(), sym, bound);
                    let vals_expanded = vals.map(|x| b.eval_elem(x.clone()));
                    self.eval_form(~[bound] + vals_expanded)
                } else {
                    EvalError(~"Symbol Not defined")
                }
            },
            List(ref l) => {
                let vals_expanded = vals.map(|x| b.eval_elem(x.clone()));
                let newform = ~[b.eval_form(*l)] + vals_expanded;
                b.eval_form(newform)
            },
            FuncPrimitive(ref fptr) => {
                // lookup name, pass the rest of the list in
                let vals_expanded = vals.map(|x| b.eval_elem(x.clone()));
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
                // TODO: different behavior for macros?
                b.eval_elem(fptr.f.clone())
            }
            _ => EvalError(~"Failed to evaluate form")

        }
    }
    #[allow(dead_code)]
    pub fn eval_elem(&mut self, form: Element) -> Element
    {
        //println!("eval({:u}): {:?}", self.bindings.len(), form);
        match form {
            List(l) => self.eval_form(l),
            Vec(v) => Vec(v.map(|x| self.eval_elem(x.clone()))),
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
    #[allow(dead_code)]
    pub fn eval(&mut self, s: &str) -> Element
    {
        let parsed = tokenize(s);
        self.eval_elem(parsed)
    }
    #[allow(dead_code)]
    fn if_fn(&mut self, list: &[Element]) -> Element
    {
        let list_len = list.len();
        if list_len > 3 || list_len < 2 {
            return EvalError(format!("if: wrong number of args ({:u})", list_len));
        }
        let rest = list.slice_from(1);
        match self.eval_elem(list[0].clone()) {
            Boolean(true) => self.eval_elem(rest[0].clone()),
            Boolean(false) if list_len > 2 => self.eval_elem(rest[1].clone()),
            Boolean(false) if list_len == 2 => nil,
            _ => EvalError(~"if: first element must be boolean")
        }
    }
}


#[allow(dead_code)]
pub fn eval(s: &str) -> Element
{
    let mut bindings = Bindings::new();
    bindings.eval(s)
}



#[test]
fn test_basic_eval() {
    assert!(eval("1") == ::types::Number(1));
    assert!(eval("1\n") == ::types::Number(1));
    assert!(eval("") == ::types::nil);
    assert!(eval("\n") == ::types::nil);
    assert!(eval("()") == List(~[]));
    assert!(eval("()\n") == List(~[]));
    assert!(eval("[]") == Vec(~[]));
    assert!(eval("\"\"") == ::types::String(~""));
    assert!(eval("\"test string\"") == ::types::String(~"test string"));
    assert!(eval("\"(+ 1 1)\"") == ::types::String(~"(+ 1 1)"));
    assert!(eval("[(+ 1 1)]") == Vec(~[::types::Number(2)]));
}

#[test]
fn test_if_fn() {
    assert!(::eval::eval("(if true 1 0)") == ::types::Number(1));
    assert!(::eval::eval("(if (= 5 5) 6 4)") == ::types::Number(6));
    assert!(::eval::eval("(if (= 5 4) 6 4)") == ::types::Number(4));
    assert!(::eval::eval("(if (= (+ 1 2) 3) true false)") == ::types::Boolean(true));
}


#[test]
fn test_def() {
    let mut bindings = Bindings::new();
    // basic assignment
    bindings.eval("(def a 5)");
    assert!(bindings.eval("a") == ::types::Number(5));
    assert!(bindings.eval("(+ a 5)") == ::types::Number(10));
    assert!(bindings.eval("(inc a)") == ::types::Number(6));
    // test eager evaluation of form
    bindings.eval("(def x (+ 5 6))");
    assert!(bindings.eval("x") == ::types::Number(11));
    // check for weird self-assign conditions
    bindings.eval("(def a (+ a 1))");
    assert!(bindings.eval("a") == ::types::Number(6));
}

#[test]
fn test_fn() {
    let mut bindings = Bindings::new();
    assert!(eval("((fn [x] (+ x 5)) 6)") == ::types::Number(11));
    bindings.eval("(def f (fn [x] (+ x 1)))");
    assert!(bindings.eval("(f 5)") == ::types::Number(6));
    // define factorial
    bindings.eval("(def fac (fn [x] (if (= x 0) 1 (* x (fac (dec x))))))");
    assert!(bindings.eval("(fac 5)") == ::types::Number(120));
}

