use std::{any::type_name, str::FromStr};
pub mod tasks;
use ahash::{HashMap, HashMapExt};
use api::utils::{is_check_box_item, CheckboxValue, FieldNameAndIndex};
use chrono::NaiveDate;
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
pub trait RedCapDataSet {
    fn insert(&mut self, key: impl Into<String>, value: RedCapExportDataType);
    fn insert_multi_select<T: MultiSelectType>(&mut self, key: impl Into<String>, value: &[T]) {
        let key = key.into();
        let multi_select = T::create_multiselect(&key, value);
        self.insert(key, multi_select.into());
    }
    fn get(&self, key: &str) -> Option<&RedCapExportDataType>;

    fn get_number(&self, key: &str) -> Option<usize> {
        self.get(key).and_then(|value| value.to_number())
    }
    fn get_float(&self, key: &str) -> Option<f32> {
        self.get(key).and_then(|value| value.to_float())
    }

    fn get_date(&self, key: &str) -> Option<NaiveDate> {
        self.get(key).and_then(|value| value.to_date())
    }
    fn get_bad_boolean(&self, key: &str) -> Option<bool> {
        self.get(key).and_then(|value| value.to_bad_boolean())
    }
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

    fn get_string(&self, key: &str) -> Option<String> {
        self.get(key).and_then(|value| value.to_string())
    }
    fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key).and_then(|value| value.to_bool())
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

pub fn find_and_extract_multi_selects(items: &mut HashMap<String, String>) -> Vec<MultiSelect> {
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

        let checkbox_value = match CheckboxValue::from_str(value.as_str()) {
            Ok(ok) => ok,
            Err(err) => {
                error!(?err, "Error parsing checkbox value");
                CheckboxValue::Unchecked
            }
        };

        multi_select.set_values.insert(index, checkbox_value);
    }
    multi_selects.into_values().collect()
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
impl From<f32> for RedCapExportDataType {
    fn from(value: f32) -> Self {
        Self::Float(value)
    }
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
impl From<String> for RedCapExportDataType {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}
impl From<bool> for RedCapExportDataType {
    fn from(value: bool) -> Self {
        Self::Number(value as isize)
    }
}
impl From<NaiveDate> for RedCapExportDataType {
    fn from(value: NaiveDate) -> Self {
        Self::Date(value)
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
impl From<isize> for RedCapExportDataType {
    fn from(value: isize) -> Self {
        Self::Number(value)
    }
}

impl From<MultiSelect> for RedCapExportDataType {
    fn from(value: MultiSelect) -> Self {
        Self::MultiSelect(value)
    }
}
impl<T> From<T> for RedCapExportDataType
where
    T: RedCapEnum,
{
    fn from(value: T) -> Self {
        Self::Number(value.to_usize() as isize)
    }
}
impl RedCapExportDataType {
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
    mut input: HashMap<String, String>,
) -> HashMap<String, RedCapExportDataType> {
    let multi_selects = find_and_extract_multi_selects(&mut input);

    let mut output = HashMap::new();
    for multi_select in multi_selects {
        output.insert(
            multi_select.field_base.clone(),
            RedCapExportDataType::MultiSelect(multi_select),
        );
    }
    for (key, value) in input {
        let value = RedCapExportDataType::process_string(value);
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

        let database = crate::database::tests::setup_red_cap_db_test(&env).await?;
        let client = RedcapClient::new(
            env.get("RED_CAP_TOKEN")
                .context("Missing Red Cap Token")?
                .to_owned(),
        );

        Ok((client, database))
    }
}
