use ahash::{HashMap, HashMapExt};
use serde_json::Value;
use tracing::error;

use super::{
    api::utils::{is_check_box_item, CheckboxValue, FieldNameAndIndex},
    MultiSelect, RedCapExportDataType,
};
/// Removes all multi select items and returns them as a new HashMap of MultiSelect
pub fn find_and_extract_multi_selects(
    items: &mut HashMap<String, Value>,
) -> HashMap<String, MultiSelect> {
    let mut multi_selects = HashMap::new();
    let keys: Vec<_> = items
        .keys()
        .filter_map(|key| {
            if is_check_box_item(key) {
                Some(key.clone())
            } else {
                None
            }
        })
        .collect();
    for key in keys {
        let value = items.remove(&key).unwrap();
        let FieldNameAndIndex { field_name, index } =
            FieldNameAndIndex::try_from(key.as_str()).unwrap();

        let multi_select = if let Some(multi_select) = multi_selects.get_mut(field_name) {
            multi_select
        } else {
            multi_selects.insert(field_name.to_owned(), MultiSelect::new(field_name));
            multi_selects.get_mut(field_name).unwrap()
        };

        let checkbox_value = match CheckboxValue::try_from(value) {
            Ok(ok) => ok,
            Err(err) => {
                error!(?err, "Error parsing checkbox value");
                CheckboxValue::Unchecked
            }
        };

        multi_select.insert(index, checkbox_value);
    }
    multi_selects
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
/// Flattens the [RedCapExportDataType] into a HashMap<String, String>
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
