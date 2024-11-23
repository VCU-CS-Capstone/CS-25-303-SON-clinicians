use std::{any::type_name, str::FromStr};
pub mod tasks;
use ahash::{HashMap, HashMapExt};
use api::utils::{is_check_box_item, CheckboxValue, FieldNameAndIndex};
use chrono::NaiveDate;
use serde_json::Value;
use tracing::{error, warn};
pub mod converter;

mod enum_types;
pub use enum_types::*;
pub mod utils;

pub mod api;
// TODO: Use a faster hash map. It doesn't have to be DDOS resistant
pub type RedCapDataMap = HashMap<String, RedCapExportDataType>;
pub trait RedCapEnum {
    /// To Prevent Obscure Bugs. It will return None
    fn from_usize(value: usize) -> Option<Self>
    where
        Self: Sized;

    fn to_usize(&self) -> usize;
}
pub trait MultiSelectType: RedCapEnum {
    fn from_multi_select(multi_select: &MultiSelect) -> Option<Vec<Self>>
    where
        Self: Sized,
    {
        let mut result = Vec::new();

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

    fn create_multiselect(field_base: impl Into<String>, values: &[Self]) -> MultiSelect
    where
        Self: Sized,
    {
        let mut set_values = HashMap::new();
        for value in values {
            set_values.insert(value.to_usize() as i32, CheckboxValue::Checked);
        }
        MultiSelect {
            field_base: field_base.into(),
            set_values,
        }
    }
}
pub trait RedCapType {
    /// Reads a Red Cap taking an index to generate the key
    fn read_with_index<D: RedCapDataSet>(data: &D, _index: usize) -> Option<Self>
    where
        Self: Sized,
    {
        Self::read(data)
    }
    /// Reads a Red Cap
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
macro_rules! get {
    (
        $(
            $(#[$get_docs:meta])*
            $fn_name:ident -> $to:ident -> $type:ty
        ),*
    ) => {
        $(
            $(#[$get_docs])*
            fn $fn_name(&self, key: &str) -> Option<$type> {
                self.get(key).and_then(|value| value.$to())
            }
        )*
    };
}
pub trait RedCapDataSet {
    fn insert(&mut self, key: impl Into<String>, value: RedCapExportDataType);
    fn insert_multi_select<T: MultiSelectType>(&mut self, key: impl Into<String>, value: &[T]) {
        let key = key.into();
        let multi_select = T::create_multiselect(&key, value);
        self.insert(key, multi_select.into());
    }
    fn get(&self, key: &str) -> Option<&RedCapExportDataType>;
    get!(
        /// Get a number from the data set.
        ///
        /// If the value is not a number, it will return None
        get_number -> to_number -> usize,
        /// Get a float from the data set.
        ///
        /// If the value is not a float, it will return None
        get_float -> to_float -> f32,
        /// Get a date from the data set.
        ///
        /// If the value is not a date, it will return None
        get_date -> to_date -> NaiveDate,

        /// Get a bad boolean from the data set.
        ///
        /// If the value is not a bad boolean, it will return None
        get_bad_boolean -> to_bad_boolean -> bool,

        /// Get a string from the data set.
        /// If it is any other type it will call to_string. Except for MultiSelect and Enums
        get_string -> to_string -> String,
        /// Get a boolean from the data set.
        get_bool -> to_bool -> bool
    );
    fn get_enum<T>(&self, key: &str) -> Option<T>
    where
        T: RedCapEnum,
    {
        self.get(key).and_then(|value| value.to_enum())
    }
    fn get_enum_multi_select<T>(&self, key: &str) -> Option<Vec<T>>
    where
        T: MultiSelectType,
    {
        self.get(key).and_then(|value| value.process_multiselect())
    }

    fn iter(&self) -> impl Iterator<Item = (&String, &RedCapExportDataType)>;
}
impl RedCapDataSet for RedCapDataMap {
    fn insert(&mut self, key: impl Into<String>, value: RedCapExportDataType) {
        self.insert(key.into(), value);
    }

    fn get(&self, key: &str) -> Option<&RedCapExportDataType> {
        self.get(key)
    }
    fn iter(&self) -> impl Iterator<Item = (&String, &RedCapExportDataType)> {
        self.iter()
    }
}
#[derive(Debug, Clone)]
pub struct MultiSelect {
    pub field_base: String,
    pub set_values: HashMap<i32, CheckboxValue>,
}

pub fn find_and_extract_multi_selects(
    items: &mut HashMap<String, Value>,
) -> HashMap<String, MultiSelect> {
    let mut multi_selects = HashMap::new();
    let keys = items
        .keys()
        .filter(|key| is_check_box_item(key.as_str()))
        .cloned()
        .collect::<Vec<String>>();
    for key in keys {
        let value = items.remove(&key).unwrap();
        let FieldNameAndIndex { field_name, index } =
            FieldNameAndIndex::from_str(key.as_str()).unwrap();
        let multi_select = multi_selects
            .entry(field_name.clone())
            .or_insert_with(|| MultiSelect {
                field_base: field_name,
                set_values: HashMap::new(),
            });
        let index = index.unwrap();

        let checkbox_value = match CheckboxValue::try_from(value) {
            Ok(ok) => ok,
            Err(err) => {
                error!(?err, "Error parsing checkbox value");
                CheckboxValue::Unchecked
            }
        };

        multi_select.set_values.insert(index, checkbox_value);
    }
    multi_selects
}
#[derive(Debug, Clone)]
pub enum RedCapExportDataType {
    MultiSelect(MultiSelect),
    Text(String),
    Null,
    Float(f32),
    Number(isize),
    Date(NaiveDate),
}

impl<T> From<Option<T>> for RedCapExportDataType
where
    T: Into<RedCapExportDataType>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => value.into(),
            None => Self::Null,
        }
    }
}
macro_rules! from_for_export {
    (
        $(
            $type:ty => $variant:ident
        ),*
    ) => {
        $(
            impl From<$type> for RedCapExportDataType {
                fn from(value: $type) -> Self {
                    Self::$variant(value)
                }
            }
        )*
    };
}
from_for_export!(
    String => Text,
    NaiveDate => Date,
    f32 => Float,
    isize => Number,
    MultiSelect => MultiSelect
);

impl From<bool> for RedCapExportDataType {
    fn from(value: bool) -> Self {
        Self::Number(value as isize)
    }
}
macro_rules! from_num {
    (
        $(
            $type:ty
        ),*
    ) => {
        $(
            impl From<$type> for RedCapExportDataType {
                fn from(value: $type) -> Self {
                    Self::Number(value as isize)
                }
            }
        )*
    };
}

from_num!(i16, i32, u8, u16, u32, u64, usize);

impl<T> From<T> for RedCapExportDataType
where
    T: RedCapEnum,
{
    fn from(value: T) -> Self {
        Self::Number(value.to_usize() as isize)
    }
}
impl RedCapExportDataType {
    pub fn process_value(value: Value) -> Self {
        match value {
            Value::String(value) => Self::process_string(value),
            Value::Number(number) => {
                if number.is_i64() {
                    Self::Number(number.as_i64().unwrap() as isize)
                } else if number.is_f64() {
                    Self::Float(number.as_f64().unwrap() as f32)
                } else {
                    panic!("Unknown Number Type");
                }
            }
            Value::Bool(value) => Self::Number(value as isize),
            _ => Self::Null,
        }
    }
    pub fn process_string(value: String) -> Self {
        if value.is_empty() {
            Self::Null
        } else if let Ok(number) = value.parse::<isize>() {
            Self::Number(number)
        } else if let Ok(float) = value.parse::<f32>() {
            Self::Float(float)
        } else if let Ok(date) = NaiveDate::parse_from_str(&value, "%Y-%m-%d") {
            Self::Date(date)
        } else {
            Self::Text(value)
        }
    }
    pub fn to_string(&self) -> Option<String> {
        match self {
            Self::Text(value) => Some(value.clone()),
            Self::Number(value) => Some(value.to_string()),
            Self::Date(value) => Some(value.format("%Y-%m-%d").to_string()),
            _ => None,
        }
    }
    /// Bad Booleans are 2 = true, 1 = false
    /// Wouldn't shock me if they sometimes use 0 = false
    ///
    /// So I only check for value = 2
    pub fn to_bad_boolean(&self) -> Option<bool> {
        match self {
            Self::Text(value) => Some(value == "2"),
            Self::Number(value) => Some(*value == 2),
            _ => None,
        }
    }
    pub fn from_bad_boolean(value: bool) -> Self {
        if value {
            Self::Number(2)
        } else {
            Self::Number(1)
        }
    }
    pub fn to_float(&self) -> Option<f32> {
        match self {
            Self::Float(value) => Some(*value),
            Self::Number(value) => Some(*value as f32),
            _ => None,
        }
    }
    pub fn to_number(&self) -> Option<usize> {
        match self {
            Self::Number(value) => Some(*value as usize),
            Self::Float(value) => {
                error!(?value, "Float to Number Conversion");
                Some(*value as usize)
            }
            _ => None,
        }
    }
    pub fn to_date(&self) -> Option<NaiveDate> {
        match self {
            Self::Date(value) => Some(*value),
            _ => None,
        }
    }
    pub fn to_enum<T>(&self) -> Option<T>
    where
        T: RedCapEnum,
    {
        match self {
            Self::Number(value) => T::from_usize(*value as usize),
            _ => None,
        }
    }
    pub fn to_bool(&self) -> Option<bool> {
        match self {
            Self::Number(value) => Some(*value == 1),
            _ => None,
        }
    }
    pub fn as_multiselect(&self) -> Option<&MultiSelect> {
        match self {
            Self::MultiSelect(value) => Some(value),
            _ => None,
        }
    }
    pub fn process_multiselect<T: MultiSelectType>(&self) -> Option<Vec<T>> {
        match self {
            Self::MultiSelect(value) => T::from_multi_select(value),
            _ => None,
        }
    }
}

pub fn process_flat_json(
    mut input: HashMap<String, Value>,
) -> HashMap<String, RedCapExportDataType> {
    let multi_selects = find_and_extract_multi_selects(&mut input);

    let mut output = HashMap::with_capacity(input.len() + multi_selects.len());
    for (key, value) in multi_selects {
        output.insert(key, RedCapExportDataType::MultiSelect(value));
    }
    for (key, value) in input {
        let value = RedCapExportDataType::process_value(value);
        output.insert(key, value);
    }
    output
}
pub fn flatten_data_to_red_cap_format(
    input: HashMap<String, RedCapExportDataType>,
) -> HashMap<String, String> {
    let mut output = HashMap::new();
    for (key, value) in input {
        match value {
            RedCapExportDataType::MultiSelect(multi_select) => {
                for (index, value) in multi_select.set_values {
                    let value: usize = value.into();
                    let key = format!("{}___{}", multi_select.field_base, index);
                    output.insert(key, value.to_string());
                }
            }
            RedCapExportDataType::Text(text) => {
                output.insert(key, text);
            }
            RedCapExportDataType::Null => {
                output.insert(key, String::new());
            }
            RedCapExportDataType::Number(number) => {
                output.insert(key, number.to_string());
            }
            RedCapExportDataType::Float(float) => {
                output.insert(key, float.to_string());
            }
            RedCapExportDataType::Date(naive_date) => {
                output.insert(key, naive_date.format("%Y-%m-%d").to_string());
            }
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use anyhow::Context;

    use super::api::RedcapClient;

    pub async fn load_red_cap_api_and_db() -> anyhow::Result<(RedcapClient, sqlx::PgPool)> {
        crate::test_utils::init_logger();

        let env = crate::env_utils::read_env_file_in_core("test.env")
            .context("Unable to load test.env")?;

        let database = crate::database::tests::connect_to_db_with(&env).await?;
        let client =
            RedcapClient::new(env.get("RED_CAP_TOKEN").context("No RED_CAP_TOKEN")?).await?;

        Ok((client, database))
    }
}
