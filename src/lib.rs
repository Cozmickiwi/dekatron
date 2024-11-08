mod tokenvals;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    mem::discriminant,
    time::Instant,
};

use tokenvals::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TokenType {
    Keyword,
    Identifier,
    Constant(Constant),
    SpecialChar(Option<usize>),
    Operator(OperatorType),
    Include,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub raw: String,
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
        let mut line_number = 0;
        for line in lines {
            line_number += 1;
            if line.is_empty() {
                continue;
            }
            println!("{line}");
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
                        token_type: TokenType::SpecialChar(None),
                        raw: traw.to_string(),
                    };
                    token_line.push(token);
                    continue;
                }
                // Handle '.' special char if not float
                if traw.contains(".") && !traw.contains("\"") && traw.parse::<f64>().is_err() {
                    let segs: Vec<&str> = traw.split(".").collect();
                    let token = Token {
                        token_type: TokenType::Identifier,
                        raw: segs[0].to_string(),
                    };
                    token_line.push(token);
                    let token = Token {
                        token_type: TokenType::SpecialChar(None),
                        raw: ".".to_string(),
                    };
                    token_line.push(token);
                    let token = Token {
                        token_type: TokenType::Identifier,
                        raw: segs[1].to_string(),
                    };
                    token_line.push(token);
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
                println!("Line {line_number}: Invalid string");
                panic!();
            }
            //       check_line(&token_line);
            // Split token_line into seperate line if needed
            let mut lstart = 0;
            //println!("{:?}", token_line);
            if token_line
                .iter()
                .find(|&x| {
                    discriminant(&x.token_type) == discriminant(&TokenType::SpecialChar(None))
                        && ["{", "}", ";", ":"].contains(&x.raw.as_str())
                })
                .is_none()
            {
                tokens.push(token_line);
            } else {
                for (i, token) in token_line.iter().enumerate() {
                    if let TokenType::SpecialChar(..) = token.token_type {
                        match &token.raw[0..1] {
                            // Braces should go on their own line to easily define blocks of code
                            "{" | "}" => {
                                if i != lstart {
                                    tokens.push(token_line[lstart..i].to_vec());
                                }
                                tokens.push(vec![token.clone()]);
                                lstart = i + 1;
                            }
                            ";" | ":" => {
                                tokens.push(token_line[lstart..i + 1].to_vec());
                                lstart = i + 1;
                            }
                            _ => {}
                        }
                    } else if i == token_line.len() - 1 {
                        tokens.push(token_line[lstart..i + 1].to_vec());
                    }
                }
            }
            //tokens.push(token_line);
            let el = now.elapsed();
            println!("{:?}", el);
        }
        let now = Instant::now();
        check_blocks(&mut tokens);
        // Merge severed lines, for example:
        // int a = (1
        // + 2);
        // Becomes:
        // int a = (1 + 2);
        merge_lines(&mut tokens);
        //        check_blocks(&mut tokens);
        let el = now.elapsed();
        println!("Check and merge: {:?}", el);
        return Self { tokens, token_map };
    }
}

fn merge_lines(tlines: &mut Vec<Vec<Token>>) {
    let mut merge = false;
    let mut merge_index = 0;
    for i in 0..tlines.len() {
        let line = &tlines[i];
        if line.len() == 0 {
            continue;
        }
        let comp_line = line.last().unwrap().raw == ";".to_string()
            || line.last().unwrap().raw == ":".to_string()
            || (["{", "}"].contains(&line[0].raw.as_str()) && line.len() == 1)
            || line.last().unwrap().token_type == TokenType::Include;
        if merge {
            let line = tlines[i].clone();
            if line.len() == 1 && (line[0].raw == "}".to_string() || line[0].raw == "{".to_string())
            {
                merge = false;
                continue;
            } else {
                // assumes lines with inline special characters (e.g. a = 1; b = 2;)
                // have been handled
                tlines[merge_index].extend(line);
                tlines[i] = Vec::new();
                if comp_line {
                    merge = false;
                }
            }
        }
        if !comp_line && !merge {
            merge = true;
            merge_index = i;
        }
    }
    let mut rem_lines = Vec::new();
    for i in 0..tlines.len() {
        if tlines[i].is_empty() {
            rem_lines.push(i);
        }
    }
    for (i, r) in rem_lines.iter().enumerate() {
        tlines.remove(r - i);
    }
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Constant {
    Integer(SuffixType),
    Float(SuffixType),
    Octal(SuffixType),
    Hexadecimal(SuffixType),
    Character,
    String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SuffixType {
    LongInt,
    LongLongInt,
    UInt,
    ULongInt,
    ULongLongInt,
    Float,
    LongDouble,
    Default,
}

fn get_constant(token: &str) -> Option<Constant> {
    if token.starts_with("\"") {
        return Some(Constant::String);
    }
    if token.starts_with("'") {
        return Some(Constant::Character);
    }
    // If token is a number, determine if it contains a prefix
    let token_chars: Vec<char> = token.chars().collect();
    if !token_chars[0].is_numeric() {
        return None;
    }
    let mut suffix = SuffixType::Default;
    let mut suffix_chars: Vec<char> = Vec::new();
    let tn = token_chars.len();
    for c in token_chars.into_iter().rev() {
        if c.is_alphabetic() {
            suffix_chars.push(c);
        } else {
            break;
        }
    }
    let sn = suffix_chars.len();
    if sn > 0 {
        suffix_chars.sort();
        let suffix_str: String = suffix_chars.iter().collect();
        match &(*suffix_str.to_lowercase()) {
            "u" => suffix = SuffixType::UInt,
            "l" => suffix = SuffixType::LongInt,
            "f" => suffix = SuffixType::Float,
            "lu" => suffix = SuffixType::ULongInt,
            "ll" => suffix = SuffixType::LongLongInt,
            "llu" => suffix = SuffixType::ULongLongInt,
            _ => panic!(),
        }
    }
    let trimmed_token = &token[..tn - sn];
    if trimmed_token.parse::<isize>().is_ok() {
        return Some(Constant::Integer(suffix));
    }
    if isize::from_str_radix(trimmed_token, 8).is_ok() && !token.contains("x") {
        return Some(Constant::Octal(suffix));
    }
    if isize::from_str_radix(trimmed_token, 16).is_ok() && token.starts_with("0x") {
        return Some(Constant::Hexadecimal(suffix));
    }
    if trimmed_token.parse::<f32>().is_ok() {
        if suffix == SuffixType::LongInt {
            suffix = SuffixType::LongDouble;
        }
        return Some(Constant::Float(suffix));
    }
    None
}

fn check_blocks(lines: &mut Vec<Vec<Token>>) {
    // Go through each line until an open brace with no id is found
    // Then its matching closing brace is found assign an id to it
    // Do the same for parens and brackets
    let mut id: [usize; 3] = [0; 3];
    let mut inner_id: [usize; 3] = [0; 3];
    let mut block_nest_count: [usize; 3] = [0; 3];
    let openi = ["{", "(", "["];
    let closei = ["}", ")", "]"];
    for l in lines {
        for token in l {
            if token.token_type == TokenType::SpecialChar(None) {
                let symbol = &token.raw[0..1];
                if let Some(i) = openi.iter().position(|&x| x == symbol) {
                    inner_id[i] += 1;
                    if symbol == "{" || symbol == "}" {
                        token.token_type = TokenType::SpecialChar(Some(id[i] + inner_id[i]));
                    }
                    block_nest_count[i] += 1;
                } else if let Some(i) = closei.iter().position(|&x| x == symbol) {
                    if inner_id[i] == 0 {
                        panic!("Unexpected closing delimiter!");
                    }
                    if symbol == "{" || symbol == "}" {
                        token.token_type = TokenType::SpecialChar(Some(id[i] + inner_id[i]));
                    }
                    inner_id[i] -= 1;
                    if inner_id[i] == 0 {
                        id[i] += block_nest_count[i];
                        block_nest_count[i] = 0;
                    }
                }
            }
        }
    }
    if inner_id.iter().sum::<usize>() > 0 {
        panic!("Unclosed delimiter!");
    }
}

pub fn read_file(path: &str) -> Vec<String> {
    if !path.ends_with(".c") {
        panic!("Not a c file!");
    }
    let mut file = File::open(path).expect("Could not open file!!!");
    let buf = BufReader::new(&mut file);
    let mut lines = Vec::new();
    for l in buf.lines() {
        let line = l.unwrap();
        //if !line.is_empty() {
            lines.push(line);
        //}
    }
    return lines;
}
