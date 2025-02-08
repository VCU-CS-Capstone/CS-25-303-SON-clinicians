use serde::{Deserialize, Serialize};
use strum::EnumIs;
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", content = "value")]

pub enum NumberQuery<I = i32> {
    GreaterThan(I),
    LessThan(I),
    EqualTo(I),
    NotEqualTo(I),
    GreaterThanOrEqualTo(I),
    LessThanOrEqualTo(I),
}

#[derive(Debug, Clone, PartialEq, Eq, ToSchema, EnumIs)]
pub enum ItemOrArray<T> {
    Item(T),
    Array(Vec<T>),
}

mod item_or_array_serde {
    impl<T> serde::Serialize for super::ItemOrArray<T>
    where
        T: serde::Serialize,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            match self {
                super::ItemOrArray::Item(item) => item.serialize(serializer),
                super::ItemOrArray::Array(array) => array.serialize(serializer),
            }
        }
    }

    macro_rules! deserialize_num {
        ($mod_name:ident => fn $fn_name:ident($type:ty)  {
            $(fn $inner_fn_type:ident($inner_type:ty)),*
        }) => {
            mod mod_name {
                use super::super::ItemOrArray;
                use serde::Deserializer;

                struct ItemOrArrayVisitor;

                impl<'de> serde::de::Visitor<'de> for ItemOrArrayVisitor {
                    type Value = ItemOrArray<$type>;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        write!(formatter, "a number or an array of numbers")
                    }

                    fn $fn_name<E>(self, value: $type) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        Ok(ItemOrArray::Item(value as $type))
                    }

                    $(
                        fn $inner_fn_type<E>(self, value: $inner_type) -> Result<Self::Value, E>
                        where
                            E: serde::de::Error,
                        {
                            Ok(ItemOrArray::Item(value as $type))
                        }
                    )*
                    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                    where
                        A: serde::de::SeqAccess<'de>,
                    {
                        let mut array = Vec::new();
                        while let Some(value) = seq.next_element()? {
                            array.push(value);
                        }
                        Ok(ItemOrArray::Array(array))
                    }
                }
                impl<'de> serde::de::Deserialize<'de> for ItemOrArray<$type> {
                    fn deserialize<D>(deserializer: D) -> Result<ItemOrArray<$type>, D::Error>
                    where
                        D: Deserializer<'de>,
                    {
                        deserializer.deserialize_any(ItemOrArrayVisitor)
                    }
                }
            }
        };
    }
    deserialize_num!(i32_deserialize => fn visit_i32(i32) {
        fn visit_i64(i64),
        fn visit_u8(u8),
        fn visit_u16(u16),
        fn visit_u32(u32),
        fn visit_u64(u64)
    });
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
