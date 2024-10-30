mod tokenvals;
use std::collections::HashMap;

use tokenvals::*;

#[derive(Clone, Copy, Debug)]
pub enum TokenType {
    Keyword,
    Identifier,
    Constant(Constant),
    SpecialChar,
    Operator(OperatorType),
}

#[derive(Debug)]
pub struct Token {
    token_type: TokenType,
    raw: String,
}

#[derive(Clone, Copy, Debug)]
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
    pub fn tokenize(lines: Vec<&str>) -> Self {
        let token_map = fill_td();
        let mut tokens = Vec::new();
        for line in lines {
            let line = add_space_around_chars(line, &SPECIALCHARS);
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
            tokens.push(token_line);
        }
        return Self { tokens, token_map };
    }
}

fn add_space_around_chars(input: &str, chars: &[char]) -> String {
    let mut result = String::new();
    let mut prev_char = ' ';

    for current_char in input.chars() {
        // Check if the current character is one of the specified characters
        if chars.contains(&current_char) && prev_char != ' ' {
            result.push(' '); // Add a space before the character if not already there
        }
        result.push(current_char);
        if chars.contains(&current_char) {
            result.push(' '); // Add a space before the character if not already there
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
        return Some(Constant::String);
    }
    if token.parse::<isize>().is_ok() {
        return Some(Constant::Integer);
    }
    if isize::from_str_radix(token, 8).is_ok() && !token.contains("x") {
        return Some(Constant::Octal);
    }
    if isize::from_str_radix(token, 16).is_ok() && token.contains("x") {
        return Some(Constant::Hexadecimal);
    }
    None
}
