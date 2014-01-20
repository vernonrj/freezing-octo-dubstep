use functypes::{RustFunc, BoundFn};

#[deriving(Clone, Eq)]
pub enum Element {
    Symbol(~str),
    Number(i64),
    String(~str),
    Character(char),
    Boolean(bool),
    ParseError(~str),
    EvalError(~str),
    List(~[Element]),
    Vec(~[Element]),
    Function(~BoundFn),
    FuncPrimitive(~RustFunc),
    nil
}

impl ToStr for Element {
    fn to_str(&self) -> ~str {
        match self.clone() {
            Symbol(s) => s.clone(),
            Number(n) => n.to_str(),
            String(s) => format!("\"{:s}\"", s),
            Character(c) => c.to_str(),
            Boolean(b) => b.to_str(),
            ParseError(p) => format!("Parse Error: {:s}", p),
            EvalError(e) => format!("Eval Error: {:s}", e),
            List(l) => {
                let form = l.iter().fold(~"", |a, b| a + " " + b.to_str());
                ~"(" + form.trim() + ")"
            },
            Vec(v) => {
                let form = v.iter().fold(~"", |a, b| a + " " + b.to_str());
                ~"[" + form.trim() + "]"
            },
            Function(f) => f.to_str(),
            FuncPrimitive(f) => f.to_str(),
            _ => format!("{:?}", self)
        }
    }
}
