mod tokenvals;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    time::Instant,
};

use tokenvals::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TokenType {
    Keyword,
    Identifier,
    Constant(Constant),
    SpecialChar,
    Operator(OperatorType),
    Include,
}

#[derive(Debug)]
pub struct Token {
    token_type: TokenType,
    raw: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Constant {
    Integer,
    Float,
    Octal,
    Hexadecimal,
    Character,
    String,
}

#[derive(Debug)]
pub struct Dekatron {
    pub tokens: Vec<Vec<Token>>,
    pub token_map: HashMap<&'static str, TokenType>,
}

// "." and "->" need to be handled separately
const SPECIALCHARS: [char; 9] = ['(', ')', '{', '}', '[', ']', ',', ';', ':'];

impl Dekatron {
    pub fn tokenize(lines: Vec<String>) -> Self {
        let token_map = fill_td();
        let mut tokens = Vec::new();
        for line in lines {
            let now = Instant::now();
            let line = add_space_around_chars(line, &SPECIALCHARS);
            // Handle #include statements
            if line.starts_with("#include") {
                let il: Vec<&str> = line.split_whitespace().collect();
                if il.len() != 2 {
                    panic!();
                }
                let dep = il[1].replace("\"", "");
                let token = Token {
                    token_type: TokenType::Include,
                    raw: dep,
                };
                tokens.push(vec![token]);
                continue;
            }
            let segments = line.split_whitespace();
            let mut token_line = Vec::new();
            let mut assemble_str = false;
            let mut astr_vec = Vec::new();
            for traw in segments {
                // If the current token is a part of a string which was deconstructed by
                // SplitWhitespace, remake the string
                if traw.starts_with("\"") && !traw.ends_with("\"") && !assemble_str {
                    assemble_str = true;
                }
                if assemble_str {
                    astr_vec.push(traw);
                    if traw.ends_with("\"") {
                        assemble_str = false;
                        let token = Token {
                            token_type: TokenType::Constant(Constant::String),
                            raw: astr_vec.join(" "),
                        };
                        astr_vec.clear();
                        token_line.push(token);
                    }
                    continue;
                }

                // Handle special chars
                if traw.len() == 1 && SPECIALCHARS.contains(&(traw.as_bytes()[0] as char)) {
                    let token = Token {
                        token_type: TokenType::SpecialChar,
                        raw: traw.to_string(),
                    };
                    token_line.push(token);
                    continue;
                }
                // Handle '.' special char if not float
                if traw.contains(".") && !traw.contains("\"") {
                    if traw.parse::<f64>().is_ok() {
                        let token = Token {
                            token_type: TokenType::Constant(Constant::Float),
                            raw: traw.to_string(),
                        };
                        token_line.push(token);
                    } else {
                        let segs: Vec<&str> = traw.split(".").collect();
                        let token = Token {
                            token_type: TokenType::Identifier,
                            raw: segs[0].to_string(),
                        };
                        token_line.push(token);
                        let token = Token {
                            token_type: TokenType::SpecialChar,
                            raw: ".".to_string(),
                        };
                        token_line.push(token);
                        let token = Token {
                            token_type: TokenType::Identifier,
                            raw: segs[1].to_string(),
                        };
                        token_line.push(token);
                    }
                    continue;
                }
                if let Some(tnt) = token_map.get(traw) {
                    let token = Token {
                        token_type: tnt.clone(),
                        raw: traw.to_string(),
                    };
                    token_line.push(token);
                } else if let Some(tnt) = get_constant(traw) {
                    let token = Token {
                        token_type: TokenType::Constant(tnt.clone()),
                        raw: traw.to_string(),
                    };
                    token_line.push(token);
                } else {
                    let token = Token {
                        token_type: TokenType::Identifier,
                        raw: traw.to_string(),
                    };
                    token_line.push(token);
                }
            }
            if assemble_str {
                panic!();
            }
            check_line(&token_line);
            tokens.push(token_line);
            let el = now.elapsed();
            println!("{:?}", el);
        }
        return Self { tokens, token_map };
    }
}

fn check_line(line: &Vec<Token>) {
    // First check the ordering of tokens is syntactically correct

    // Make sure delimiters open and close properly
    let mut index = 0;
    while index < line.len() {
        if is_opening_delimiter(&line[index].raw) {
            index = delimiter_check(&line, index);
        } else if index + 1 == line.len()
            && line.len() != 1
            && is_closing_delimiter(&line[index].raw)
            && line[index].raw != ")"
        {
            panic!("Unmatched closing delimiter!");
        }
        index += 1;
    }
    // If the last token isnt a SpecialChar (Like ";" or "{" for example), panic
    // Also panic if the last token is a closing delimiter
    let last_token = line.last().unwrap();
    if last_token.token_type != TokenType::SpecialChar || is_closing_delimiter(&last_token.raw) {
        //println!("{:?}", line);
        if last_token.raw != ")" && line.len() > 1 {
            panic!()
        }
    };
    // If the first token is a keyword, make sure its not followed by a constant
    // or an operator (with the exception of return constant)
    match line[0].token_type {
        TokenType::Keyword => match line[1].token_type {
            TokenType::Operator(..) => panic!(),
            TokenType::Constant(..) => {
                if line[0].raw != "return" {
                    panic!();
                }
            }
            _ => {}
        },
        TokenType::Constant(..) | TokenType::Operator(..) => panic!(),
        _ => {}
    }
    // Iterate over tokens and make sure ordering of TokenTypes is syntactically correct
    // Exhaustive type checking to be handled by parser
    // This also makes sure identifier names are syntactically correct
    for (i, token) in line.iter().enumerate() {
        match token.token_type {
            TokenType::Keyword => {
                if i != 0 {
                    panic!();
                }
            }
            TokenType::Operator(operator_type) => {
                if i == 0 {
                    panic!()
                }
                let left_val = &line[i - 1];
                let right_val = &line[i + 1];
                match operator_type {
                    OperatorType::Assignment => {
                        if left_val.token_type != TokenType::Identifier {
                            panic!();
                        }
                        match right_val.token_type {
                            TokenType::Identifier | TokenType::Constant(..) => {}
                            TokenType::SpecialChar => {
                                if !is_opening_delimiter(&right_val.raw) {
                                    panic!();
                                }
                            }
                            _ => panic!(),
                        }
                    }
                    OperatorType::Logical
                    | OperatorType::Bitwise
                    | OperatorType::Arithmetic
                    | OperatorType::Relational
                    | OperatorType::Special => {
                        match left_val.token_type {
                            TokenType::Identifier | TokenType::Constant(..) => {}
                            TokenType::SpecialChar => {
                                if !is_closing_delimiter(&left_val.raw) {
                                    panic!();
                                }
                            }
                            _ => panic!(),
                        }
                        match right_val.token_type {
                            TokenType::Identifier | TokenType::Constant(..) => {}
                            TokenType::SpecialChar => {
                                if !is_opening_delimiter(&right_val.raw) {
                                    panic!();
                                }
                            }
                            _ => panic!(),
                        }
                    }
                    OperatorType::IncDec => {
                        if right_val.token_type != TokenType::Identifier {
                            panic!();
                        }
                    }
                }
            }
            TokenType::Constant(..) => {
                if i == 0 {
                    panic!()
                }
            }
            TokenType::Identifier => {
                let tchars: Vec<char> = token.raw.chars().collect();
                if tchars[0].is_numeric() {
                    panic!();
                }
                for c in tchars {
                    if !(c.is_alphanumeric() || c == '_') {
                        panic!();
                    }
                }
            }
            TokenType::SpecialChar => {
                if (i == 0 && token.raw != "{" && token.raw != "}")
                    || (i != 0
                        && i == line.len() - 1
                        && token.raw != ";".to_string()
                        && token.raw != ")".to_string()
                        && line[i - 1].token_type == TokenType::SpecialChar)
                    || (i != line.len() - 1 && token.raw == ";")
                {
                    panic!();
                }
            }
            _ => {}
        }
    }
}

fn is_closing_delimiter(token: &String) -> bool {
    return ["}".to_string(), "]".to_string(), ")".to_string()].contains(token);
}

fn is_opening_delimiter(token: &String) -> bool {
    return ["{".to_string(), "[".to_string(), "(".to_string()].contains(token);
}

fn delimiter_check(line: &Vec<Token>, index: usize) -> usize {
    let raw = &line[index].raw;
    let n = line.len();
    // Could potentially make a static variable to avoid unnecessary allocations
    let ocd = [
        [&"(".to_string(), &")".to_string()],
        [&"{".to_string(), &"}".to_string()],
        [&"[".to_string(), &"]".to_string()],
    ];
    let mut i = index + 1;
    while i < n {
        if line[i].token_type == TokenType::SpecialChar {
            //println!("{:?}", &[raw, &line[i].raw]);
            if ocd.contains(&[raw, &line[i].raw]) {
                //println!("CLosinh");
                return i;
            } else {
                if is_closing_delimiter(&line[i].raw) {
                    panic!("Unmatched opening delimiter!");
                } else if is_opening_delimiter(&line[i].raw) && i != n - 1 {
                    i = delimiter_check(line, i);
                }
                if i == n - 1 && index != n - 1 {
                    panic!("Unmatched opening delimiter!");
                }
            }
        }
        i += 1;
    }
    return n;
}

fn add_space_around_chars(input: String, chars: &[char]) -> String {
    let mut result = String::new();
    let mut prev_char = ' ';

    for current_char in input.chars() {
        // Check if the current character is one of the specified characters
        if chars.contains(&current_char) && prev_char != ' ' {
            result.push(' '); // Add a space before the character if not already there
        }
        result.push(current_char);
        if chars.contains(&current_char) {
            result.push(' '); // Add a space after the character
        }
        prev_char = current_char; // Update previous character
    }

    result
}

fn get_constant(token: &str) -> Option<Constant> {
    if token.starts_with("\"") {
        return Some(Constant::String);
    }
    if token.starts_with("'") {
        return Some(Constant::Character);
    }
    if token.parse::<isize>().is_ok() {
        return Some(Constant::Integer);
    }
    if isize::from_str_radix(token, 8).is_ok() && !token.contains("x") {
        return Some(Constant::Octal);
    }
    if isize::from_str_radix(token, 16).is_ok() && token.starts_with("0x") {
        return Some(Constant::Hexadecimal);
    }
    None
}

pub fn read_file(path: &str) -> Vec<String> {
    if !path.ends_with(".c") {
//        panic!("Not a c file!");
    }
    let mut file = File::open(path).expect("Could not open file!!!");
    let buf = BufReader::new(&mut file);
    let mut lines = Vec::new();
    for l in buf.lines() {
        let line = l.unwrap();
        if !line.is_empty() {
            lines.push(line);
        }
    }
    return lines;
}





