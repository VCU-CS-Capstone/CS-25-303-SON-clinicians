use chumsky::{
    label::LabelError,
    prelude::*,
    text::{digits, int},
};
use std::num::ParseIntError;
use tracing::warn;

use thiserror::Error;

use super::ErrType;
#[derive(Debug, Error)]
pub enum NumberParseError {
    #[error(transparent)]
    NumParse(#[from] ParseIntError),
    #[error(transparent)]
    FloatParse(#[from] std::num::ParseFloatError),
}
#[derive(Debug, Error)]
pub enum InvalidNumberType {
    #[error("Expected a whole number but found a float")]
    WholeNumberExpected,
    #[error("Number Exceeds the maximum value for this type")]
    NumberExceedsMaxValue,
    #[error("Number should be positive but found a negative number")]
    NegativeNumber,
}
pub trait ParseNumber: Sized {
    fn from_str(str: &str) -> Result<Self, NumberParseError>;

    fn parser<'a>() -> impl Parser<'a, &'a str, Self, ErrType>;

    fn from_any_number(any: AnySerdeNumber) -> Result<Self, InvalidNumberType>;
}

macro_rules! impl_from_radix {
    (
        $(
            $t:ty
        ),*
    ) => {
        $(
            impl ParseNumber for $t {
                fn from_str(str: &str) -> Result<Self, NumberParseError> {
                    <$t>::from_str_radix(str, 10).map_err(NumberParseError::from)
                }
                fn parser<'a>() -> impl Parser<'a, &'a str, Self, ErrType>{
                    number::<Self>()
                }
                fn from_any_number(any: AnySerdeNumber) -> Result<Self, InvalidNumberType>{
                    any.try_into()
                }
            }
        )*
    };
}

impl_from_radix!(i8, i16, i32, i64, u8, u16, u32, u64);

impl ParseNumber for f32 {
    fn from_str(str: &str) -> Result<Self, NumberParseError> {
        str.parse().map_err(NumberParseError::from)
    }
    fn parser<'a>() -> impl Parser<'a, &'a str, Self, ErrType> {
        float()
    }
    fn from_any_number(any: AnySerdeNumber) -> Result<Self, InvalidNumberType> {
        any.try_into()
    }
}

impl ParseNumber for f64 {
    fn from_str(str: &str) -> Result<Self, NumberParseError> {
        str.parse().map_err(NumberParseError::from)
    }
    fn parser<'a>() -> impl Parser<'a, &'a str, Self, ErrType> {
        float()
    }
    fn from_any_number(any: AnySerdeNumber) -> Result<Self, InvalidNumberType> {
        any.try_into()
    }
}

fn number<'a, I>() -> impl Parser<'a, &'a str, I, ErrType>
where
    I: ParseNumber,
{
    int(10).try_map(|n: &str, span| {
        I::from_str(n).map_err(|err| {
            warn!(
                ?err,
                "Failed to parse number however, chumsky should have caught this"
            );
            <Cheap as LabelError<'a, &'a str, _>>::expected_found(vec![""], None, span)
        })
    })
}
///
///
/// ## Note
///  This float parser will accept regular whole numbers as floats
fn float<'a, I>() -> impl Parser<'a, &'a str, I, ErrType>
where
    I: ParseNumber,
{
    // TODO: Accept Scientific notation
    digits(10)
        .to_slice()
        .then(just('.').then(digits(10).to_slice()).or_not())
        .try_map(|(digits, fraction): (&str, Option<(char, &str)>), span| {
            let mut raw_number = digits.to_string();
            if let Some((_dot, fraction)) = fraction {
                raw_number.push('.');
                raw_number.push_str(fraction);
            }
            I::from_str(&raw_number).map_err(|err| {
                warn!(
                    ?err,
                    "Failed to parse number however, chumsky should have caught this"
                );
                <Cheap as LabelError<'a, &'a str, _>>::expected_found(vec![""], None, span)
            })
        })
}

#[cfg(test)]
mod tests {

    use chumsky::Parser;
    #[test]
    fn f32_test() {
        let float = super::float::<f32>();
        let value = float.parse("1.0").unwrap();
        assert_eq!(value, 1.0);

        let value = float.parse("1").unwrap();
        assert_eq!(value, 1.0);

        let value = float.parse("1.1").unwrap();
        assert_eq!(value, 1.1);
    }
    #[test]
    fn f64_test() {
        let float = super::float::<f64>();
        let value = float.parse("1.0").unwrap();
        assert_eq!(value, 1.0);

        let value = float.parse("1").unwrap();
        assert_eq!(value, 1.0);

        let value = float.parse("1.1").unwrap();
        assert_eq!(value, 1.1);
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum AnySerdeNumber {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
}

macro_rules! any_number {
    (
        $(
            $t:ty => $variant:ident
        ),*
    ) => {
        $(
            impl From<$t> for AnySerdeNumber {
                fn from(value: $t) -> Self {
                    AnySerdeNumber::$variant(value)
                }
            }
            // Technically this should do further checks to ensure that the number is within the range of the type
            impl TryFrom<AnySerdeNumber> for $t {
                type Error = InvalidNumberType;

                fn try_from(value: AnySerdeNumber) -> Result<Self, Self::Error> {
                    match value {
                        AnySerdeNumber::I8(value) => Ok(value as $t),
                        AnySerdeNumber::I16(value) => Ok(value as $t),
                        AnySerdeNumber::I32(value) => Ok(value as $t),
                        AnySerdeNumber::I64(value) => Ok(value as $t),
                        AnySerdeNumber::U8(value) => Ok(value as $t),
                        AnySerdeNumber::U16(value) => Ok(value as $t),
                        AnySerdeNumber::U32(value) => Ok(value as $t),
                        AnySerdeNumber::U64(value) => Ok(value as $t),
                        AnySerdeNumber::F32(value) => Ok(value as $t),
                        AnySerdeNumber::F64(value) => Ok(value as $t),
                    }
                }
            }
        )*
    };
}

any_number!(
    i8 => I8,
    i16 => I16,
    i32 => I32,
    i64 => I64,
    u8 => U8,
    u16 => U16,
    u32 => U32,
    u64 => U64,
    f32 => F32,
    f64 => F64
);
