use rust_decimal::Decimal;

use crate::calc::parser::Parser;

mod token;
mod tokenizer;
mod ast;
mod parser;

pub fn expr(expr: &str) -> Result<Decimal, String> {
    let parse_res = Parser::parse(expr)?;
    Ok(parse_res.eval())
}