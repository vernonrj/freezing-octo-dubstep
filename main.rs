use std::ptr;
use std::c_str::CString;
use std::libc::c_char;
use tokenizer::tokenize;

use eval::Bindings;

mod primitive;
mod types;
mod tokenizer;
mod funcs;
mod eval;


#[link(name = "readline")]
#[allow(dead_code)]
extern {
    fn readline(prompt: *c_char) -> *c_char;
}

#[allow(dead_code)]
fn main()
{
    let bindings = Bindings::new();
    loop {
        let line = unsafe {
            let allocd: *c_char = readline(ptr::null());
            let read = CString::new(allocd, true);
            let read_s = read.as_str();
            match read_s {
                Some(s) => Some(s.to_owned()),
                None => None
            }
        };
        match line {
            Some(s) => {
                let parsed = tokenize(s);
                let evald = bindings.eval(parsed.clone());
                //let parsed = eval(bindings, line);
                println!("{:?}", evald);
            },
            None => return
        }
    }
}

