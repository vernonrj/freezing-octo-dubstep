use std::vec;

use std::ptr;
use std::c_str::CString;
use std::libc::c_char;


#[deriving(Clone, Eq)]
enum Element {
    Symbol(~str),
    Number(i64),
    String(~str),
    Character(char),
    Boolean(bool),
    ParseError(~str),
    EvalError(~str),
    List(~[Element]),
    Vec(~[Element]),
    nil
}

fn tokenize_firstpass(s: &str) -> Option<~[~str]>
{
    let mut v: ~[~str] = ~[];
    let mut index = 0;
    let mut tok_start = 0;
    let mut inside_string = false;
    let mut stringbuilder = ~"";
    let ss = s.trim().replace(",", " ");
    while index < ss.len() {
        let c: char = ss.char_at(index);
        if "()[] ".contains(c.to_str()) && !inside_string {
            if index != tok_start {
                v.push(ss.slice(tok_start, index).to_owned());
            }
            if c != ' ' {
                v.push(c.to_str());
            }
            tok_start = index + 1;
        } else if c == '"' {
            stringbuilder.push_char('"');
            if inside_string {
                v.push(stringbuilder);
                stringbuilder = ~"";
                tok_start = index + 1;
            }
            inside_string = !inside_string;
        } else if inside_string {
            stringbuilder.push_char(c);
        }
        index += 1;
    }
    if inside_string {
        return None;
    }
    if tok_start != index {
        v.push(ss.slice(tok_start, index).to_owned());
    }
    return Some(v);
}

fn do_tokenize_structure(tokens: &[~str], start_index: uint, num_parens: uint) -> (uint, Element)
{
    let mut v: ~[Element] = ~[];
    let mut index = start_index;
    while index < tokens.len() {
        let token = tokens[index].clone();
        if token == ~"(" || token == ~"[" {
            // indent
            let close_paren = match token {
                ~"(" => ~")",
                ~"[" => ~"]",
                _ => return (tokens.len(), ParseError(~"unknown parenthesis open type"))
            };
            let (next_index, elem) = do_tokenize_structure(tokens, index+1, num_parens+1);
            v.push(elem);
            index = next_index;
            if index >= tokens.len() {
                break;
            } else if tokens[index] != close_paren {
                return (tokens.len(), ParseError(~"unmatched parentheses"));
            }
        } else if token == ~")" || token == ~"]" {
            // outdent
            if num_parens <= 0 {
                return (tokens.len(), ParseError(~"unbalanced parentheses"));
            }
            let elem_type = match token {
                ~"]" => Vec,
                ~")" => List,
                _ => fail!("unknown close brace")
            };
            return (index, elem_type(v));
        } else {
            // another element
            if token.starts_with("\"") && token.ends_with("\"") {
                v.push(String(token.slice(1, token.len()-1).to_owned()));
            } else {
                v.push(Symbol(token.to_owned()));
            }
        }
        index += 1;
    }
    if num_parens > 0 {
        return (tokens.len(), ParseError(~"unbalanced parentheses"));
    }
    match v.len() {
        0 => (index, nil),
        1 => (index, v[0]),
        _ => (index, List(v))
    }
}


fn tokenize_structure(tokens: &[~str]) -> Element
{
    let (_, elem) = do_tokenize_structure(tokens, 0, 0);
    return elem;
}


fn tokenize_infer_types(token: Element) -> Element
{
    match token {
        List(l) => {
            let mut v: ~[Element] = ~[];
            if l.len() > 0 {
                v.push(l[0].clone());
                let rest = l.slice_from(1);
                for elem in rest.iter() {
                    v.push(tokenize_infer_types(elem.clone()));
                }
            }
            List(v)
        },
        Symbol(s) => {
            match from_str::<i64>(s) {
                Some(i) => Number(i),
                None => EvalError(~"value cannot be converted to number")
            }
        },
        Vec(s) => {
            let mut v: ~[Element] = ~[];
            for elem in s.iter() {
                v.push(tokenize_infer_types(elem.clone()));
            }
            Vec(v)
        }
        _ => token
    }
}

#[allow(dead_code)]
fn tokenize(s: &str) -> Element
{
    let maybe_tokenized: Option<~[~str]> = tokenize_firstpass(s);
    let tokenized = match maybe_tokenized {
        Some(ss) => ss,
        None => return ParseError(~"unbalanced string quotes")
    };
    let elems = tokenize_structure(tokenized);
    match elems {
        ParseError(_) => return elems,
        _ => ()
    }
    return tokenize_infer_types(elems);
}


fn unwrap_to_nums(list: &[Element]) -> Option<~[i64]>
{
    let is = list.map(|x| {
        match x {
            &Number(ref s) => Some(*s),
            _ => None
        }
    });
    if is.iter().any(|x| { x.is_none() }) {
        return None;
    } else {
        return Some(is.map(|x| { x.clone().unwrap() }));
    }
}

fn add(list: &[Element]) -> Element
{
    let vals: Option<~[i64]> = unwrap_to_nums(list);
    match vals {
        Some(is) => {
            let sum: i64 = is.iter().fold(0, |a, &b| {
                a + b
            });
            Number(sum)
        },
        None => ParseError(~"+: invalid value")
    }
}

fn sub(list: &[Element]) -> Element
{
    let vals: Option<~[i64]> = unwrap_to_nums(list);
    match vals {
        Some(is) => {
            match is.len() {
                0 => EvalError(~"-: Wrong number of args (0)"),
                1 => {
                    let subbed = 0 - is[0];
                    Number(subbed)
                },
                _ => {
                    let first = is[0];
                    let tail = is.slice_from(1);
                    let subbed = tail.iter().fold(first, |a, &b| {
                        a - b
                    });
                    Number(subbed)
                }
            }
        },
        None => ParseError(~"-: invalid value")
    }
}

fn mul(list: &[Element]) -> Element
{
    let vals: Option<~[i64]> = unwrap_to_nums(list);
    match vals {
        Some(is) => {
            match is.len() {
                0 => Number(1),
                _ => {
                    let muld = is.iter().fold(1, |a, &b| {
                        a * b
                    });
                    Number(muld)
                }
            }
        },
        None => ParseError(~"*: invalid value")
    }
}

fn div(list: &[Element]) -> Element
{
    let vals: Option<~[i64]> = unwrap_to_nums(list);
    match vals {
        Some(is) => {
            match is.len() {
                0 => EvalError(~"/: Wrong number of args (0)"),
                1 => {
                    if is[0] == 0 {
                        return EvalError(~"/: Divide by zero");
                    }
                    let divd = 1 / is[0];
                    Number(divd)
                },
                _ => {
                    let first = is[0];
                    let tail = is.slice_from(1);
                    let mut zeros = tail.iter().filter(|&a| *a == 0);
                    if zeros.len() > 0 {
                        return EvalError(~"/: Divide by zero");
                    }
                    let divd = tail.iter().fold(first, |a, &b| {
                        a / b
                    });
                    Number(divd)
                }
            }
        },
        None => ParseError(~"/: invalid value")
    }
}

fn modfn(list: &[Element]) -> Element
{
    let vals: Option<~[i64]> = unwrap_to_nums(list);
    match vals {
        Some(is) => {
            let isl = is.len();
            match isl {
                2 => {
                    let first = is[0];
                    let second = is[1];
                    if second == 0 {
                        return EvalError(~"%: Divide by zero");
                    }
                    let modd = first % second;
                    Number(modd)
                },
                _ => EvalError(format!("%: Wrong number of args ({:u})", isl))
            }
        },
        None => ParseError(~"%: invalid value")
    }
}

fn concat(more: &[Element]) -> Element
{
    let mut unwrapped: ~[~[Element]] = ~[];
    for elem in more.iter() {
        unwrapped.push(match elem {
            &List(ref s) => s.to_owned(),
            &Vec(ref s) => s.to_owned(),
            &String(ref s) => s.chars().map(|x| Character(x)).collect(),
            _ => return EvalError(~"not a concatable collection type")
        });
    }
    let mut coll: ~[Element] = ~[];
    for &ref elem in unwrapped.iter() {
        coll = vec::append(coll, *elem);
    }
    List(coll)
}

fn equal(list: &[Element]) -> Element
{
    let list_len = list.len();
    match list_len {
        0 => return EvalError(format!("=: wrong number of args ({:u}) passed", list_len)),
        1 => return Boolean(true),
        _ => ()
    }
    let first: Element = list[0].clone();
    Boolean(list.slice_from(1).iter().all(|x| x.clone() == first))
}

//fn if_fn(list: &[Element]) -> Element
//{
//    let list_len = list.len();
//    if list_len > 3 || list_len < 2 {
//        return EvalError(format!("if: wrong number of args ({:u})", list_len));
//    }
//}

fn eval_top(list: ~[Element]) -> Element
{
    if list.len() < 1 {
        return List(list.to_owned());
    }
    let vals: ~[Element] = list.slice_from(1).to_owned();
    let vals_expanded = vals.map(|x| do_eval(x.clone()));
    match list[0] {
        Symbol(~"+") => add(vals_expanded),
        Symbol(~"-") => sub(vals_expanded),
        Symbol(~"*") => mul(vals_expanded),
        Symbol(~"/") => div(vals_expanded),
        Symbol(~"%") => modfn(vals_expanded),
        Symbol(~"=") => equal(vals_expanded),
        Symbol(~"concat") => concat(vals_expanded),
        List(l) => do_eval(List(l)),
        _ => ParseError(~"Unrecognized operation")
    }
}

fn do_eval(list: Element) -> Element
{
    match list {
        List(l) => eval_top(l),
        _ => list
    }
}


fn eval(s: &str) -> Element
{
    let parsed = tokenize(s);
    match parsed {
        List(_) => do_eval(parsed),
        _ => parsed
    }
}


#[link(name = "readline")]
#[allow(dead_code)]
extern {
    fn readline(prompt: *c_char) -> *c_char;
}

#[allow(dead_code)]
fn main()
{
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
                let parsed = eval(s);
                println!("{:?}", parsed);
            },
            None => return
        }
    }
}




#[test]
fn test_tokenizer_firstpass() {
    // empty
    assert!(tokenize_firstpass("") == Some(~[]));
    // stripping elements
    assert!(tokenize_firstpass(",") ==  Some(~[]));
    // single elements
    assert!(tokenize_firstpass("1") == Some(~[~"1"]));
    assert!(tokenize_firstpass("()") == Some(~[~"(", ~")"]));
    assert!(tokenize_firstpass("(1)") == Some(~[~"(", ~"1", ~")"]));
    // multiple elements
    assert!(tokenize_firstpass("1 2") == Some(~[~"1", ~"2"]));
    assert!(tokenize_firstpass("+ 1 2") == Some(~[~"+", ~"1", ~"2"]));
    assert!(tokenize_firstpass("(+ 1 2)") == Some(~[~"(", ~"+", ~"1", ~"2", ~")"]));
    assert!(tokenize_firstpass(" (+ 1 2)") == Some(~[~"(", ~"+", ~"1", ~"2", ~")"]));
    assert!(tokenize_firstpass("( + 1 2)") == Some(~[~"(", ~"+", ~"1", ~"2", ~")"]));
    assert!(tokenize_firstpass("(+ 1 (+ 2 3))") == Some(~[~"(", ~"+", ~"1",
                                                        ~"(", ~"+", ~"2", ~"3",
                                                        ~")", ~")"]));
    // vectors
    assert!(tokenize_firstpass("[]") == Some(~[~"[", ~"]"]));
    assert!(tokenize_firstpass("[1 2]") == Some(~[~"[", ~"1", ~"2", ~"]"]));
    assert!(tokenize_firstpass("[1, 2]") == Some(~[~"[", ~"1", ~"2", ~"]"]));
    // strings
    assert!(tokenize_firstpass("\"\"") == Some(~[~"\"\""]));
    assert!(tokenize_firstpass("\"hello\"") == Some(~[~"\"hello\""]));
}

#[test]
fn test_tokenizer_structure() {
    let test1 = tokenize_structure([]);
    assert!(test1 == nil);
    let test2 = tokenize_structure([~"(", ~"+", ~")"]);
    assert!(test2 == List(~[Symbol(~"+")]));
    let test3 = tokenize_structure([~"(", ~"+", ~"1", ~"2", ~")"]);
    assert!(test3 == List(~[Symbol(~"+"), Symbol(~"1"), Symbol(~"2")]));
    let test4 = tokenize_structure([~"1"]);
    assert!(test4 == Symbol(~"1"));
    let test5 = tokenize_structure([~"\"hello\""]);
    assert!(test5 == String(~"hello"));
    let test6 = tokenize_structure([~"[", ~"]"]);
    assert!(test6 == Vec(~[]));
    let test7 = tokenize_structure([~"[", ~"1", ~"]"]);
    assert!(test7 == Vec(~[Symbol(~"1")]));
}


#[test]
fn test_tokenizer_structure_errors() {
    let test1 = tokenize_structure([~"(", ~"+"]);
    match test1 {
        ParseError(_) => (),
        _ => fail!("{:?} != ParseError", test1)
    }
    let test2 = tokenize_structure([~"+", ~"1", ~"2", ~")"]);
    match test2 {
        ParseError(_) => (),
        _ => fail!("{:?} != ParseError", test2)
    }
    let test3 = tokenize_structure([~"[", ~"1"]);
    match test3 {
        ParseError(_) => (),
        _ => fail!("{:?} != ParseError", test3)
    }
    let test4 = tokenize_structure([~"1", ~"]"]);
    match test4 {
        ParseError(_) => (),
        _ => fail!("{:?} != ParseError", test4)
    }
    let test5 = tokenize_structure([~"(", ~"+", ~"3", ~"4", ~"]"]);
    match test5 {
        ParseError(_) => (),
        _ => fail!("{:?} != ParseError", test5)
    }
    let test6 = tokenize_structure([~")", ~"+", ~"5", ~"6", ~"("]);
    match test6 {
        ParseError(_) => (),
        _ => fail!("{:?} != ParseError", test6)
    }
}

#[test]
fn test_tokenizer_inference() {
    assert!(tokenize_infer_types(nil) == nil);
    assert!(tokenize_infer_types(Symbol(~"1")) == Number(1));
    assert!(tokenize_infer_types(List(~[Symbol(~"+"), Symbol(~"1")]))
            == List(~[Symbol(~"+"), Number(1)]));
    assert!(tokenize_infer_types(String(~"hello"))
            == String(~"hello"));
    assert!(tokenize_infer_types(Vec(~[Symbol(~"1"), Symbol(~"2")]))
            == Vec(~[Number(1), Number(2)]));
}

#[test]
fn test_tokenizer() {
    assert!(tokenize("") == nil);
    assert!(tokenize("(+ 1 1)") == List(~[Symbol(~"+"), Number(1), Number(1)]));
    assert!(tokenize("(- 5 1)") == List(~[Symbol(~"-"), Number(5), Number(1)]));
    assert!(tokenize("1") == Number(1));
    assert!(tokenize("\"hello\"") == String(~"hello"));
    assert!(tokenize("[1 2 3]") == Vec(~[Number(1), Number(2), Number(3)]));
}

#[test]
fn test_tokenizer_errors() {
    let test1 = tokenize("\"");
    match test1 {
        ParseError(_) => (),
        _ => fail!("{:?} != ParseError", test1)
    }
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
}

#[test]
fn test_add() {
    assert!(eval("(+)") == Number(0));
    assert!(eval("(+ 5)") == Number(5));
    assert!(eval("(+ 1 1)") == Number(2));
    assert!(eval("(+ 4 5 6)") == Number(15));
    assert!(eval("(+ 5 -1)") == Number(4));
}

#[test]
fn test_sub() {
    assert!(eval("(-)") == EvalError(~"-: Wrong number of args (0)"));
    assert!(eval("(- 1)") == Number(-1));
    assert!(eval("(- 1 1)") == Number(0));
    assert!(eval("(- 2 3)") == Number(-1));
    assert!(eval("(- 5 3)") == Number(2));
    assert!(eval("(- 9 5 2)") == Number(2));
    assert!(eval("(- 4 -2)") == Number(6));
}

#[test]
fn test_mul() {
    assert!(eval("(*)") == Number(1));
    assert!(eval("(* 2)") == Number(2));
    assert!(eval("(* 2 3)") == Number(6));
    assert!(eval("(* 2 0)") == Number(0));
    assert!(eval("(* 4 -1)") == Number(-4));
}

#[test]
fn test_div() {
    assert!(eval("(/)") == EvalError(~"/: Wrong number of args (0)"));
    assert!(eval("(/ 1)") == Number(1));
    assert!(eval("(/ 2)") == Number(0));
    assert!(eval("(/ 2 1)") == Number(2));
    assert!(eval("(/ 4 2)") == Number(2));
    assert!(eval("(/ 100 2 2 5)") == Number(5));
    assert!(eval("(/ 0)") == EvalError(~"/: Divide by zero"));
    assert!(eval("(/ 10 0)") == EvalError(~"/: Divide by zero"));
}

#[test]
fn test_mod() {
    assert!(eval("(%)") == EvalError(~"%: Wrong number of args (0)"));
    assert!(eval("(% 1)") == EvalError(~"%: Wrong number of args (1)"));
    assert!(eval("(% 1 2 3)") == EvalError(~"%: Wrong number of args (3)"));
    assert!(eval("(% 1 0)") == EvalError(~"%: Divide by zero"));
    assert!(eval("(% 1 1)") == Number(0));
    assert!(eval("(% 10 1)") == Number(0));
    assert!(eval("(% 10 7)") == Number(3));
    assert!(eval("(% 10 -3)") == Number(1));
    assert!(eval("(% -10 3)") == Number(-1));
}

#[test]
fn test_concat() {
    assert!(eval("(concat [1] [2])") == List(~[Number(1), Number(2)]));
    assert!(eval("(concat \"ab\" \"cd\")") == List(~[Character('a'),
                                                     Character('b'),
                                                     Character('c'),
                                                     Character('d')]));
}

#[test]
fn test_equal() {
    assert!(eval("(= 1 1)") == Boolean(true));
    assert!(eval("(= 1 2)") == Boolean(false));
    assert!(eval("(= 1 \"1\")") == Boolean(false));
    assert!(eval("(= \"1\" \"1\")") == Boolean(true));
    assert!(eval("(= [1 2] [1 2])") == Boolean(true));
    assert!(eval("(= [1 2] [1 3])") == Boolean(false));
    assert!(eval("(= [1 2 3] [1 2])") == Boolean(false));
}
