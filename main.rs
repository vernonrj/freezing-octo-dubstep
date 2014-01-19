
use std::io::buffered::BufferedReader;
use std::io::stdin;


use tokenizer::tokenize;

use eval::Bindings;

mod primitive;
mod types;
mod tokenizer;
mod funcs;
mod eval;


#[allow(dead_code)]
fn main()
{
    let bindings = Bindings::new();
    let mut stdin = BufferedReader::new(stdin());
    for line in stdin.lines() {
        let parsed = tokenize(line);
        let evald = bindings.eval(parsed.clone());
        //let parsed = eval(bindings, line);
        println!("{:?}", evald);
    }
}




