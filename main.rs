use std::io::buffered::BufferedReader;
use std::io::stdin;



#[deriving(Clone, Eq)]
enum Element {
    Symbol(~str),
    Number(~str),
    String(~str),
    ParseError(~str),
    List(~[Element]),
    Vec(~[Element]),
    nil
}

fn tokenize_firstpass(s: &str) -> ~[~str]
{
    let mut v: ~[~str] = ~[];
    let mut index = 0;
    let mut tok_start = 0;
    let mut inside_string = false;
    let mut stringbuilder = ~"";
    let ss = s.replace(",", " ");
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
    if tok_start != index {
        v.push(ss.slice(tok_start, index).to_owned());
    }
    return v;
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
            Number(s)
        },
        _ => token
    }
}

#[allow(dead_code)]
fn tokenize(s: &str) -> Element
{
    let tokenized: ~[~str] = tokenize_firstpass(s);
    let elems = tokenize_structure(tokenized);
    match elems {
        ParseError(_) => return elems,
        _ => ()
    }
    return tokenize_infer_types(elems);
}


fn unwrap_to_nums<T: FromStr + Clone>(list: &[Element]) -> Option<~[T]>
{
    let is = list.map(|x| {
        match x {
            &Number(ref s) => from_str::<T>(*s),
            _ => None
        }
    });
    if is.iter().any(|x| { x.is_none() }) {
        return None;
    } else {
        return Some(is.map(|x| { x.clone().unwrap() }));
    }
}

fn plus(list: &[Element]) -> Element
{
    let vals: Option<~[int]> = unwrap_to_nums(list);
    match vals {
        Some(is) => {
            let sum: int = is.iter().fold(0, |a, &b| {
                a + b
            });
            Number(sum.to_str())
        },
        None => ParseError(~"+: invalid value")
    }
}

fn sub(list: &[Element]) -> Element
{
    let vals: Option<~[int]> = unwrap_to_nums(list);
    match vals {
        Some(is) => {
            match is.len() {
                0 => Number(~"0"),
                _ => {
                    let first = is[0];
                    let tail = is.slice_from(1);
                    let subbed = first + tail.iter().fold(0, |a, &b| {
                        a - b
                    });
                    Number(subbed.to_str())
                }
            }
        },
        None => ParseError(~"-: invalid value")
    }
}

fn mul(list: &[Element]) -> Element
{
    let vals: Option<~[int]> = unwrap_to_nums(list);
    match vals {
        Some(is) => {
            match is.len() {
                0 => Number(~"0"),
                _ => {
                    let muld = is.iter().fold(1, |a, &b| {
                        a * b
                    });
                    Number(muld.to_str())
                }
            }
        },
        None => ParseError(~"*: invalid value")
    }
}

fn eval_top(list: ~[Element]) -> Element
{
    if list.len() < 1 {
        return List(list);
    }
    let vals: ~[Element] = list.slice_from(1).to_owned();
    let vals_expanded = vals.map(|x| do_eval(x.clone()));
    match list[0] {
        Symbol(~"+") => plus(vals_expanded),
        Symbol(~"-") => sub(vals_expanded),
        Symbol(~"*") => mul(vals_expanded),
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



#[allow(dead_code)]
fn main()
{
    let mut stdin = BufferedReader::new(stdin());
    for line in stdin.lines() {
        let parsed = eval(line);
        println(format!("{:?}", parsed));
    }
}




#[test]
fn test_tokenizer_firstpass() {
    // empty
    assert!(tokenize_firstpass("") == ~[]);
    // stripping elements
    assert!(tokenize_firstpass(",") == ~[]);
    // single elements
    assert!(tokenize_firstpass("1") == ~[~"1"]);
    assert!(tokenize_firstpass("()") == ~[~"(", ~")"]);
    assert!(tokenize_firstpass("(1)") == ~[~"(", ~"1", ~")"]);
    // multiple elements
    assert!(tokenize_firstpass("1 2") == ~[~"1", ~"2"]);
    assert!(tokenize_firstpass("+ 1 2") == ~[~"+", ~"1", ~"2"]);
    assert!(tokenize_firstpass("(+ 1 2)") == ~[~"(", ~"+", ~"1", ~"2", ~")"]);
    assert!(tokenize_firstpass(" (+ 1 2)") == ~[~"(", ~"+", ~"1", ~"2", ~")"]);
    assert!(tokenize_firstpass("( + 1 2)") == ~[~"(", ~"+", ~"1", ~"2", ~")"]);
    assert!(tokenize_firstpass("(+ 1 (+ 2 3))") == ~[~"(", ~"+", ~"1",
                                                     ~"(", ~"+", ~"2", ~"3",
                                                     ~")", ~")"]);
    // vectors
    assert!(tokenize_firstpass("[]") == ~[~"[", ~"]"]);
    assert!(tokenize_firstpass("[1 2]") == ~[~"[", ~"1", ~"2", ~"]"]);
    assert!(tokenize_firstpass("[1, 2]") == ~[~"[", ~"1", ~"2", ~"]"]);
    // strings
    assert!(tokenize_firstpass("\"\"") == ~[~"\"\""]);
    assert!(tokenize_firstpass("\"hello\"") == ~[~"\"hello\""]);
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
    assert!(tokenize_infer_types(Symbol(~"1")) == Number(~"1"));
    assert!(tokenize_infer_types(List(~[Symbol(~"+"), Symbol(~"1")]))
            == List(~[Symbol(~"+"), Number(~"1")]));
    assert!(tokenize_infer_types(String(~"hello"))
            == String(~"hello"));
}

#[test]
fn test_tokenizer() {
    assert!(tokenize("") == nil);
    assert!(tokenize("(+ 1 1)") == List(~[Symbol(~"+"), Number(~"1"), Number(~"1")]));
    assert!(tokenize("(- 5 1)") == List(~[Symbol(~"-"), Number(~"5"), Number(~"1")]));
    assert!(tokenize("1") == Number(~"1"));
    assert!(tokenize("\"hello\"") == String(~"hello"));
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
    assert!(eval("1") == Number(~"1"));
    assert!(eval("") == nil);
    assert!(eval("()") == List(~[]));
    assert!(eval("[]") == Vec(~[]));
    assert!(eval("\"\"") == String(~""));
}

#[test]
fn test_plus() {
    assert!(eval("(+)") == Number(~"0"));
    assert!(eval("(+ 5)") == Number(~"5"));
    assert!(eval("(+ 1 1)") == Number(~"2"));
    assert!(eval("(+ 4 5 6)") == Number(~"15"));
    assert!(eval("(+ 5 -1)") == Number(~"4"));
}

#[test]
fn test_sub() {
    assert!(eval("(-)") == Number(~"0"));
    assert!(eval("(- 1)") == Number(~"1"));
    assert!(eval("(- 1 1)") == Number(~"0"));
    assert!(eval("(- 2 3)") == Number(~"-1"));
    assert!(eval("(- 5 3)") == Number(~"2"));
    assert!(eval("(- 9 5 2)") == Number(~"2"));
    assert!(eval("(- 4 -2)") == Number(~"6"));
}

#[test]
fn test_mul() {
    assert!(eval("(*)") == Number(~"0"));
    assert!(eval("(* 2)") == Number(~"2"));
    assert!(eval("(* 2 3)") == Number(~"6"));
    assert!(eval("(* 2 0)") == Number(~"0"));
    assert!(eval("(* 4 -1)") == Number(~"-4"));
}


