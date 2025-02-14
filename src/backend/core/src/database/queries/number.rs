use std::fmt::Display;
use std::num::ParseIntError;
use std::str::FromStr;

use chumsky::label::LabelError;
use chumsky::prelude::*;
use chumsky::text::int;
use pg_extended_sqlx_queries::{ColumnType, DynEncodeType, ExprType, FilterConditionBuilder};
use tracing::warn;
use utoipa::ToSchema;

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
impl<'args, I> NumberQuery<I>
where
    I: DynEncodeType<'args>,
{
    pub fn filter(self, column: impl ColumnType + 'static) -> FilterConditionBuilder<'args> {
        match self {
            NumberQuery::GreaterThan(n) => column.dyn_column().greater_than(n.value()),
            NumberQuery::LessThan(n) => column.dyn_column().less_than(n.value()),
            NumberQuery::EqualTo(n) => column.dyn_column().equals(n.value()),
            NumberQuery::GreaterThanOrEqualTo(n) => {
                column.dyn_column().greater_than_or_equals(n.value())
            }
            NumberQuery::LessThanOrEqualTo(n) => column.dyn_column().less_than_or_equals(n.value()),
            NumberQuery::Range { start, end } => {
                column.dyn_column().between(start.value(), end.value())
            }
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
    I: FromStrRadix,
{
    type Err = Vec<Cheap>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        number_query().parse(s).into_result()
    }
}

fn number_query<'a, I>() -> impl Parser<'a, &'a str, NumberQuery<I>, ErrType>
where
    I: FromStrRadix,
{
    choice((
        parse_symbol_type().then(number()).map(|(t, n)| match t {
            NumberQueryType::GreaterThan => NumberQuery::GreaterThan(n),
            NumberQueryType::LessThan => NumberQuery::LessThan(n),
            NumberQueryType::EqualTo => NumberQuery::EqualTo(n),
            NumberQueryType::GreaterThanOrEqualTo => NumberQuery::GreaterThanOrEqualTo(n),
            NumberQueryType::LessThanOrEqualTo => NumberQuery::LessThanOrEqualTo(n),
        }),
        range(),
        number().map(NumberQuery::EqualTo),
    ))
}

fn range<'a, I>() -> impl Parser<'a, &'a str, NumberQuery<I>, ErrType>
where
    I: FromStrRadix,
{
    number()
        .then(just(".."))
        .then(number())
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
fn number<'a, I>() -> impl Parser<'a, &'a str, I, ErrType>
where
    I: FromStrRadix,
{
    int(10).try_map(|n: &str, span| {
        I::from_radix(n, 10).map_err(|err| {
            warn!(
                ?err,
                "Failed to parse number however, chumsky should have caught this"
            );
            <Cheap as LabelError<'a, &'a str, _>>::expected_found(vec![""], None, span)
        })
    })
}
pub trait FromStrRadix: Sized {
    fn from_radix(str: &str, radix: u32) -> Result<Self, ParseIntError>;
}

macro_rules! impl_from_radix {
    (
        $(
            $t:ty
        ),*
    ) => {
        $(
            impl FromStrRadix for $t {
                fn from_radix(str: &str, radix: u32) -> Result<Self, ParseIntError> {
                    <$t>::from_str_radix(str, radix)
                }
            }
        )*
    };
}

impl_from_radix!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
mod _serde {
    use std::fmt::Display;

    use serde::Serialize;

    use super::{FromStrRadix, NumberQuery};

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
                    match I::try_from(v) {
                        Ok(ok) => Ok(NumberQuery::EqualTo(ok)),
                        Err(err) => Err(serde::de::Error::custom(err)),
                    }
                }
            )*
        };
    }
    impl<'de, I> serde::de::Visitor<'de> for NumberQueryVisitor<I>
    where
        I: FromStrRadix,
        I: TryFrom<i64>,
        <I as TryFrom<i64>>::Error: std::fmt::Display,
        I: TryFrom<i32>,
        <I as TryFrom<i32>>::Error: std::fmt::Display,

        I: TryFrom<u64>,
        <I as TryFrom<u64>>::Error: std::fmt::Display,
        I: TryFrom<u32>,
        <I as TryFrom<u32>>::Error: std::fmt::Display,
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
            visit_u64(u64)
        );
    }
    impl<'de, I> serde::Deserialize<'de> for NumberQuery<I>
    where
        I: FromStrRadix,
        I: TryFrom<i64>,
        <I as TryFrom<i64>>::Error: std::fmt::Display,
        I: TryFrom<i32>,
        <I as TryFrom<i32>>::Error: std::fmt::Display,

        I: TryFrom<u64>,
        <I as TryFrom<u64>>::Error: std::fmt::Display,
        I: TryFrom<u32>,
        <I as TryFrom<u32>>::Error: std::fmt::Display,
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
}
