use serde::{Deserialize, Serialize};
use strum::EnumIs;
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", content = "value")]
pub enum NumberQuery<I = i32> {
    GreaterThan(I),
    LessThan(I),
    EqualTo(I),
    GreaterThanOrEqualTo(I),
    LessThanOrEqualTo(I),
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIs, Serialize, Deserialize, ToSchema)]
#[serde(untagged)]
pub enum ItemOrArray<T> {
    Item(T),
    Array(Vec<T>),
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_deserialize_array() {
        let json = r#"[1,2,3]"#;
        let result: super::ItemOrArray<i32> = serde_json::from_str(json).unwrap();
        assert_eq!(result, super::ItemOrArray::Array(vec![1, 2, 3]));
    }

    #[test]
    fn test_deserialize_item() {
        let json = r#"1"#;
        let result: super::ItemOrArray<i32> = serde_json::from_str(json).unwrap();
        assert_eq!(result, super::ItemOrArray::Item(1));
    }
}
