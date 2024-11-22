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
/// use vcu_red_cap::api::utils::FieldNameAndIndex;
/// use std::str::FromStr;
/// let value = FieldNameAndIndex::from_str("health_ed___1").unwrap();
/// assert_eq!(value.field_name, "health_ed");
/// assert_eq!(value.index, Some(1));
/// ```

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldNameAndIndex {
    pub field_name: String,
    pub index: Option<i32>,
}
impl FromStr for FieldNameAndIndex {
    type Err = RedCapParseError;
    fn from_str(field_name: &str) -> Result<Self, Self::Err> {
        if !field_name.contains("___") {
            return Ok(Self {
                field_name: field_name.to_owned(),
                index: None,
            });
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
                field_name: actual_field_name.to_owned(),
                index: Some(index),
            })
        } else {
            Ok(Self {
                field_name: field_name.to_owned(),
                index: None,
            })
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, EnumIs)]
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

pub fn is_check_box_item(value: &str) -> bool {
    value.contains("___")
}
