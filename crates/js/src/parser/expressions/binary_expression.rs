use crate::parser::{
    expressions::UpdateExpression,
    tokenization::{Punctuator, SkipLineTerminators, Token, Tokenizer},
    SyntaxError,
};

use super::{Expression, UnaryExpression};

#[derive(Clone, Debug)]
pub struct BinaryExpression {
    pub op: BinaryOp,
    pub lhs: Box<Expression>,
    pub rhs: Box<Expression>,
}

impl BinaryExpression {
    #[must_use]
    pub fn operator(&self) -> BinaryOp {
        self.op
    }

    #[must_use]
    pub fn left_hand_side(&self) -> &Expression {
        &self.lhs
    }

    #[must_use]
    pub fn right_hand_side(&self) -> &Expression {
        &self.rhs
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOp {
    Arithmetic(ArithmeticOp),
    Logical(LogicalOp),
    Bitwise(BitwiseOp),
    Equality(EqualityOp),
    Relational(RelationalOp),
    Shift(ShiftOp),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArithmeticOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Exponentiation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogicalOp {
    Or,
    And,
    Coalesce,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BitwiseOp {
    Or,
    And,
    Xor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EqualityOp {
    Equal,
    NotEqual,
    StrictEqual,
    StrictNotEqual,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelationalOp {
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShiftOp {
    ShiftLeft,
    ShiftRight,
    ShiftRightZeros,
}

macro_rules! binary_op {
    ($docs: expr, $name: ident<$(const $const_ident: ident:$const_type:ty,)*>, $next: path, $($symbol: pat => $op: path,)*) => {
        #[doc = $docs]
        pub(super) fn $name<$(const $const_ident: $const_type,)*>(
            tokenizer: &mut Tokenizer<'_>,
        ) -> Result<Expression, SyntaxError> {
            let mut expression: Expression = $next(tokenizer)?.into();

            loop {
                let operator = match tokenizer.peek(0, SkipLineTerminators::Yes)? {
                    $(Some(Token::Punctuator($symbol)) => {
                        tokenizer.advance(1);
                        $op
                    },)*
                    _ => break
                };

                let rhs = $next(tokenizer)?.into();

                expression = BinaryExpression {
                    op: operator.into(),
                    lhs: Box::new(expression),
                    rhs: Box::new(rhs),
                }
                .into();
            }

            Ok(expression)
        }
    };
}

binary_op!(
    "<https://262.ecma-international.org/14.0/#prod-LogicalANDExpression>",
    parse_logical_and_expression<const IN: bool, const YIELD: bool, const AWAIT: bool,>,
    parse_bitwise_or_expression::<IN, YIELD, AWAIT>,
    Punctuator::DoubleAmpersand => LogicalOp::And,
);

binary_op!(
    "<https://262.ecma-international.org/14.0/#prod-BitwiseORExpression>",
    parse_bitwise_or_expression<const IN: bool, const YIELD: bool, const AWAIT: bool,>,
    parse_bitwise_xor_expression::<IN, YIELD, AWAIT>,
    Punctuator::VerticalBar => BitwiseOp::Or,
);

binary_op!(
    "<https://262.ecma-international.org/14.0/#prod-BitwiseXORExpression>",
    parse_bitwise_xor_expression<const IN: bool, const YIELD: bool, const AWAIT: bool,>,
    parse_bitwise_and_expression::<IN, YIELD, AWAIT>,
    Punctuator::Caret => BitwiseOp::Xor,
);

binary_op!(
    "<https://262.ecma-international.org/14.0/#prod-BitwiseANDExpression>",
    parse_bitwise_and_expression<const IN: bool, const YIELD: bool, const AWAIT: bool,>,
    parse_equality_expression::<IN, YIELD, AWAIT>,
    Punctuator::Ampersand => BitwiseOp::And,
);

binary_op!(
    "<https://262.ecma-international.org/14.0/#prod-EqualityExpression>",
    parse_equality_expression<const IN: bool, const YIELD: bool, const AWAIT: bool,>,
    parse_relational_expression::<IN, YIELD, AWAIT>,
    Punctuator::DoubleEqual => EqualityOp::Equal,
    Punctuator::TripleEqual => EqualityOp::StrictEqual,
    Punctuator::ExclamationMarkEqual => EqualityOp::NotEqual,
    Punctuator::ExclamationMarkDoubleEqual => EqualityOp::StrictNotEqual,
);

binary_op!(
    "<https://262.ecma-international.org/14.0/#prod-RelationalExpression>",
    parse_relational_expression<const IN: bool, const YIELD: bool, const AWAIT: bool,>,
    parse_shift_expression::<YIELD, AWAIT>,
    Punctuator::LessThan => RelationalOp::LessThan,
    Punctuator::GreaterThan => RelationalOp::GreaterThan,
    Punctuator::LessThanEqual => RelationalOp::LessThanOrEqual,
    Punctuator::GreaterThanEqual => RelationalOp::GreaterThanOrEqual,
);

binary_op!(
    "<https://262.ecma-international.org/14.0/#prod-RelationalExpression>",
    parse_shift_expression<const YIELD: bool, const AWAIT: bool,>,
    parse_additive_expression::<YIELD, AWAIT>,
    Punctuator::DoubleLessThan => ShiftOp::ShiftLeft,
    Punctuator::DoubleGreaterThan => ShiftOp::ShiftRight,
    Punctuator::DoubleGreaterThanEqual => ShiftOp::ShiftRightZeros,
);

binary_op!(
    "<https://262.ecma-international.org/14.0/#prod-AdditiveExpression>",
    parse_additive_expression<const YIELD: bool, const AWAIT: bool,>,
    parse_multiplicative_expression::<YIELD, AWAIT>,
    Punctuator::Plus => ArithmeticOp::Add,
    Punctuator::Minus => ArithmeticOp::Subtract,
);

binary_op!(
    "<https://262.ecma-international.org/14.0/#prod-MultiplicativeExpression>",
    parse_multiplicative_expression<const YIELD: bool, const AWAIT: bool,>,
    parse_exponentiation_expression::<YIELD, AWAIT>,
    Punctuator::Asterisk => ArithmeticOp::Multiply,
    Punctuator::Slash => ArithmeticOp::Divide,
    Punctuator::Percent => ArithmeticOp::Modulo,
);

/// <https://262.ecma-international.org/14.0/#prod-ExponentiationExpression>
pub fn parse_exponentiation_expression<const YIELD: bool, const AWAIT: bool>(
    tokenizer: &mut Tokenizer<'_>,
) -> Result<Expression, SyntaxError> {
    // NOTE: This function cannot be defined with the macro above since it can contain either UpdateExpressions
    //       or UnaryExpressions
    let Some(next_token) = tokenizer.peek(0, SkipLineTerminators::Yes)? else {
        return Err(tokenizer.syntax_error("expected more tokens"));
    };

    let is_unary_expression = match next_token {
        Token::Punctuator(
            Punctuator::Plus | Punctuator::Minus | Punctuator::Tilde | Punctuator::ExclamationMark,
        ) => true,
        Token::Identifier(ident) if matches!(ident.as_str(), "delete" | "void" | "typeof") => true,
        _ => false,
    };

    let exponentiation_expression = if is_unary_expression {
        UnaryExpression::parse::<YIELD, AWAIT>(tokenizer)?
    } else {
        let mut expression = UpdateExpression::parse::<YIELD, AWAIT>(tokenizer)?;

        if tokenizer
            .peek(0, SkipLineTerminators::Yes)?
            .is_some_and(|t| t.is_punctuator(Punctuator::DoubleAsterisk))
        {
            tokenizer.advance(1);

            let exponentiation_expression =
                parse_exponentiation_expression::<YIELD, AWAIT>(tokenizer)?;
            expression = BinaryExpression {
                op: BinaryOp::Arithmetic(ArithmeticOp::Exponentiation),
                lhs: Box::new(expression),
                rhs: Box::new(exponentiation_expression),
            }
            .into();
        }

        expression
    };

    Ok(exponentiation_expression)
}

impl From<ArithmeticOp> for BinaryOp {
    fn from(value: ArithmeticOp) -> Self {
        Self::Arithmetic(value)
    }
}

impl From<LogicalOp> for BinaryOp {
    fn from(value: LogicalOp) -> Self {
        Self::Logical(value)
    }
}

impl From<BitwiseOp> for BinaryOp {
    fn from(value: BitwiseOp) -> Self {
        Self::Bitwise(value)
    }
}

impl From<EqualityOp> for BinaryOp {
    fn from(value: EqualityOp) -> Self {
        Self::Equality(value)
    }
}

impl From<RelationalOp> for BinaryOp {
    fn from(value: RelationalOp) -> Self {
        Self::Relational(value)
    }
}

impl From<ShiftOp> for BinaryOp {
    fn from(value: ShiftOp) -> Self {
        Self::Shift(value)
    }
}
