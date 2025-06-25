use std::{iter::Peekable, str::Chars};

use crate::calc::token::Operator;
use crate::calc::token::Token;

pub struct Tokenizer<'a> {
    expr: Peekable<Chars<'a>>,
    error: Option<String>,
    reached_end: bool,
}

impl<'a> Tokenizer<'a> {
    pub fn from(expr: &'a str) -> Self {
        Tokenizer {
            expr: expr.chars().peekable(),
            error: None,
            reached_end: false,
        }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        // 如果到了文件结束，那么返回
        if self.reached_end {
            return None;
        }

        // 消耗所有的空格
        while let Some(_) = self.expr.next_if(|c| c.is_whitespace()) {}
        // 查看是否有下一个字符，并以此为依据判断应该走到哪一个分支，此时消耗这个字符
        if let Some(char) = self.expr.next() {
            // 查看是否是数字
            if char.is_numeric() {
                let mut num = String::from(char);
                let mut found_point = false;

                // 尽量耗尽这一次的字符
                while let Some(c) = self.expr.next_if(|c| {
                    // 如果是数字字符或者小数点，那么被返回，然后匹配
                    c.is_numeric() || *c == '.'
                }) {
                    // 当是小数点的时候，要检查是否已经有一个小数点，如果有，说明输入错误，将这个字符加入错误中
                    if c == '.' {
                        if found_point {
                            self.error = Some("Multiple decimal point.".to_string());
                            return None;
                        }
                        found_point = true;
                    }
                    num.push(c);
                }

                Some(Token::Num(num.parse().unwrap()))
            } else {
                // 如果是非数字的符号
                match char {
                    '+' => Some(Token::Op(Operator::Add)),
                    '-' => Some(Token::Op(Operator::Sub)),
                    '*' => Some(Token::Op(Operator::Mul)),
                    '/' => Some(Token::Op(Operator::Div)),
                    '^' => Some(Token::Op(Operator::Pow)),

                    '(' => Some(Token::Paren('(')),
                    ')' => Some(Token::Paren(')')),

                    err => {
                        let mut base = "error character: ".to_string();
                        base.push(err);
                        self.error = Some(base);
                        None
                    }
                }
            }
        } else {
            // 没有下一个字符，说明到了末尾，已经没有任意一个字符了
            self.reached_end = true;
            Some(Token::EOF)
        }
    }
}

#[cfg(test)]
mod test {
    use rust_decimal::dec;

    use crate::calc::token::*;
    use crate::calc::tokenizer::*;

    #[test]
    fn number_parsing() {
        let tknz = Tokenizer::from("   +   -   ;   ");
        let res = tknz.collect::<Vec<Token>>();
        assert_eq!(
            res,
            vec![Token::Op(Operator::Add), Token::Op(Operator::Sub)]
        );
    }

    #[test]
    fn whitespace_skipping() {
        let tknz = Tokenizer::from("   12.3  12..3   ");
        let res = tknz.collect::<Vec<Token>>();
        assert_eq!(res, vec![Token::Num(dec!(12.3))]);
    }

    #[test]
    fn total_legal_condition() {
        let tknz = Tokenizer::from(" + 12.000 -");
        let res = tknz.collect::<Vec<Token>>();
        assert_eq!(
            res,
            vec![
                Token::Op(Operator::Add),
                Token::Num(dec!(12.000)),
                Token::Op(Operator::Sub),
                Token::EOF
            ]
        );
    }
}
