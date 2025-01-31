use strum::EnumIs;

use super::RedCapParseError;
use std::str::FromStr;
/// So redcap stores multi check boxes like this
/// `{field_name}___{index}`
///
/// This function will split the field name and the index
///
/// # Example
/// ```
/// use cs25_303_core::red_cap::api::utils::FieldNameAndIndex;
/// use std::str::FromStr;
/// let value = FieldNameAndIndex::try_from("health_ed___1").unwrap();
/// assert_eq!(value.field_name, "health_ed");
/// assert_eq!(value.index, 1);
/// ```

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldNameAndIndex<'a> {
    pub field_name: &'a str,
    pub index: i32,
}
impl<'a> TryFrom<&'a str> for FieldNameAndIndex<'a> {
    type Error = RedCapParseError;
    fn try_from(field_name: &'a str) -> Result<Self, Self::Error> {
        if !field_name.contains("___") {
            return Err(RedCapParseError::NotAValidCheckBoxKey(
                field_name.to_owned(),
            ));
        }
        let mut parts = field_name.splitn(2, "___");
        let actual_field_name = parts.next().unwrap();
        let index = parts.next();
        if let Some(index) = index {
            let index = i32::from_str(index).map_err(|err| {
                RedCapParseError::InvalidMultiCheckboxField {
                    input: field_name.to_owned(),
                    reason: err.into(),
                }
            })?;
            Ok(Self {
                field_name: actual_field_name,
                index,
            })
        } else {
            Err(RedCapParseError::NotAValidCheckBoxKey(
                field_name.to_owned(),
            ))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIs)]
pub enum CheckboxValue {
    Checked,
    Unchecked,
}
impl From<CheckboxValue> for bool {
    fn from(value: CheckboxValue) -> Self {
        match value {
            CheckboxValue::Checked => true,
            CheckboxValue::Unchecked => false,
        }
    }
}
impl From<CheckboxValue> for usize {
    fn from(val: CheckboxValue) -> Self {
        match val {
            CheckboxValue::Checked => 1,
            CheckboxValue::Unchecked => 0,
        }
    }
}
impl FromStr for CheckboxValue {
    type Err = RedCapParseError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "Checked" | "1" => Ok(Self::Checked),
            "Unchecked" | "0" | "" => Ok(Self::Unchecked),
            _ => Err(RedCapParseError::InvalidMultiCheckboxField {
                input: value.to_owned(),
                reason: super::GenericError::Other("Invalid value".to_owned()),
            }),
        }
    }
}
impl TryFrom<serde_json::Value> for CheckboxValue {
    type Error = RedCapParseError;
    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        match value {
            serde_json::Value::String(value) => value.parse(),
            serde_json::Value::Number(value) => {
                let value = value.as_u64().unwrap();
                if value == 1 {
                    Ok(Self::Checked)
                } else {
                    Ok(Self::Unchecked)
                }
            }
            _ => Err(RedCapParseError::InvalidMultiCheckboxField {
                input: value.to_string(),
                reason: super::GenericError::Other("Invalid value".to_owned()),
            }),
        }
    }
}
pub fn is_check_box_item(value: &str) -> bool {
    value.contains("___")
}
