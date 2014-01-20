extern mod extra;

use std::c_str::CString;
use std::libc::c_char;
use std::ptr;

use extra::getopts::{optflag,getopts};
use std::os;

use eval::Bindings;

mod primitives;
mod types;
mod functypes;
mod tokenizer;
mod eval;


#[link(name = "readline")]
#[allow(dead_code)]
extern {
    fn readline(prompt: *c_char) -> *c_char;
}


#[allow(dead_code)]
fn print_version(program: &str)
{
    println(format!("{:s} 0.1", program));
    println("Copyright (C) 2014 Vernon Jones, Bradon Kanyid");
    println("License GPLv3+: GNU GPL version 3 or later <http://gnu.org/licenses/gpl.html>.");
    println("This is free software: you are free to change and redistribute it.");
    println("There is NO WARRANTY, to the extent permitted by law.");
}


#[allow(dead_code)]
fn main()
{
    let mut bindings = Bindings::new();
    let args = os::args();
    let program = args[0].clone();
    let opts = ~[
        optflag("v"), optflag("version")];
    let matches = match getopts(args.tail(), opts) {
        Ok(m)  => { m },
        Err(f) => { fail!(f.to_err_msg()) }
    };
    if matches.opts_present([~"v", ~"version"]) {
        print_version(program);
        return;
    }
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
                let evald = bindings.eval(s);
                println!("{:?}", evald);
            },
            None => return
        }
    }
}

