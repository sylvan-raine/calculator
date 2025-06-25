use crate::calc::{
    ast::{Ast, Node},
    token::{Operator, Token},
    tokenizer::Tokenizer,
};

pub struct Parser;

fn basic_check(tokens: Vec<Token>) -> Result<Vec<Token>, String> {
    let mut res = Vec::new();
    if !tokens.ends_with(&[Token::EOF]) {
        Err("Errors occurs before translating".to_string())
    } else {
        let mut last_token = Token::EOF;
        let mut lparen = Vec::<bool>::new();
        for token in tokens {
            match token {
                Token::Num(n) => {
                    match last_token {
                        Token::Num(last_num) => {
                            // 上一个 token 是 num，那么这一个符号应该是运算符或者结束，但是这一个却是数字，非法
                            return Err(format!("Expected operator or eof behind num: '{last_num}', found num: '{n}'"))
                        }
                        Token::Paren(lr) => {
                            // 如果上一个括号类token是括号
                            if lr == ')' {
                                // 如果上一个token是右括号，这并不合法
                                return Err(format!("Expected operator or eof behind ')', found num: '{n}'"));
                            } else {
                                res.push(Token::Num(n));
                                last_token = Token::Num(n);
                            }
                        }
                        Token::Op(op) => {
                            // 如果上一个token类型是运算符，那么应该将运算符的正负合到数字里面（仅正负能够整合到数字里）
                            match op {
                                Operator::Sub => {
                                    res.pop(); // 这里将负号弹出来了
                                    match res.last() {
                                        Some(Token::Num(_)) => {
                                            // 如果是 数 负号 数 的序列，将这个负号解释为减号，把这个减号重新加入结果的token序列
                                            res.push(Token::Op(Operator::Sub));
                                            res.push(Token::Num(n));
                                            last_token = Token::Num(n);
                                        }
                                        Some(Token::Op(_)) => {
                                            // 如果是 运算符 负号 数 的序列，将这个负号解释为负号，将这个数字取反加入token序列
                                            res.push(Token::Num(-n));
                                            last_token = Token::Num(-n);
                                        }
                                        Some(Token::Paren(lr)) => {
                                            if *lr == '(' {
                                                // 如果是左括号 减号 数字的序列，那么这个数字是表达式开始，解释为负号，将数字取反加入token序列
                                                res.push(Token::Num(-n));
                                                last_token = Token::Num(-n);
                                            } else {
                                                // 如果是右括号 减号 数字的序列，那么这个数字是减号的第二个操作数，解释为减号
                                                res.push(Token::Op(Operator::Sub));
                                                res.push(Token::Num(n));
                                                last_token = Token::Num(n);
                                            }
                                        }
                                        Some(Token::EOF) => {
                                            // EOF 不会加入序列，不可能有
                                            unreachable!()
                                        }
                                        None => {
                                            // 负号前面没有token，那么这个数字是算式开始，将这个负号解释为负号，将数字取反加入token序列
                                            res.push(Token::Num(-n));
                                            last_token = Token::Num(-n);
                                        }
                                    }
                                }
                                Operator::Add => {
                                    res.pop(); // 这里将正号弹出来了
                                    match res.last() {
                                        Some(Token::Num(_)) => {
                                            // 如果是 数 正号 数 的序列，将这个负号解释为加号，把这个加号重新加入结果的token序列
                                            res.push(Token::Op(Operator::Add));
                                            res.push(Token::Num(n));
                                            last_token = Token::Num(n);
                                        }
                                        Some(Token::Op(_)) => {
                                            // 如果是 运算符 正号 数 的序列，将这个负号解释为正号，将这个数字加入token序列
                                            res.push(Token::Num(n));
                                            last_token = Token::Num(n);
                                        }
                                        Some(Token::Paren(lr)) => {
                                            if *lr == '(' {
                                                // 如果是左括号 加号号 数字的序列，那么这个数字是表达式开始，解释为正号，将数字加入token序列
                                                res.push(Token::Num(n));
                                                last_token = Token::Num(n);
                                            } else {
                                                // 如果是右括号 加号 数字的序列，那么这个数字是加号的第二个操作数，解释为加号
                                                res.push(Token::Op(Operator::Add));
                                                res.push(Token::Num(n));
                                                last_token = Token::Num(n);
                                            }
                                        }
                                        Some(Token::EOF) => {
                                            // EOF 不会加入序列，不可能有
                                            unreachable!()
                                        }
                                        None => {
                                            // 正号前面没有token，那么这个数字是算式开始，将这个正号解释为正号，将数字加入token序列
                                            res.push(Token::Num(n));
                                            last_token = Token::Num(n);
                                        }
                                    }
                                }
                                _ => {
                                    // 如果是其他运算符，直接加入序列
                                    res.push(Token::Num(n));
                                    last_token = Token::Num(n);
                                }
                            }
                        }
                        Token::EOF => {
                            // 这说明这个数字是算式开始，直接加入
                            res.push(Token::Num(n));
                            last_token = Token::Num(n);
                        }
                    }
                }
                Token::Op(this_op) => {
                    if let Token::Op(last_op) = last_token {
                        // 如果上一个 token 是运算符，看一下能否进行运算符的合并，同样只有正负号能够合并
                        use Operator::*;
                        if last_op.priority() == 1 && this_op.priority() == 1 {
                            // 合并正负运算符
                            if last_op == this_op {
                                // 如果同号，那么把上一个token拿出来（有可能是负负得正），再加入一个正号token
                                res.pop();
                                res.push(Token::Op(Add));
                                last_token = Token::Op(Add);
                            } else {
                                // 如果异号，那么把上一个token拿出来（有可能是正负得负），再加入一个负号token
                                res.pop();
                                res.push(Token::Op(Sub));
                                last_token = Token::Op(Sub);
                            }
                        } else if last_op.priority() > 1 && this_op.priority() == 1 {
                            // 允许乘除一个负数或者负的表达式，也允许乘幂后面跟一个正负号
                            res.push(Token::Op(this_op));
                            last_token = Token::Op(this_op);
                        } else {
                            // 这种情况就是剩下的this_op的优先级大于1的情况，比如^*, -/, */，^/ 都是不合法的
                            return Err(format!(
                                "Expected num or parenthesis behind '{last_op}', found operator: '{this_op}'"
                            ));
                        }
                    } else {
                        // 上一个token不是运算符
                        match last_token {
                            Token::Paren(last_token) => {
                                if last_token == '(' && this_op.priority() >= 1 {
                                    // 左括号后面跟着非正负的运算符，不合法，提前返回
                                    return Err(format!(
                                        "Expected num or expr behind '(', found operator: '{this_op}'"
                                    ));
                                }
                            }
                            Token::EOF => {
                                // 如果上一个 token 是一开始的 EOF，如果不是正负号，非法
                                if this_op.priority() > 1 {
                                    return Err(format!("Expected a num or '(' to start an expr, found '{this_op}'"))
                                }
                            }
                            // 如果上一个 token 是数字、右括号，合法离开这个分支继续执行
                            _ => {}
                        }
                        res.push(Token::Op(this_op));
                        last_token = Token::Op(this_op)
                    }
                }
                Token::Paren(lr) => {
                    match lr {
                        '(' => lparen.push(true),
                        ')' => match lparen.pop() {
                            Some(_) => {}
                            None => return Err("Unmatched brackets".to_string()),
                        },
                        _ => unreachable!(), // 分词器会剔除其他符号
                    }
                    res.push(Token::Paren(lr));
                    last_token = Token::Paren(lr)
                }
                Token::EOF => break,
            }
        }
        if !lparen.is_empty() {
            Err("Unmatched brackets".to_string())
        } else {
            Ok(res)
        }
    }
}

fn get_rpn(tokens: Vec<Token>) -> Result<Vec<Token>, String> {
    use Token::*;

    let mut rpn = Vec::new(); // 输出队列
    let mut stack = Vec::new(); // 运算符栈

    for token in tokens {
        match token {
            Num(_) => rpn.push(token.clone()),       // 数字直接输出
            Paren('(') => stack.push(token.clone()), // 左括号入栈
            Paren(')') => {
                // 弹出所有运算符直到左括号
                while let Some(top) = stack.pop() {
                    match top {
                        Paren('(') => break, // 找到左括号，停止
                        _ => rpn.push(top),  // 其他运算符输出
                    }
                }
            }
            Paren(_) => unreachable!(),

            Op(op) => {
                let current_op = op;
                // 处理栈顶优先级更高的运算符
                while let Some(top) = stack.last() {
                    match top {
                        Op(top_op) => {
                            let top_pri = top_op.priority();
                            let cur_pri = current_op.priority();

                            // 优先级比较：考虑结合性
                            if (top_op.is_left_associative() && top_pri >= cur_pri)
                                || (!top_op.is_left_associative() && top_pri > cur_pri)
                            {
                                rpn.push(stack.pop().unwrap());
                            } else {
                                break;
                            }
                        }
                        Paren('(') => break, // 左括号保留在栈中
                        _ => break,
                    }
                }

                // 当前运算符入栈
                stack.push(token.clone());
            }

            EOF => break, // 结束标志
        }
    }

    // 弹出栈中所有剩余运算符
    while let Some(op) = stack.pop() {
        if let Paren('(') = op {
            return Err("Unmatched parentheses".to_string(),);
        }
        rpn.push(op);
    }

    Ok(rpn)
}

fn parse(tokens: Vec<Token>) -> Result<Ast, String> {
    use Token::*;

    let mut stack: Vec<Node> = Vec::new();

    for token in tokens {
        match token {
            Num(n) => stack.push(Node::Num(n)),

            Op(op) => {
                if stack.len() < 2 {
                    return Err(
                        "Insufficient operands for operator".to_string(),
                    );
                }

                let right = Box::new(stack.pop().unwrap());
                let left = Box::new(stack.pop().unwrap());

                let node = match op {
                    Operator::Add => Node::Add(left, right),
                    Operator::Sub => Node::Sub(left, right),
                    Operator::Mul => Node::Mul(left, right),
                    Operator::Div => Node::Div(left, right),
                    Operator::Pow => Node::Pow(left, right),
                };

                stack.push(node);
            }

            _ => {} // 忽略其他token
        }
    }

    if stack.len() != 1 {
        Err(
            "Malformed expression".to_string(),
        )
    } else {
        Ok(Ast(stack.pop().unwrap()))
    }
}

impl Parser {
    pub fn parse(expr: &str) -> Result<Ast, String> {
        let tokens: Vec<Token> = Tokenizer::from(expr).collect();
        parse(get_rpn(basic_check(tokens)?)?)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rust_decimal::dec;

    #[test]
    fn basic_check_sign_merge() {
        // 测试两个正号合并
        let tokens = Tokenizer::from("13.0 + +1 ").collect();
        assert_eq!(
            basic_check(tokens),
            Ok(vec![
                Token::Num(dec!(13.0)),
                Token::Op(Operator::Add),
                Token::Num(dec!(1))
            ])
        );

        // 测试正负号合并
        let tokens = Tokenizer::from("13.0 +- 1 ").collect();
        assert_eq!(
            basic_check(tokens),
            Ok(vec![
                Token::Num(dec!(13.0)),
                Token::Op(Operator::Sub),
                Token::Num(dec!(1))
            ])
        );

        // 测试负正号合并
        let tokens = Tokenizer::from("13.0 -+ 1 ").collect();
        assert_eq!(
            basic_check(tokens),
            Ok(vec![
                Token::Num(dec!(13.0)),
                Token::Op(Operator::Sub),
                Token::Num(dec!(1))
            ])
        );

        // 测试两个负号合并
        let tokens = Tokenizer::from("13.0 -- 1 ").collect();
        assert_eq!(
            basic_check(tokens),
            Ok(vec![
                Token::Num(dec!(13.0)),
                Token::Op(Operator::Add),
                Token::Num(dec!(1))
            ])
        );

        // 测试三个负号合并
        let tokens = Tokenizer::from("13.0 --- 1 ").collect();
        assert_eq!(
            basic_check(tokens),
            Ok(vec![
                Token::Num(dec!(13.0)),
                Token::Op(Operator::Sub),
                Token::Num(dec!(1))
            ])
        );

        // 测试三个正号合并
        let tokens = Tokenizer::from("13.0 +++ 1 ").collect();
        assert_eq!(
            basic_check(tokens),
            Ok(vec![
                Token::Num(dec!(13.0)),
                Token::Op(Operator::Add),
                Token::Num(dec!(1))
            ])
        );
    }

    #[test]
    fn basic_check_error_check() {
        // 检测括号匹配
        let tokens = Tokenizer::from("(13.0 + 1 (").collect();
        assert_eq!(basic_check(tokens), Err("Unmatched brackets".to_string()));

        // 测试运算符之后跟运算符的错误
        let tokens = Tokenizer::from("13.0 +* 1 ").collect();
        assert_eq!(
            basic_check(tokens),
            Err("Expected num or parenthesis behind '+', found operator: '*'".to_string())
        );

        // 测试运算符之后跟运算符的错误
        let tokens = Tokenizer::from("13.0 *+ 1 ").collect();
        assert_eq!(
            basic_check(tokens),
            Ok(vec![
                Token::Num(dec!(13.0)),
                Token::Op(Operator::Mul),
                Token::Num(dec!(1))
            ])
        );

        // 测试运算符之后跟运算符的错误
        let tokens = Tokenizer::from("13.0 *- 1 ").collect();
        assert_eq!(
            basic_check(tokens),
            Ok(vec![
                Token::Num(dec!(13.0)),
                Token::Op(Operator::Mul),
                Token::Num(dec!(-1))
            ])
        );

        // 测试运算符之后跟运算符的错误
        let tokens = Tokenizer::from("13.0 *---- 1 ").collect();
        assert_eq!(
            basic_check(tokens),
            Ok(vec![
                Token::Num(dec!(13.0)),
                Token::Op(Operator::Mul),
                Token::Num(dec!(1))
            ])
        );

        // 测试运算符之后跟运算符的错误
        let tokens = Tokenizer::from("13.0 *-+-- 1 ").collect();
        assert_eq!(
            basic_check(tokens),
            Ok(vec![
                Token::Num(dec!(13.0)),
                Token::Op(Operator::Mul),
                Token::Num(dec!(-1))
            ])
        );

        // 测试运算符之后跟运算符的错误
        let tokens = Tokenizer::from("13.0 *--/ 1 ").collect();
        assert_eq!(
            basic_check(tokens),
            Err("Expected num or parenthesis behind '+', found operator: '/'".to_string())
        );

        // 测试左括号后跟运算符的错误
        let tokens = Tokenizer::from("13.0 * 1 (--/ ").collect();
        assert_eq!(
            basic_check(tokens),
            Err("Expected num or expr behind '(', found operator: '-'".to_string())
        );

        // 测试右括号后跟数字的错误
        let tokens = Tokenizer::from("13.0 * 1* ( 1-4) 3.0 ").collect();
        assert_eq!(
            basic_check(tokens),
            Err("Expected operator or eof behind ')', found num: '3.0'".to_string())
        );

        // 测试两个连续数字的错误
        let tokens = Tokenizer::from("13.0  1 --/ ").collect();
        assert_eq!(
            basic_check(tokens),
            Err("Expected operator or eof behind num: '13.0', found num: '1'".to_string())
        );

        // 测试以非正负号符号开头的错误
        let tokens = Tokenizer::from("*13.0  1 --/ ").collect();
        assert_eq!(
            basic_check(tokens),
            Err("Expected a num or '(' to start an expr, found '*'".to_string())
        );
    }

    #[test]
    fn test_get_rpn_conversion() {
        use super::Operator::*;
        use super::Token::*;
        // 测试基本表达式
        let tokens = vec![Num(dec!(1)), Op(Add), Num(dec!(2)), Token::EOF];
        let rpn = get_rpn(tokens).unwrap();
        assert_eq!(rpn, vec![Num(dec!(1)), Num(dec!(2)), Op(Add)]);

        // 测试运算符优先级 1 + 2 * 3 => 1 2 3 * +
        let tokens = vec![
            Num(dec!(1)),
            Op(Add),
            Num(dec!(2)),
            Op(Mul),
            Num(dec!(3)),
            Token::EOF,
        ];
        let rpn = get_rpn(tokens).unwrap();
        assert_eq!(
            rpn,
            vec![Num(dec!(1)), Num(dec!(2)), Num(dec!(3)), Op(Mul), Op(Add)]
        );

        // 测试括号改变优先级 (1 + 2) * 3 => 1 2 + 3 *
        let tokens = vec![
            Paren('('),
            Num(dec!(1)),
            Op(Add),
            Num(dec!(2)),
            Paren(')'),
            Op(Mul),
            Num(dec!(3)),
            Token::EOF,
        ];
        let rpn = get_rpn(tokens).unwrap();
        assert_eq!(
            rpn,
            vec![Num(dec!(1)), Num(dec!(2)), Op(Add), Num(dec!(3)), Op(Mul)]
        );

        // 测试幂运算的右结合性 2 ^ 3 ^ 4 => 2 3 4 ^ ^
        let tokens = vec![
            Num(dec!(2)),
            Op(Pow),
            Num(dec!(3)),
            Op(Pow),
            Num(dec!(4)),
            Token::EOF,
        ];
        let rpn = get_rpn(tokens).unwrap();
        assert_eq!(
            rpn,
            vec![Num(dec!(2)), Num(dec!(3)), Num(dec!(4)), Op(Pow), Op(Pow)]
        );

        // 测试复杂表达式 3 * (4 + 5) - 2 ^ 3 => 3 4 5 + * 2 3 ^ -
        let tokens = vec![
            Num(dec!(3)),
            Op(Mul),
            Paren('('),
            Num(dec!(4)),
            Op(Add),
            Num(dec!(5)),
            Paren(')'),
            Op(Sub),
            Num(dec!(2)),
            Op(Pow),
            Num(dec!(3)),
            Token::EOF,
        ];
        let rpn = get_rpn(tokens).unwrap();
        assert_eq!(
            rpn,
            vec![
                Num(dec!(3)),
                Num(dec!(4)),
                Num(dec!(5)),
                Op(Add),
                Op(Mul),
                Num(dec!(2)),
                Num(dec!(3)),
                Op(Pow),
                Op(Sub)
            ]
        );
    }

    // 测试parse构建AST的正确性
    #[test]
    fn test_parse_ast_construction() {
        use super::Operator::*;
        use super::Token::*;
        // 测试简单加法
        let tokens = vec![Num(dec!(1)), Num(dec!(2)), Op(Add), EOF];
        let ast = parse(tokens).unwrap();
        assert_eq!(
            ast,
            Ast(Node::Add(
                Box::new(Node::Num(dec!(1))),
                Box::new(Node::Num(dec!(2)))
            ))
        );

        // 测试混合运算
        let tokens = vec![
            Num(dec!(2)),
            Num(dec!(3)),
            Num(dec!(4)),
            Op(Mul),
            Op(Add),
            EOF,
        ];
        let ast = parse(tokens).unwrap();
        assert_eq!(
            ast,
            Ast(Node::Add(
                Box::new(Node::Num(dec!(2))),
                Box::new(Node::Mul(
                    Box::new(Node::Num(dec!(3))),
                    Box::new(Node::Num(dec!(4)))
                ))
            ))
        );

        // 测试带括号的表达式
        let tokens = vec![
            Num(dec!(1)),
            Num(dec!(2)),
            Op(Add),
            Num(dec!(3)),
            Op(Mul),
            EOF,
        ];
        let ast = parse(tokens).unwrap();
        assert_eq!(
            ast,
            Ast(Node::Mul(
                Box::new(Node::Add(
                    Box::new(Node::Num(dec!(1))),
                    Box::new(Node::Num(dec!(2)))
                )),
                Box::new(Node::Num(dec!(3)))
            ))
        );

        // 测试幂运算
        let tokens = vec![Num(dec!(2)), Num(dec!(3)), Op(Pow), EOF];
        let ast = parse(tokens).unwrap();
        assert_eq!(
            ast,
            Ast(Node::Pow(
                Box::new(Node::Num(dec!(2))),
                Box::new(Node::Num(dec!(3)))
            ))
        );

        // 测试复杂嵌套结构
        let tokens = vec![
            Num(dec!(3)),
            Num(dec!(4)),
            Num(dec!(5)),
            Op(Add),
            Op(Mul),
            Num(dec!(2)),
            Num(dec!(3)),
            Op(Pow),
            Op(Sub),
            EOF,
        ];
        let ast = parse(tokens).unwrap();
        assert_eq!(
            ast,
            Ast(Node::Sub(
                Box::new(Node::Mul(
                    Box::new(Node::Num(dec!(3))),
                    Box::new(Node::Add(
                        Box::new(Node::Num(dec!(4))),
                        Box::new(Node::Num(dec!(5)))
                    ))
                )),
                Box::new(Node::Pow(
                    Box::new(Node::Num(dec!(2))),
                    Box::new(Node::Num(dec!(3)))
                ))
            ))
        );
    }

    // 测试端到端解析流程
    #[test]
    fn test_end_to_end_parsing() {
        // 简单表达式
        let ast = Parser::parse("1 + 2").unwrap();
        assert_eq!(
            ast,
            Ast(Node::Add(
                Box::new(Node::Num(dec!(1))),
                Box::new(Node::Num(dec!(2)))
            ))
        );

        // 带括号的表达式
        let ast = Parser::parse("(3 + 4) * 5").unwrap();
        assert_eq!(
            ast,
            Ast(Node::Mul(
                Box::new(Node::Add(
                    Box::new(Node::Num(dec!(3))),
                    Box::new(Node::Num(dec!(4)))
                )),
                Box::new(Node::Num(dec!(5)))
            ))
        );

        // 带负号的表达式
        let ast = Parser::parse("-5 + --3").unwrap();
        assert_eq!(
            ast,
            Ast(Node::Add(
                Box::new(Node::Num(dec!(-5))),
                Box::new(Node::Num(dec!(3)))
            ))
        );

        // 幂运算表达式
        let ast = Parser::parse("2 ^ 3 ^ 2").unwrap();
        assert_eq!(
            ast,
            Ast(Node::Pow(
                Box::new(Node::Num(dec!(2))),
                Box::new(Node::Pow(
                    Box::new(Node::Num(dec!(3))),
                    Box::new(Node::Num(dec!(2)))
                ))
            ))
        );

        // 复杂表达式
        let ast = Parser::parse("3 * (4 + 5) - 2 ^ 3").unwrap();
        assert_eq!(
            ast,
            Ast(Node::Sub(
                Box::new(Node::Mul(
                    Box::new(Node::Num(dec!(3))),
                    Box::new(Node::Add(
                        Box::new(Node::Num(dec!(4))),
                        Box::new(Node::Num(dec!(5)))
                    ))
                )),
                Box::new(Node::Pow(
                    Box::new(Node::Num(dec!(2))),
                    Box::new(Node::Num(dec!(3)))
                ))
            ))
        );
    }
}
