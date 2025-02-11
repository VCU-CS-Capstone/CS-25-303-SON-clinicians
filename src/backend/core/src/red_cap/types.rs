//! This module contains the traits for the RedCap types.
//!
//! ## Why do the data return Option instead of Result?
//! Due to the nature of the RedCap API. I don't want inconsistent data from the api to cause errors.
//!
//! We might in the future start to return errors. But for now, Option will be used.
use std::any::type_name;

use ahash::{HashMap, HashMapExt};
use strum::IntoEnumIterator;
use tracing::warn;

use super::{api::utils::CheckboxValue, MultiSelect, RedCapDataSet};

pub trait RedCapEnum {
    /// To Prevent Obscure Bugs. It will return None
    fn from_usize(value: usize) -> Option<Self>
    where
        Self: Sized;
    /// Returns the Red Cap value as a usize
    fn to_usize(&self) -> usize;
    /// This exists for MultiSelect optimization.
    /// However, because the Enum implementation is created by a macro it was easier to implement it here.
    fn num_variants() -> usize
    where
        Self: Sized;
}
/// A MultiSelectType is an extension of an Enum.
///
/// Where an Enum is like a single value such as Radio or Dropdown
///
/// A multi select is like a checkbox where multiple values can be selected
///
/// So impl this trait on a type that is a RedCapEnum the default implementation will handle the rest.
pub trait MultiSelectType: RedCapEnum + PartialEq + IntoEnumIterator {
    /// Converts a MultiSelect into a Vec<Self>
    fn from_multi_select(multi_select: &MultiSelect) -> Option<Vec<Self>>
    where
        Self: Sized,
    {
        //
        let span = tracing::trace_span!(
            "from_multi_select",
            ?multi_select,
            enum_type = Self::multi_select_key()
        );
        let _enter = span.enter();
        let mut result = Vec::with_capacity(multi_select.set_values.len());

        for (id, value) in multi_select.set_values.iter() {
            if value == &CheckboxValue::Checked {
                if let Some(value) = Self::from_usize(*id as usize) {
                    result.push(value);
                } else {
                    warn!(?id, "Unknown {}", type_name::<Self>());
                }
            }
        }

        Some(result)
    }
    /// Converts a Vec<Self> into a MultiSelect
    ///
    /// Takes in a field_base to generate the keys
    ///
    /// Any value that is not in the Vec will be set to Unchecked
    fn create_multiselect(field_base: impl Into<String>, values: &[Self]) -> MultiSelect
    where
        Self: Sized,
    {
        let mut set_values = HashMap::with_capacity(Self::num_variants());
        for value in Self::iter() {
            let checked: CheckboxValue = if values.contains(&value) {
                CheckboxValue::Checked
            } else {
                CheckboxValue::Unchecked
            };
            set_values.insert(value.to_usize() as i32, checked);
        }
        MultiSelect {
            field_base: field_base.into(),
            set_values,
        }
    }

    fn multi_select_key() -> &'static str
    where
        Self: Sized;
}

/// A RedCapType is a group of fields that are connected.
///
/// Such as `gender` and `gender_self`
pub trait RedCapType {
    /// Reads a Red Cap taking an index to generate the key
    fn read_with_index<D: RedCapDataSet>(data: &D, _index: usize) -> Option<Self>
    where
        Self: Sized,
    {
        Self::read(data)
    }
    /// Reads a Red Cap
    ///
    /// If the implementation expects an index it should return None
    fn read<D: RedCapDataSet>(data: &D) -> Option<Self>
    where
        Self: Sized;
    /// Writes a Red Cap taking an index to generate the key
    fn write_with_index<D: RedCapDataSet>(&self, data: &mut D, _index: usize)
    where
        Self: Sized,
    {
        self.write(data)
    }
    /// Writes a Red Cap
    fn write<D: RedCapDataSet>(&self, data: &mut D);
}
