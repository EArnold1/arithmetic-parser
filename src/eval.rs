use crate::parser::{Expr, Op};

#[derive(Debug, PartialEq)]
pub enum EvalError {
    DivisionByZero,
}

/// Evaluates an expression tree to its `i64` value.
///
/// The tree is walked recursively: a `Number` is its own value, `Negate`
/// flips the sign of its child, and `Binary` evaluates both sides and then
/// applies the operator. Precedence and grouping are already encoded in the
/// shape of the tree, so `eval` never has to think about them.
///
/// # Errors
/// Returns `EvalError::DivisionByZero` if any division has a right-hand side
/// that evaluates to `0`.
pub fn eval(tree: Expr) -> Result<i64, EvalError> {
    match tree {
        Expr::Number(n) => Ok(n),
        Expr::Negate(v) => {
            let value = eval(*v)?;
            Ok(-value)
        }
        Expr::Binary(l, op, r) => {
            let left = eval(*l)?;
            let right = eval(*r)?;

            let calculation = match op {
                Op::Add => left + right,
                Op::Sub => left - right,
                Op::Div => {
                    if right == 0 {
                        return Err(EvalError::DivisionByZero);
                    }
                    left / right
                }
                Op::Mul => left * right,
            };

            Ok(calculation)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Token;
    use crate::parser::Parser;

    fn num(n: i64) -> Box<Expr> {
        Box::new(Expr::Number(n))
    }

    fn bin(left: Box<Expr>, op: Op, right: Box<Expr>) -> Box<Expr> {
        Box::new(Expr::Binary(left, op, right))
    }

    fn neg(value: Box<Expr>) -> Box<Expr> {
        Box::new(Expr::Negate(value))
    }

    #[test]
    fn eval_applies_each_binary_operator() {
        assert_eq!(eval(*bin(num(6), Op::Add, num(3))), Ok(9));
        assert_eq!(eval(*bin(num(6), Op::Sub, num(3))), Ok(3));
        assert_eq!(eval(*bin(num(6), Op::Mul, num(3))), Ok(18));
        assert_eq!(eval(*bin(num(6), Op::Div, num(3))), Ok(2));
    }

    #[test]
    fn eval_negate_flips_sign_each_time_it_is_applied() {
        assert_eq!(eval(*neg(num(5))), Ok(-5));
        assert_eq!(eval(*neg(neg(num(5)))), Ok(5));
    }

    #[test]
    fn eval_result_depends_on_tree_shape_not_operand_order() {
        // (10 - 4) - 3
        let left_grouped = bin(bin(num(10), Op::Sub, num(4)), Op::Sub, num(3));
        // 10 - (4 - 3)
        let right_grouped = bin(num(10), Op::Sub, bin(num(4), Op::Sub, num(3)));

        assert_eq!(eval(*left_grouped), Ok(3));
        assert_eq!(eval(*right_grouped), Ok(9));
    }

    #[test]
    fn eval_division_truncates_toward_zero() {
        assert_eq!(eval(*bin(num(7), Op::Div, num(2))), Ok(3));
        assert_eq!(eval(*bin(neg(num(7)), Op::Div, num(2))), Ok(-3));
    }

    #[test]
    fn eval_catches_zero_divisors_that_are_not_literals_or_are_nested() {
        // 1 / (2 - 2): the divisor only becomes zero once evaluated
        let computed_zero = bin(num(1), Op::Div, bin(num(2), Op::Sub, num(2)));
        // 1 + (4 / 0): the error must bubble up through the outer addition
        let nested = bin(num(1), Op::Add, bin(num(4), Op::Div, num(0)));

        assert_eq!(eval(*computed_zero), Err(EvalError::DivisionByZero));
        assert_eq!(eval(*nested), Err(EvalError::DivisionByZero));
    }

    #[test]
    fn eval_rejects_division_by_zero() {
        let mut parser = Parser::new(vec![Token::Number(7), Token::Divide, Token::Number(0)]);
        let expr = parser.parse().unwrap();

        assert_eq!(eval(expr), Err(EvalError::DivisionByZero))
    }
}
