use expr::*;

#[derive(Debug, Clone)]
pub struct Query {
    compiled: Expr,
}

impl Query {
    pub fn new(input: &str) -> Result<Self, String> {
        use nom::{character::complete::multispace0, combinator::eof, sequence::terminated};

        let try_full_input = terminated(terminated(Expr::parse, multispace0), eof)(input);

        let compiled = match try_full_input {
            Ok(("", expr)) => Ok(expr),
            Ok((remaining, _)) => Err(format!(
                "Error parsing Query: unexpected continuation: ‘{}’",
                remaining
            )),
            Err(nom::Err::Error(err)) | Err(nom::Err::Failure(err)) => Err(format!(
                "Error parsing Query: {}",
                nom::error::convert_error(input, err)
            )),
            Err(err) => Err(format!("Error parsing Query: {}", err.to_string())),
        }?;

        Ok(Self { compiled })
    }

    pub fn evaluate_on(&self, element: &crate::elements::Element) -> Result<bool, String> {
        match eval::Value::eval(&self.compiled, element)? {
            eval::Value::Bool(b) => Ok(b),
            other => Err(format!(
                "Query did not evaluate to a boolean value but to {:?}.",
                other
            )),
        }
    }
}

mod eval {
    use std::collections::BTreeSet;

    #[derive(PartialEq, Eq, Debug, Clone, PartialOrd, Ord)]
    pub enum Value {
        Bool(bool),
        Int(i32),
        Set(BTreeSet<Value>),
    }

    use super::expr;
    use super::expr::Expr;

    impl Value {
        pub fn eval(expr: &Expr, element: &crate::elements::Element) -> Result<Self, String> {
            //Ok(Self::Bool(false))

            fn bset_to_value(xs: &BTreeSet<i8>) -> Value {
                Value::Set(xs.iter().map(|&a| Value::Int(a as i32)).collect())
            }

            match expr {
                Expr::LBool(a) => Ok(Value::Bool(*a)),
                Expr::LInt(a) => Ok(Value::Int(*a)),
                Expr::Symbol(symb) => match symb.as_str() {
                    "atomic_number" => Ok(Value::Int(element.atomic_number as i32)),
                    "z" => Ok(Value::Int(element.atomic_number as i32)),
                    "group" => Ok(Value::Int(element.group.map_or(-1, |a| a as i32))),
                    "period" => Ok(Value::Int(element.period as i32)),
                    "block" => Ok(Value::Int(element.block as i32)),
                    "oxidation_states.common" => {
                        Ok(bset_to_value(&element.oxidation_states.common))
                    }
                    "oxidation_states.notable" => {
                        Ok(bset_to_value(&element.oxidation_states.notable))
                    }
                    "oxidation_states.predicted" => {
                        Ok(bset_to_value(&element.oxidation_states.predicted))
                    }
                    "oxidation_states.citation_needed" => {
                        Ok(bset_to_value(&element.oxidation_states.citation_needed))
                    }
                    other => Err(format!("Eval: unknown symbol: {}", other)),
                },
                Expr::LSet(subexprs) => Ok(Value::Set(
                    subexprs
                        .into_iter()
                        .map(|sx| Value::eval(sx, element))
                        .collect::<Result<BTreeSet<_>, _>>()?,
                )),
                Expr::UnaryOp(op, subexpr) => {
                    let subval = Value::eval(subexpr, element)?;
                    match (op, subval) {
                        (expr::UnaryOperator::Not, Value::Bool(a)) => Ok(Value::Bool(!a)),
                        (expr::UnaryOperator::Minus, Value::Int(a)) => Ok(Value::Int(-a)),
                        (op, other) => Err(format!(
                            "Eval: unary operator {:?} does not apply to {:?}",
                            op, other
                        )),
                    }
                }
                Expr::BinaryOp(op, subexpr_l, subexpr_r) => {
                    let subval_l = Value::eval(subexpr_l, element)?;
                    let subval_r = Value::eval(subexpr_r, element)?;
                    use expr::BinaryOperator::*;
                    use Value::*;
                    match (op, subval_l, subval_r) {
                        (Or, Bool(l), Bool(r)) => Ok(Bool(l || r)),
                        (And, Bool(l), Bool(r)) => Ok(Bool(l && r)),
                        (Equal, Bool(l), Bool(r)) => Ok(Bool(l == r)),
                        (Equal, Int(l), Int(r)) => Ok(Bool(l == r)),
                        (Equal, Set(l), Set(r)) => Ok(Bool(l == r)),
                        (NotEqual, Bool(l), Bool(r)) => Ok(Bool(l != r)),
                        (NotEqual, Int(l), Int(r)) => Ok(Bool(l != r)),
                        (NotEqual, Set(l), Set(r)) => Ok(Bool(l != r)),
                        (LessThan, Int(l), Int(r)) => Ok(Bool(l < r)),
                        (LessEqual, Int(l), Int(r)) => Ok(Bool(l <= r)),
                        (GreaterThan, Int(l), Int(r)) => Ok(Bool(l > r)),
                        (GreaterEqual, Int(l), Int(r)) => Ok(Bool(l >= r)),
                        (Plus, Int(l), Int(r)) => Ok(Int(l + r)),
                        (Minus, Int(l), Int(r)) => Ok(Int(l - r)),
                        (Multiply, Int(l), Int(r)) => Ok(Int(l * r)),
                        (Divide, Int(l), Int(r)) => Ok(Int(l / r)),
                        (Plus, Set(l), Set(r)) => Ok(Set(l.union(&r).cloned().collect())),
                        (Minus, Set(l), Set(r)) => Ok(Set(l.difference(&r).cloned().collect())),
                        (InSet, l @ Int(_), Set(r)) => Ok(Bool(r.contains(&l))),
                        (InSet, l @ Bool(_), Set(r)) => Ok(Bool(r.contains(&l))),
                        // Here’s a little inconsequency, because we allow sets of sets… But well.
                        (InSet, Set(l), Set(r)) => Ok(Bool(r.is_superset(&l))),
                        (op, other_l, other_r) => Err(format!(
                            "Eval: binary operator {:?} does not apply to ({:?}, {:?})",
                            op, other_l, other_r
                        )),
                    }
                }
            }
        }
    }
}

mod expr {
    #[derive(PartialEq, Eq, Debug, Clone)]
    pub enum Expr {
        // Literals:
        LBool(bool),
        LInt(i32),
        LSet(Vec<Expr>),
        // Symbols, e.g. "oxidation_states.notable":
        Symbol(String),
        // Operators:
        BinaryOp(BinaryOperator, Box<Expr>, Box<Expr>),
        UnaryOp(UnaryOperator, Box<Expr>),
    }

    #[derive(PartialEq, Eq, Debug, Clone, Copy)]
    pub enum BinaryOperator {
        // And/or:
        Or,
        And,
        // Comparison:
        Equal,
        NotEqual,
        LessThan,
        LessEqual,
        GreaterThan,
        GreaterEqual,
        // Set:
        InSet,
        // Arithmetic:
        Plus,
        Minus,
        Multiply,
        Divide,
    }

    #[derive(PartialEq, Eq, Debug, Clone, Copy)]
    pub enum UnaryOperator {
        Not,
        Minus,
    }

    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::{alpha1, alphanumeric1, multispace0},
        combinator::{map, recognize, value},
        error::VerboseError,
        multi::{fold_many0, many0_count, separated_list0},
        sequence::{delimited, pair, preceded},
        IResult,
    };

    type IR<'a, A> = IResult<&'a str, A, VerboseError<&'a str>>;

    impl Expr {
        pub fn parse(input: &str) -> IR<Self> {
            Self::or(input)
        }

        fn or(input: &str) -> IR<Self> {
            Self::binary_op("||", BinaryOperator::Or, Self::and)(input)
        }

        fn and(input: &str) -> IR<Self> {
            Self::binary_op("&&", BinaryOperator::And, Self::equal)(input)
        }

        fn equal(input: &str) -> IR<Self> {
            Self::binary_op("==", BinaryOperator::Equal, Self::not_equal)(input)
        }

        fn not_equal(input: &str) -> IR<Self> {
            Self::binary_op("!=", BinaryOperator::NotEqual, Self::less_than)(input)
        }

        fn less_than(input: &str) -> IR<Self> {
            Self::binary_op("<", BinaryOperator::LessThan, Self::less_equal)(input)
        }

        fn less_equal(input: &str) -> IR<Self> {
            Self::binary_op("<=", BinaryOperator::LessEqual, Self::greater_than)(input)
        }

        fn greater_than(input: &str) -> IR<Self> {
            Self::binary_op(">", BinaryOperator::GreaterThan, Self::greater_equal)(input)
        }

        fn greater_equal(input: &str) -> IR<Self> {
            Self::binary_op(">=", BinaryOperator::GreaterEqual, Self::in_set)(input)
        }

        fn in_set(input: &str) -> IR<Self> {
            Self::binary_op("in", BinaryOperator::InSet, Self::plus)(input)
        }

        fn plus(input: &str) -> IR<Self> {
            Self::binary_op("+", BinaryOperator::Plus, Self::minus)(input)
        }

        fn minus(input: &str) -> IR<Self> {
            Self::binary_op("-", BinaryOperator::Minus, Self::multiply)(input)
        }

        fn multiply(input: &str) -> IR<Self> {
            Self::binary_op("*", BinaryOperator::Multiply, Self::divide)(input)
        }

        fn divide(input: &str) -> IR<Self> {
            Self::binary_op("/", BinaryOperator::Divide, Self::not)(input)
        }

        fn not(input: &str) -> IR<Self> {
            Self::unary_op("!", UnaryOperator::Not, Self::unary_minus)(input)
        }

        fn unary_minus(input: &str) -> IR<Self> {
            Self::unary_op("-", UnaryOperator::Minus, Self::parens)(input)
        }

        fn parens(input: &str) -> IR<Self> {
            alt((
                delimited(
                    preceded(multispace0, tag("(")),
                    Self::parse,
                    preceded(multispace0, tag(")")),
                ),
                Self::literal_set,
                Self::literal_bool,
                Self::literal_int,
                Self::symbol,
            ))(input)
        }

        fn literal_bool(input: &str) -> IR<Self> {
            alt((
                value(Self::LBool(true), preceded(multispace0, tag("true"))),
                value(Self::LBool(false), preceded(multispace0, tag("false"))),
            ))(input)
        }

        fn literal_int(input: &str) -> IR<Self> {
            map(preceded(multispace0, nom::character::complete::i32), |i| {
                Self::LInt(i)
            })(input)
        }

        fn symbol(input: &str) -> IR<Self> {
            fn part(input: &str) -> IR<&str> {
                recognize(pair(alpha1, many0_count(alt((alphanumeric1, tag("_"))))))(input)
            }
            map(
                preceded(
                    multispace0,
                    recognize(pair(part, many0_count(pair(tag("."), part)))),
                ),
                |s| Self::Symbol(s.to_string()),
            )(input)
        }

        fn literal_set(input: &str) -> IR<Self> {
            delimited(
                preceded(multispace0, tag("{")),
                map(separated_list0(tag(","), Self::parse), |xs| Self::LSet(xs)),
                preceded(multispace0, tag("}")),
            )(input)
        }

        // ---------------------- operator helpers ---------------------- //

        fn binary_op<'a, F>(
            op_tag: &'a str,
            op_variant: BinaryOperator,
            mut lower_precedence: F,
        ) -> impl FnMut(&'a str) -> IR<Self>
        where
            F: FnMut(&'a str) -> IR<Self> + Copy,
        {
            move |input| {
                let (input, init) = lower_precedence(input)?;
                fold_many0(
                    preceded(preceded(multispace0, tag(op_tag)), lower_precedence),
                    move || init.clone(),
                    |acc, item| Self::BinaryOp(op_variant, Box::new(acc), Box::new(item)),
                )(input)
            }
        }

        fn unary_op<'a, F>(
            op_tag: &'a str,
            op_variant: UnaryOperator,
            lower_precedence: F,
        ) -> impl FnMut(&'a str) -> IR<Self>
        where
            F: FnMut(&'a str) -> IR<Self> + Copy,
        {
            move |input| {
                alt((
                    map(
                        preceded(multispace0, preceded(tag(op_tag), lower_precedence)),
                        |e| Self::UnaryOp(op_variant, Box::new(e)),
                    ),
                    lower_precedence,
                ))(input)
            }
        }
    }

    // ------------------------------------------------------

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_expr_symbol() {
            assert_eq!(
                Expr::symbol("atomic_number "),
                Ok((" ", Expr::Symbol("atomic_number".to_string())))
            );
            assert_eq!(
                Expr::symbol("oxidation_state.notable  "),
                Ok(("  ", Expr::Symbol("oxidation_state.notable".to_string())))
            );
            assert_eq!(
                Expr::symbol("CamelCase  "),
                Ok(("  ", Expr::Symbol("CamelCase".to_string())))
            );
            assert_eq!(
                Expr::symbol("oxidation..state  "),
                Ok(("..state  ", Expr::Symbol("oxidation".to_string())))
            );
            assert!(matches!(Expr::symbol("_oxidation_state  "), Err(_)));
            assert!(matches!(Expr::symbol(".oxidation"), Err(_)));
        }

        #[test]
        fn test_expr_literal_bool() {
            assert_eq!(Expr::literal_bool("  true "), Ok((" ", Expr::LBool(true))));
        }

        #[test]
        fn test_expr_literal_int() {
            assert_eq!(Expr::literal_int("  -15 "), Ok((" ", Expr::LInt(-15))));
        }

        #[test]
        fn test_expr_binary_op() {
            use BinaryOperator::*;
            use Expr::*;
            use UnaryOperator::*;
            assert_eq!(
                Expr::parse("atomic_number == 5  "),
                Ok((
                    "  ",
                    BinaryOp(
                        BinaryOperator::Equal,
                        Box::new(Symbol("atomic_number".to_string())),
                        Box::new(LInt(5)),
                    )
                ))
            );
        }

        #[test]
        fn test_expr_precedence() {
            use BinaryOperator::*;
            use Expr::*;
            use UnaryOperator::*;
            assert_eq!(
                Expr::parse("true || Z > 13 && Z < 55"),
                Ok((
                    "",
                    BinaryOp(
                        BinaryOperator::Or,
                        Box::new(LBool(true)),
                        Box::new(BinaryOp(
                            BinaryOperator::And,
                            Box::new(BinaryOp(
                                BinaryOperator::GreaterThan,
                                Box::new(Symbol("Z".to_string())),
                                Box::new(LInt(13))
                            )),
                            Box::new(BinaryOp(
                                BinaryOperator::LessThan,
                                Box::new(Symbol("Z".to_string())),
                                Box::new(LInt(55))
                            ))
                        ))
                    ),
                ))
            );
            assert_eq!(
                Expr::parse("true ||  Z > 13 && Z < 55"),
                Expr::parse("true || (Z > 13 && Z < 55)"),
            );
            assert_eq!(Expr::parse("2 +  2 * 2"), Expr::parse("2 + (2 * 2)"),);
        }

        #[test]
        fn test_expr_unary_not() {
            use BinaryOperator::*;
            use Expr::*;
            use UnaryOperator::*;
            assert_eq!(
                Expr::parse("!a || b"),
                Ok((
                    "",
                    BinaryOp(
                        BinaryOperator::Or,
                        Box::new(UnaryOp(
                            UnaryOperator::Not,
                            Box::new(Symbol("a".to_string()))
                        )),
                        Box::new(Symbol("b".to_string()))
                    )
                ))
            );
        }

        #[test]
        fn test_expr_set() {
            use BinaryOperator::*;
            use Expr::*;
            assert_eq!(
                Expr::parse("{Z, 2, 3} in oxidation_states.notable"),
                Ok((
                    "",
                    BinaryOp(
                        InSet,
                        Box::new(LSet(vec![Symbol("Z".to_string()), LInt(2), LInt(3)])),
                        Box::new(Symbol("oxidation_states.notable".to_string())),
                    )
                ))
            );
        }
    }
}
