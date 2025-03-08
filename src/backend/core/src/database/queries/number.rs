use std::fmt::Display;
use std::str::FromStr;

use chumsky::label::LabelError;
use chumsky::prelude::*;
use parse::ParseNumber;
use pg_extended_sqlx_queries::prelude::*;
use utoipa::ToSchema;
pub mod parse;

pub type ErrType = chumsky::extra::Err<chumsky::error::Cheap>;
pub enum NumberQueryType {
    GreaterThan,
    LessThan,
    EqualTo,
    GreaterThanOrEqualTo,
    LessThanOrEqualTo,
}
/// Accepts a string that represents a range in numbers
///
/// # Examples
/// ```
/// use cs25_303_core::database::queries::NumberQuery;
/// let query = "10..20";
/// let result: NumberQuery<i32> = query.parse().unwrap();
/// assert_eq!(result, NumberQuery::Range { start: 10, end: 20 });
///
/// let query = ">=10";
///
/// let result: NumberQuery<i32> = query.parse().unwrap();
///
/// assert_eq!(result, NumberQuery::GreaterThanOrEqualTo(10));
///
/// let query = "10";
/// let result: NumberQuery<i32> = query.parse().unwrap();
/// assert_eq!(result, NumberQuery::EqualTo(10));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[schema(as = String)]
pub enum NumberQuery<I = i32> {
    GreaterThan(I),
    LessThan(I),
    EqualTo(I),
    GreaterThanOrEqualTo(I),
    LessThanOrEqualTo(I),
    Range { start: I, end: I },
}
impl<I: Default> Default for NumberQuery<I> {
    fn default() -> Self {
        NumberQuery::EqualTo(I::default())
    }
}
impl<'args, I> NumberQuery<I>
where
    I: ExprType<'args> + 'args,
{
    pub fn filter(
        self,
        column: impl ColumnType + 'static,
    ) -> FilterConditionBuilder<'args, DynExpr<'args>, DynExpr<'args>> {
        match self {
            NumberQuery::GreaterThan(n) => column.dyn_column().greater_than(n).dyn_expression(),
            NumberQuery::LessThan(n) => column.dyn_column().less_than(n).dyn_expression(),
            NumberQuery::EqualTo(n) => column.dyn_column().equals(n).dyn_expression(),
            NumberQuery::GreaterThanOrEqualTo(n) => column
                .dyn_column()
                .greater_than_or_equals(n)
                .dyn_expression(),
            NumberQuery::LessThanOrEqualTo(n) => {
                column.dyn_column().less_than_or_equals(n).dyn_expression()
            }
            NumberQuery::Range { start, end } => {
                column.dyn_column().between(start, end).dyn_expression()
            }
        }
    }

    pub fn complex_value_filter<E: ExprType<'args> + 'args>(
        self,
        value: E,
    ) -> FilterConditionBuilder<'args, DynExpr<'args>, DynExpr<'args>> {
        match self {
            NumberQuery::GreaterThan(n) => value.greater_than(n).dyn_expression(),
            NumberQuery::LessThan(n) => value.less_than(n).dyn_expression(),
            NumberQuery::EqualTo(n) => value.equals(n).dyn_expression(),
            NumberQuery::GreaterThanOrEqualTo(n) => {
                value.greater_than_or_equals(n).dyn_expression()
            }
            NumberQuery::LessThanOrEqualTo(n) => value.less_than_or_equals(n).dyn_expression(),
            NumberQuery::Range { start, end } => value.between(start, end).dyn_expression(),
        }
    }
}
impl<I> Display for NumberQuery<I>
where
    I: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NumberQuery::GreaterThan(n) => write!(f, ">{}", n),
            NumberQuery::LessThan(n) => write!(f, "<{}", n),
            NumberQuery::EqualTo(n) => write!(f, "={}", n),
            NumberQuery::GreaterThanOrEqualTo(n) => write!(f, ">={}", n),
            NumberQuery::LessThanOrEqualTo(n) => write!(f, "<={}", n),
            NumberQuery::Range { start, end } => write!(f, "{}..{}", start, end),
        }
    }
}
impl<I> FromStr for NumberQuery<I>
where
    I: ParseNumber,
{
    type Err = Vec<Cheap>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        number_query().parse(s).into_result()
    }
}

fn number_query<'a, I>() -> impl Parser<'a, &'a str, NumberQuery<I>, ErrType>
where
    I: ParseNumber,
{
    choice((
        parse_symbol_type().then(I::parser()).map(|(t, n)| match t {
            NumberQueryType::GreaterThan => NumberQuery::GreaterThan(n),
            NumberQueryType::LessThan => NumberQuery::LessThan(n),
            NumberQueryType::EqualTo => NumberQuery::EqualTo(n),
            NumberQueryType::GreaterThanOrEqualTo => NumberQuery::GreaterThanOrEqualTo(n),
            NumberQueryType::LessThanOrEqualTo => NumberQuery::LessThanOrEqualTo(n),
        }),
        range(),
        I::parser().map(NumberQuery::EqualTo),
    ))
}

fn range<'a, I>() -> impl Parser<'a, &'a str, NumberQuery<I>, ErrType>
where
    I: ParseNumber,
{
    I::parser()
        .then(just(".."))
        .then(I::parser())
        .map(|((start, _), end)| NumberQuery::Range { start, end })
        .labelled("range")
}
fn parse_symbol_type<'a>() -> impl Parser<'a, &'a str, NumberQueryType, ErrType> {
    symbol()
        .then(symbol().or_not())
        .try_map(|(s1, s2), span| match (s1, s2) {
            (Symbol::GreatherThan, Some(Symbol::Equal))
            | (Symbol::Equal, Some(Symbol::GreatherThan)) => {
                Ok(NumberQueryType::GreaterThanOrEqualTo)
            }
            (Symbol::LessThan, Some(Symbol::Equal)) | (Symbol::Equal, Some(Symbol::LessThan)) => {
                Ok(NumberQueryType::LessThanOrEqualTo)
            }
            (Symbol::GreatherThan, None) => Ok(NumberQueryType::GreaterThan),
            (Symbol::LessThan, None) => Ok(NumberQueryType::LessThan),
            (Symbol::Equal, None) => Ok(NumberQueryType::EqualTo),
            _ => {
                // TODO: Put something in the expected
                Err(<Cheap as LabelError<'a, &'a str, _>>::expected_found(
                    vec![""],
                    None,
                    span,
                ))
            }
        })
        .labelled("parse_symbol_type")
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Symbol {
    /// >
    GreatherThan,
    /// <
    LessThan,
    /// =
    Equal,
}
fn symbol<'a>() -> impl Parser<'a, &'a str, Symbol, ErrType> {
    choice((
        just('>').map(|_| Symbol::GreatherThan),
        just('=').map(|_| Symbol::Equal),
        just('<').map(|_| Symbol::LessThan),
    ))
}
mod _serde {
    use std::fmt::Display;

    use serde::Serialize;

    use super::{NumberQuery, parse::AnySerdeNumber, parse::ParseNumber};

    impl<I> Serialize for NumberQuery<I>
    where
        I: Display,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let to_str = self.to_string();
            serializer.serialize_str(&to_str)
        }
    }
    struct NumberQueryVisitor<I> {
        _phantom: std::marker::PhantomData<I>,
    }
    macro_rules! visit_num {
        (
            $(
                $fn_name:ident($t:ty)
            ),*
        ) => {

            $(
                fn $fn_name<E>(self, v: $t) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    let any_number = AnySerdeNumber::from(v);
                    I::from_any_number(any_number)
                    .map_err(|err| E::custom(format!("{}", err)))
                    .map(|num| NumberQuery::EqualTo(num))
                }
            )*
        };
    }
    impl<'de, I> serde::de::Visitor<'de> for NumberQueryVisitor<I>
    where
        I: ParseNumber,
    {
        type Value = NumberQuery<I>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "a number query")
        }
        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            match value.parse() {
                Ok(ok) => Ok(ok),
                Err(err) => {
                    if err.len() == 1 {
                        Err(serde::de::Error::custom(format!("{:?}", err[0])))
                    } else {
                        Err(serde::de::Error::custom(format!("{:?}", err)))
                    }
                }
            }
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            match value.parse() {
                Ok(ok) => Ok(ok),
                Err(err) => {
                    if err.len() == 1 {
                        Err(serde::de::Error::custom(format!("{:?}", err[0])))
                    } else {
                        Err(serde::de::Error::custom(format!("{:?}", err)))
                    }
                }
            }
        }
        visit_num!(
            visit_i32(i32),
            visit_i64(i64),
            visit_u32(u32),
            visit_u64(u64),
            visit_f32(f32),
            visit_f64(f64)
        );
    }
    impl<'de, I> serde::Deserialize<'de> for NumberQuery<I>
    where
        I: ParseNumber,
    {
        fn deserialize<D>(deserializer: D) -> Result<NumberQuery<I>, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_str(NumberQueryVisitor {
                _phantom: std::marker::PhantomData,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::database::queries::NumberQuery;

    #[test]
    fn greater_than_or_equal() {
        let query = ">=10";
        let result: super::NumberQuery<i32> = query.parse().unwrap();
        assert_eq!(result, super::NumberQuery::GreaterThanOrEqualTo(10));
        assert_eq!(result.to_string(), ">=10");

        let query = "=>10";
        let result: NumberQuery = query.parse().unwrap();
        assert_eq!(result, super::NumberQuery::GreaterThanOrEqualTo(10));
        assert_eq!(result.to_string(), ">=10");
    }
    #[test]
    fn equal_to() {
        let query = "10";
        let result: super::NumberQuery<i32> = query.parse().unwrap();
        assert_eq!(result, super::NumberQuery::EqualTo(10));
        assert_eq!(result.to_string(), "=10");
        let query = "=10";
        let result: super::NumberQuery<i32> = query.parse().unwrap();
        assert_eq!(result, super::NumberQuery::EqualTo(10));
        assert_eq!(result.to_string(), "=10");
    }
    #[test]

    fn less_than() {
        let query = "<10";
        let result: super::NumberQuery<i32> = query.parse().unwrap();
        assert_eq!(result, super::NumberQuery::LessThan(10));
        assert_eq!(result.to_string(), "<10");
    }

    #[test]
    fn less_than_or_equal() {
        let query = "<=10";
        let result: super::NumberQuery<i32> = query.parse().unwrap();
        assert_eq!(result, super::NumberQuery::LessThanOrEqualTo(10));
        assert_eq!(result.to_string(), "<=10");

        let query = "=<10";
        let result: super::NumberQuery<i32> = query.parse().unwrap();
        assert_eq!(result, super::NumberQuery::LessThanOrEqualTo(10));
        assert_eq!(result.to_string(), "<=10");
    }
    #[test]
    fn greater_than() {
        let query = ">10";
        let result: super::NumberQuery<i32> = query.parse().unwrap();
        assert_eq!(result, super::NumberQuery::GreaterThan(10));
        assert_eq!(result.to_string(), ">10");
    }
    #[test]
    fn test_range() {
        let query = "10..20";
        let result: super::NumberQuery<i32> = query.parse().unwrap();
        assert_eq!(result, super::NumberQuery::Range { start: 10, end: 20 });
        assert_eq!(result.to_string(), "10..20");
    }
    #[test]
    fn test_range_f32() {
        let query = "10..20";
        let result: super::NumberQuery<f32> = query.parse().unwrap();
        assert_eq!(
            result,
            super::NumberQuery::Range {
                start: 10.0,
                end: 20.0
            }
        );
        assert_eq!(result.to_string(), "10..20");

        let query = "10.5..20.5";
        let result: super::NumberQuery<f32> = query.parse().unwrap();
        assert_eq!(
            result,
            super::NumberQuery::Range {
                start: 10.5,
                end: 20.5
            }
        );
        assert_eq!(result.to_string(), "10.5..20.5");
    }
}
