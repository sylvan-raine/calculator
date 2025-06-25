use std::fmt::Display;

use rust_decimal::Decimal;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

impl Operator {
    pub fn priority(&self) -> u8 {
        use Operator::*;
        match self {
            Add | Sub => 1,
            Mul | Div => 2,
            Pow => 3,
        }
    }

    pub fn is_left_associative(self) -> bool {
        if self == Operator::Pow { false } else { true }
    }
}

impl PartialOrd for Operator {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.priority().cmp(&other.priority()))
    }
}

impl Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Operator::*;
        match self {
            Add => write!(f, "+"),
            Sub => write!(f, "-"),
            Mul => write!(f, "*"),
            Div => write!(f, "/"),
            Pow => write!(f, "^"),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Token {
    Op(Operator),
    Paren(char),
    Num(Decimal),
    EOF,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Token::*;
        match self {
            Op(op) => op.fmt(f),
            Paren(lr) => write!(f, "{lr}"),
            Num(num) => write!(f, "{num}"),
            EOF => write!(f, "EOF"),
        }
    }
}
