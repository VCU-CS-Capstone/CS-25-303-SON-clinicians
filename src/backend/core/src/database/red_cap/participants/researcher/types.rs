use std::{marker::PhantomData, str::FromStr};

use serde::Serialize;
use utoipa::ToSchema;

use crate::database::queries::NumberQuery;

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema, Default)]
pub struct ResearcherQueryGlucose {
    pub glucose: NumberQuery<f32>,
    /// Undefined will tell the query you do not want to filter by this
    pub fasted_atleast_2_hours: Option<bool>,
}
impl<'de> serde::Deserialize<'de> for ResearcherQueryGlucose {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[doc(hidden)]
        enum GlucoseField {
            Glucose,
            FastedAtleastTwoHours,
            Ignored,
        }
        #[doc(hidden)]
        struct FieldVisitor;

        impl<'de> serde::de::Visitor<'de> for FieldVisitor {
            type Value = GlucoseField;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("field identifier")
            }
            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    0u64 => Ok(GlucoseField::Glucose),
                    1u64 => Ok(GlucoseField::FastedAtleastTwoHours),
                    _ => Ok(GlucoseField::Ignored),
                }
            }
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "glucose" | "result" => Ok(GlucoseField::Glucose),
                    "fasted_atleast_2_hours" => Ok(GlucoseField::FastedAtleastTwoHours),
                    _ => Ok(GlucoseField::Ignored),
                }
            }
            fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    b"glucose" | b"result" => Ok(GlucoseField::Glucose),
                    b"fasted_atleast_2_hours" => Ok(GlucoseField::FastedAtleastTwoHours),
                    _ => Ok(GlucoseField::Ignored),
                }
            }
        }
        #[automatically_derived]
        impl<'de> serde::Deserialize<'de> for GlucoseField {
            #[inline]
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                serde::Deserializer::deserialize_identifier(deserializer, FieldVisitor)
            }
        }
        #[doc(hidden)]
        struct GlucoseVisitor<'de> {
            marker: PhantomData<ResearcherQueryGlucose>,
            lifetime: PhantomData<&'de ()>,
        }
        impl<'de> serde::de::Visitor<'de> for GlucoseVisitor<'de> {
            type Value = ResearcherQueryGlucose;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct ResearcherQueryGlucose or a string of NumberQuery")
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let glucose_field = NumberQuery::from_str(v).map_err(E::custom)?;
                Ok(ResearcherQueryGlucose {
                    glucose: glucose_field,
                    fasted_atleast_2_hours: None,
                })
            }
            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let glucose_field = NumberQuery::from_str(&v).map_err(E::custom)?;
                Ok(ResearcherQueryGlucose {
                    glucose: glucose_field,
                    fasted_atleast_2_hours: None,
                })
            }
            #[inline]
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let glucose =
                    match serde::de::SeqAccess::next_element::<NumberQuery<f32>>(&mut seq)? {
                        Some(__value) => __value,
                        None => {
                            return Err(serde::de::Error::invalid_length(
                                0usize,
                                &"struct ResearcherQueryGlucose with 2 elements",
                            ));
                        }
                    };
                let fasted_atleast_2_hours =
                    match serde::de::SeqAccess::next_element::<Option<bool>>(&mut seq)? {
                        Some(__value) => __value,
                        None => {
                            return Err(serde::de::Error::invalid_length(
                                1usize,
                                &"struct ResearcherQueryGlucose with 2 elements",
                            ));
                        }
                    };
                Ok(ResearcherQueryGlucose {
                    glucose,
                    fasted_atleast_2_hours,
                })
            }
            #[inline]
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut glucose: Option<NumberQuery<f32>> = None;
                let mut fasted_atleast_2_hours: Option<Option<bool>> = None;
                while let Some(__key) = serde::de::MapAccess::next_key::<GlucoseField>(&mut map)? {
                    match __key {
                        GlucoseField::Glucose => {
                            if Option::is_some(&glucose) {
                                return Err(<A::Error as serde::de::Error>::duplicate_field(
                                    "glucose",
                                ));
                            }
                            glucose = Some(serde::de::MapAccess::next_value::<NumberQuery<f32>>(
                                &mut map,
                            )?);
                        }
                        GlucoseField::FastedAtleastTwoHours => {
                            if Option::is_some(&fasted_atleast_2_hours) {
                                return Err(<A::Error as serde::de::Error>::duplicate_field(
                                    "fasted_atleast_2_hours",
                                ));
                            }
                            fasted_atleast_2_hours =
                                Some(serde::de::MapAccess::next_value::<Option<bool>>(&mut map)?);
                        }
                        _ => {
                            let _ = serde::de::MapAccess::next_value::<serde::de::IgnoredAny>(
                                &mut map,
                            )?;
                        }
                    }
                }
                let glucose = match glucose {
                    Some(glucose) => glucose,
                    None => serde::__private::de::missing_field("glucose")?,
                };
                let fasted_atleast_2_hours = fasted_atleast_2_hours.unwrap_or_default();
                Ok(ResearcherQueryGlucose {
                    glucose,
                    fasted_atleast_2_hours,
                })
            }
        }
        serde::Deserializer::deserialize_any(
            deserializer,
            GlucoseVisitor {
                marker: PhantomData::<ResearcherQueryGlucose>,
                lifetime: PhantomData,
            },
        )
    }
}
