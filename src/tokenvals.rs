use std::{collections::HashMap, sync::LazyLock};

use crate::TokenType;

#[derive(Clone, Copy, Debug)]
pub enum OperatorType {
    Arithmetic,
    Relational,
    Logical,
    IncDec,
    Bitwise,
    Assignment,
    Special,
}

const KEYWORDS: [&str; 32] = [
    "auto", "break", "case", "char", "const", "continue", "default", "do", "double", "else",
    "enum", "extern", "float", "for", "goto", "if", "int", "long", "register", "return", "short",
    "signed", "sizeof", "static", "struct", "switch", "typedef", "union", "unsigned", "void",
    "volatile", "while",
];

const ARITHMETICOPS: [&str; 5] = ["+", "-", "*", "/", "%"];
const RELATIONALOPS: [&str; 6] = ["==", ">", "<", "!=", ">=", "<="];
const LOGICALOPS: [&str; 3] = ["&&", "||", "!"];
const INCDECOPS: [&str; 2] = ["++", "--"];
const BITWISEOPS: [&str; 6] = ["&", "|", "^", "~", ">>", "<<"];
const ASSIGNMENTOPS: [&str; 6] = ["=", "+=", "-=", "*=", "/=", "%="];

pub fn fill_td() -> HashMap<&'static str, TokenType> {
    let mut map = HashMap::new();
    // Add keywords
    for i in KEYWORDS {
        map.insert(i, TokenType::Keyword);
    }
    // Add operators
    for i in ARITHMETICOPS {
        map.insert(i, TokenType::Operator(OperatorType::Arithmetic));
    }
    for i in RELATIONALOPS {
        map.insert(i, TokenType::Operator(OperatorType::Relational));
    }
    for i in LOGICALOPS {
        map.insert(i, TokenType::Operator(OperatorType::Logical));
    }
    for i in INCDECOPS {
        map.insert(i, TokenType::Operator(OperatorType::IncDec));
    }
    for i in BITWISEOPS {
        map.insert(i, TokenType::Operator(OperatorType::Bitwise));
    }
    for i in ASSIGNMENTOPS {
        map.insert(i, TokenType::Operator(OperatorType::Assignment));
    }
    return map;
}
