#[warn(unused_imports)]
use std::str::Chars;
use std::collections::HashMap;
use std::any::Any;

trait JValue {
    fn as_any(&self) -> &Any;
}

struct JBoolean(bool);

impl JValue for JBoolean {
    fn as_any(&self) -> &Any { self }
}

#[derive(PartialEq, Eq, Hash)]
struct JString(String);

impl JValue for JString {
    fn as_any(&self) -> &Any { self }
}

struct JNumber(f64);

impl JValue for JNumber {
    fn as_any(&self) -> &Any { self }
}

struct JArray(Vec<Box<JValue>>);

impl JValue for JArray {
    fn as_any(&self) -> &Any { self }
}

struct JObject(HashMap<JString, Box<JValue>>);

impl JValue for JObject {
    fn as_any(&self) -> &Any { self }
}

struct JNull;

impl JValue for JNull {
    fn as_any(&self) -> &Any { self }
}

enum Token {
    ArrayStart,
    ObjectStart,
    Elem(Box<JValue>),
}

pub fn dumps(obj: &JValue) -> String { "".to_owned() }

pub fn loads(s: &String) -> Box<JValue> {
    let mut uchars = s.chars();

    let mut stack: Vec<Token> = Vec::new();

    loop {
        //skip whitespace and punctuation

        let next = uchars.by_ref().next();

        match next {
            Some(a_char) if a_char.is_ascii_punctuation() || a_char.is_ascii_whitespace() => continue,
            Some(a_char) if a_char == '[' => stack.push(Token::ArrayStart),
            Some(a_char) if a_char == ']' => {
                let mut v: Vec<Box<JValue>> = Vec::new();
                loop {
                    let elem = stack.pop();
                    match elem {
                        Some(Token::ArrayStart) => {
                            stack.push(Token::Elem(Box::new(JArray(v))));
                            break;
                        }
                        Some(Token::Elem(box_jval)) => v.push(box_jval),
                        _ => panic!("fail to parse string"),
                    }
                }
            }
            Some(a_char) if a_char == '{' => stack.push(Token::ObjectStart),
//                Some(a_char) if a_char == '}' => {
//                    let mut map: HashMap<JString, Box<JValue>> = HashMap::new();
//                    loop {
//                        let elem = stack.pop();
//                        match elem {
//                            Some(Token::ObjectStart) => {
//                                stack.push(Token::Elem(Box::new(JObject(map))));
//                                break
//                            },
//                            Some(Token::Elem(box_jval)) => {
//                                let box_jkey = match stack.pop().unwrap() {
//                                    Token::Elem(box_jkey) => box_jkey,
//                                    _ => panic!("fail to find key for object")
//                                };
////                                let Token::Elem(box_jkey) = stack.pop().unwrap();
////                                let jkey = box_jkey as Box<JString>;
//                                let key = (*box_jkey).as_any();
////                                let key = *jkey;
//                                let x = *key.downcast_ref::<JString>().unwrap();
//                                map.insert(x, box_jval);
//                            },
//                            _ => panic!("fail to parse string"),
//                        }
//                    }
//                },
            Some(a_char) if a_char == '"' => {
                let s = uchars
                    .by_ref()
                    .take_while(|x| x != &'"')
                    .fold(String::new(), |mut acc, c| {
                        acc.push(c);
                        acc
                    });

                //consume another `"`
                assert_eq!(uchars.next(), Some('"'));

                let js = JString(s);
                stack.push(Token::Elem(Box::new(js)));
            }
            Some(a_char) if a_char.is_ascii_digit() || a_char == '-' => {
                let init = if a_char.is_ascii_digit() { a_char.to_string() } else { String::new() };
                let negative = if a_char == '-' { -1.0 } else { 1.0 };

                let mut s = uchars
                    .by_ref()
                    .take_while(|x| !x.is_ascii_whitespace() || !x.is_ascii_punctuation())
                    .fold(init, |mut acc, c| {
                        acc.push(c);
                        acc
                    });

                let jnum = JNumber(negative * s.parse::<f64>().unwrap());
                stack.push(Token::Elem(Box::new(jnum)));
            }
            Some(a_char) if a_char == 't' => {
                let take = uchars.by_ref().take(3).collect::<String>();
                let mut s = 't'.to_string();
                s.push_str(&take);
                assert_eq!(s, "true");

                stack.push(Token::Elem(Box::new(JBoolean(true))));
            }
            Some(a_char) if a_char == 'f' => {
                let take = uchars.by_ref().take(4).collect::<String>();
                let mut s = 'f'.to_string();
                s.push_str(&take);
                assert_eq!(s, "false");

                stack.push(Token::Elem(Box::new(JBoolean(false))));
            }
            Some(a_char) if a_char == 'n' => {
                let take = uchars.by_ref().take(3).collect::<String>();
                let mut s = 'n'.to_string();
                s.push_str(&take);
                assert_eq!(s, "null");

                stack.push(Token::Elem(Box::new(JNull)));
            }
            Some(_) => panic!("what shouldn't happen happened"),
            None => break,
        }
    }

    assert_eq!(stack.len(), 1);
    let result = match stack.pop().unwrap() {
        Token::Elem(res) => res,
        _ => panic!("fail to parse the input")
    };
    result
}
