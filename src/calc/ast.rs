use rust_decimal::{Decimal, MathematicalOps};

#[derive(Debug, PartialEq, Clone)]
pub struct Ast(pub Node);

#[derive(Debug, PartialEq, Clone)]
pub enum Node {
    Add(Box<Node>, Box<Node>),
    Sub(Box<Node>, Box<Node>),
    Mul(Box<Node>, Box<Node>),
    Div(Box<Node>, Box<Node>),
    Pow(Box<Node>, Box<Node>),
    Num(Decimal),
}

impl Node {
    pub fn eval(&self) -> Decimal {
        use Node::*;
        match self {
            Add(lhs, rhs) => lhs.eval() + rhs.eval(),
            Sub(lhs, rhs) => lhs.eval() - rhs.eval(),
            Mul(lhs, rhs) => lhs.eval() * rhs.eval(),
            Div(lhs, rhs) => lhs.eval() / rhs.eval(),
            Pow(lhs, rhs) => lhs.eval().powd(rhs.eval()),
            Num(val) => *val
        }
    }
}

impl Ast {
    pub fn eval(&self) -> Decimal {
        self.0.eval()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use rust_decimal::dec;
    use Node::*;

    // 辅助函数：创建数字节点
    fn num(n: i64) -> Node {
        Node::Num(Decimal::new(n, 0))
    }

    // 辅助函数：创建小数节点
    fn dec(s: &str) -> Node {
        Node::Num(Decimal::from_str_exact(s).unwrap())
    }

    #[test]
    fn test_basic_operations() {
        // 加法
        let ast = Ast(Add(Box::new(num(2)), Box::new(num(3))));
        assert_eq!(ast.eval(), dec!(5));
        
        // 减法
        let ast = Ast(Sub(Box::new(num(5)), Box::new(num(3))));
        assert_eq!(ast.eval(), dec!(2));
        
        // 乘法
        let ast = Ast(Mul(Box::new(num(4)), Box::new(num(3))));
        assert_eq!(ast.eval(), dec!(12));
        
        // 除法
        let ast = Ast(Div(Box::new(num(10)), Box::new(num(4))));
        assert_eq!(ast.eval(), dec!(2.5));
        
        // 幂运算
        let ast = Ast(Pow(Box::new(num(2)), Box::new(num(3))));
        assert_eq!(ast.eval(), dec!(8));
    }

    #[test]
    fn test_nested_operations() {
        // 混合运算: (2 + 3) * 4
        let ast = Ast(Mul(
            Box::new(Add(Box::new(num(2)), Box::new(num(3)))),
            Box::new(num(4))
        ));
        assert_eq!(ast.eval(), dec!(20));
        
        // 多层嵌套: 2^3 + 4 * (5 - 3)
        let ast = Ast(Add(
            Box::new(Pow(Box::new(num(2)), Box::new(num(3)))),
            Box::new(Mul(
                Box::new(num(4)),
                Box::new(Sub(Box::new(num(5)), Box::new(num(3))))
            ))
        ));
        assert_eq!(ast.eval(), dec!(16));
    }

    #[test]
    fn test_decimal_operations() {
        // 小数加法
        let ast = Ast(Add(
            Box::new(Node::Num(dec!(0.1))),
            Box::new(Node::Num(dec!(0.2)))
        ));
        assert_eq!(ast.eval(), dec!(0.3));
        
        // 小数除法
        let ast = Ast(Div(
            Box::new(num(1)),
            Box::new(num(8))
        ));
        assert_eq!(ast.eval(), dec!(0.125));
        
        // 小数幂运算
        let ast = Ast(Pow(
            Box::new(Node::Num(dec!(4.0))),
            Box::new(Node::Num(dec!(0.5)))
        ));
        assert!(ast.eval() - dec!(2.0) < dec!(0.000000001));
    }

    #[test]
    fn test_edge_cases() {
        // 零的负指数幂
        
        // 负数的分数幂
        let ast = Ast(Pow(
            Box::new(num(-8)),
            Box::new(Node::Num(dec!(1) / dec!(3))))
        );
        assert!(ast.eval() - dec!(-2) < dec!(0.00000001));
        
        // 大数运算
        let ast = Ast(Mul(
            Box::new(Node::Num(dec!(100_0000_0000_0000))),
            Box::new(Node::Num(dec!(100_0000_0000_0000)))
        ));
        assert_eq!(ast.eval(), dec!(1_0000_0000_0000_0000_0000_0000_0000));
    }

    #[test]
    fn test_associativity() {
        // 左结合: 10 - 5 - 2
        let ast = Ast(Sub(
            Box::new(Sub(Box::new(num(10)), Box::new(num(5)))),
            Box::new(num(2))
        ));
        assert_eq!(ast.eval(), dec!(3));
        
        // 右结合: 2^3^2
        let ast = Ast(Pow(
            Box::new(num(2)),
            Box::new(Pow(Box::new(num(3)), Box::new(num(2))))
        ));
        assert_eq!(ast.eval(), dec!(512));
    }

    #[test]
    fn test_negative_numbers() {
        // 负数的加法
        let ast = Ast(Add(
            Box::new(num(-5)),
            Box::new(num(8))
        ));
        assert_eq!(ast.eval(), dec!(3));
        
        // 负数的乘法
        let ast = Ast(Mul(
            Box::new(num(-4)),
            Box::new(num(3))
        ));
        assert_eq!(ast.eval(), dec!(-12));
        
        // 负数的幂运算
        let ast = Ast(Pow(
            Box::new(num(-2)),
            Box::new(num(3))
        ));
        assert_eq!(ast.eval(), dec!(-8));
    }

    #[test]
    fn test_complex_expressions() {
        // 复杂表达式: (3.5 + 2.5) * (4 - 1) / 2^2
        let ast = Ast(Div(
            Box::new(Mul(
                Box::new(Add(Box::new(dec("3.5")), Box::new(dec("2.5")))),
                Box::new(Sub(Box::new(num(4)), Box::new(num(1))))
            )),
            Box::new(Pow(Box::new(num(2)), Box::new(num(2))))
        ));
        assert_eq!(ast.eval(), dec!(4.5));
        
        // 带负数的复杂表达式: -2 * (3 + -4)^2
        let ast = Ast(Mul(
            Box::new(num(-2)),
            Box::new(Pow(
                Box::new(Add(Box::new(num(3)), Box::new(num(-4)))),
                Box::new(num(2))
            ))
        ));
        assert_eq!(ast.eval(), dec!(-2));
    }
    
    #[test]
    fn test_deep_nesting() {
        // 深度嵌套的表达式: ((((1 + 2) * 3) - 4) / 5)^2
        let ast = Ast(Pow(
            Box::new(Div(
                Box::new(Sub(
                    Box::new(Mul(
                        Box::new(Add(Box::new(num(1)), Box::new(num(2)))),
                        Box::new(num(3))
                    )),
                    Box::new(num(4))
                )),
                Box::new(num(5))
            )),
            Box::new(num(2))
        ));
        assert_eq!(ast.eval(), dec!(1));
    }
}