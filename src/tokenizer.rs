use types::ParseError;
use types::Symbol;
use types::{Element, Number, String, Boolean, List, Vec};
use types::nil;

mod types;

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
            if l.len() > 0 {
                List(l.map(|x| tokenize_infer_types(x.clone())))
            } else {
                List(l)
            }
        },
        Symbol(s) => {
            if s == ~"true" || s == ~"false" {
                Boolean(s == ~"true")
            } else {
                match from_str::<i64>(s) {
                    Some(i) => Number(i),
                    None => Symbol(s)
                }
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
pub fn tokenize(s: &str) -> Element
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


