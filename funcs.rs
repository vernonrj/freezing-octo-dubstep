/**
 * @file funcs.rs
 * @brief primitive function definitions
 *
 * This module contains functions that must use
 * functions compiled-in, instead of bound functions.
 */
use std::vec;

use types::Element;
use types::EvalError;
use types::{Number, String, List, Vec, Character};

use eval::eval;

mod types;

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

#[allow(dead_code)]
pub fn add(list: &[Element]) -> Element
{
    let vals: Option<~[i64]> = unwrap_to_nums(list);
    match vals {
        Some(is) => {
            let sum: i64 = is.iter().fold(0, |a, &b| {
                a + b
            });
            Number(sum)
        },
        None => EvalError(~"+: invalid value")
    }
}

#[allow(dead_code)]
pub fn sub(list: &[Element]) -> Element
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
        None => EvalError(~"-: invalid value")
    }
}

#[allow(dead_code)]
pub fn mul(list: &[Element]) -> Element
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
        None => EvalError(~"*: invalid value")
    }
}

#[allow(dead_code)]
pub fn div(list: &[Element]) -> Element
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
        None => EvalError(~"/: invalid value")
    }
}

#[allow(dead_code)]
pub fn modfn(list: &[Element]) -> Element
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
        None => EvalError(~"%: invalid value")
    }
}

#[allow(dead_code)]
pub fn concat(more: &[Element]) -> Element
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

