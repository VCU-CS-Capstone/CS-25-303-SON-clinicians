//!
//! ```rust,ignore
//! #[derive(Permissions)]
//! pub enum UserPermissions {
//!    /// Ignores all other permissions
//!    #[permission(key = "admin", title= "Admin" category = "System")]
//!    Admin,
//! }
//! ```
use std::borrow::Cow;

use proc_macro2::TokenStream;
use quote::quote;
use syn::Attribute;
use syn::Data;
use syn::DeriveInput;
use syn::Expr;
use syn::Ident;
use syn::Lit;
use syn::LitStr;
use syn::Result;
mod keywords {
    syn::custom_keyword!(key);
    syn::custom_keyword!(title);
    syn::custom_keyword!(category);
}
#[derive(Debug)]
pub struct PermissionAttributes {
    title: LitStr,
    key: Option<LitStr>,
    category: Option<LitStr>,
}
pub struct ScopeEntry {
    ident: Ident,
    attribute: PermissionAttributes,
    docs: LitStr,
}
impl ScopeEntry {
    pub fn description_tokens(&self) -> TokenStream {
        let Self {
            ident,
            attribute,
            docs,
        } = self;
        let PermissionAttributes {
            title, category, ..
        } = attribute;
        let category = if let Some(category) = category {
            quote! { Some(#category) }
        } else {
            quote! { None }
        };
        let scope = quote! {
            Self::#ident => PermissionDescription{
                key: Self::#ident,
                description: #docs,
                title: #title,
                category: #category,
                ..std::default::Default::default()
            },
        };
        scope
    }

    pub fn as_str(&self) -> TokenStream {
        let name_as_string = self.key();
        let ident = &self.ident;
        let scope = quote! {
            Self::#ident => #name_as_string,
        };
        scope
    }
    #[allow(clippy::wrong_self_convention)]
    pub fn from_string_impl(&self) -> TokenStream {
        let name_as_string = self.key();
        let ident = &self.ident;
        let scope = quote! {
            #name_as_string => Ok(Self::#ident),
        };
        scope
    }
    fn key(&self) -> Cow<'_, LitStr> {
        let Self {
            ident, attribute, ..
        } = self;

        if let Some(key) = &attribute.key {
            Cow::Borrowed(key)
        } else {
            Cow::Owned(LitStr::new(&ident.to_string(), ident.span()))
        }
    }
}
impl syn::parse::Parse for PermissionAttributes {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut title = None;
        let mut key = None;
        let mut category = None;
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(keywords::title) {
                input.parse::<keywords::title>()?;
                input.parse::<syn::Token![=]>()?;
                title = input.parse()?;
            } else if lookahead.peek(keywords::category) {
                input.parse::<keywords::category>()?;
                input.parse::<syn::Token![=]>()?;
                category = Some(input.parse()?);
            } else if lookahead.peek(keywords::key) {
                input.parse::<keywords::key>()?;
                input.parse::<syn::Token![=]>()?;
                key = Some(input.parse()?);
            } else {
                return Err(lookahead.error());
            }
            if input.peek(syn::Token![,]) {
                input.parse::<syn::Token![,]>()?;
            }
        }
        let title = title.ok_or_else(|| input.error("No title found"))?;
        Ok(Self {
            title,
            key,
            category,
        })
    }
}

pub(crate) fn expand(derive_input: DeriveInput) -> Result<TokenStream> {
    let DeriveInput { ident, data, .. } = derive_input;
    let Data::Enum(data_enum) = data else {
        return Err(syn::Error::new(ident.span(), "Expected an enum"));
    };
    let mut entries = Vec::new();
    for variant in data_enum.variants {
        if !variant.fields.is_empty() {
            return Err(syn::Error::new_spanned(variant, "Expected a unit variant"));
        }

        let mut attribute = None;
        let mut doc_comments = String::new();
        for attr in variant.attrs.iter() {
            if attr.path().is_ident("doc") {
                let doc_str = doc_attr_to_string(attr)?;
                doc_comments.push_str(doc_str.trim());
            } else if attr.path().is_ident("permission") {
                let meta = attr.parse_args::<PermissionAttributes>()?;
                attribute = Some(meta);
            }
        }
        let attribute = attribute
            .ok_or_else(|| syn::Error::new_spanned(&variant, "Expected a scope attribute"))?;
        let doc_comment = LitStr::new(&doc_comments, variant.ident.span());
        entries.push(ScopeEntry {
            ident: variant.ident,
            attribute,
            docs: doc_comment,
        });
    }
    let descriptions = entries.iter().map(ScopeEntry::description_tokens);
    let as_str = entries.iter().map(ScopeEntry::as_str);
    let from_string = entries.iter().map(ScopeEntry::from_string_impl);
    let result = quote! {
        impl #ident {
            #[allow(clippy::needless_update)]
            pub fn description(&self) -> PermissionDescription {
                match self {
                    #(#descriptions)*
                }
            }
        }
        impl std::fmt::Display for #ident{
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let name = self.as_ref();
                write!(f, "{}", name)
            }
        }
        impl std::convert::AsRef <str> for #ident{
            fn as_ref(&self) -> &str {
                match self {
                    #(#as_str)*
                }
            }
        }
        impl std::convert::TryFrom<&str> for #ident{
            type Error = InvalidPermission;
            fn try_from(value: &str) -> Result<Self, Self::Error> {
                match value {
                    #(#from_string)*
                    _ => Err(InvalidPermission::from(value.to_owned())),
                }
            }
        }
        impl  std::str::FromStr for #ident{
            type Err = InvalidPermission;
            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::try_from(value)
            }
        }
        impl std::convert::TryFrom<String> for #ident{
            type Error = InvalidPermission;
            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::try_from(value.as_str())
            }
        }
        const _: () ={
            impl serde::Serialize for #ident{
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    let name: &str = self.as_ref();
                    serializer.serialize_str(name)
                }
            }
            impl<'de> serde::Deserialize<'de> for #ident {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    let value = String::deserialize(deserializer)?;
                    Self::try_from(value).map_err(serde::de::Error::custom)
                }
            }
            impl sqlx::Type<sqlx::Postgres> for #ident {
                fn type_info() -> ::sqlx::postgres::PgTypeInfo {
                    sqlx::postgres::PgTypeInfo::with_name("VARCHAR")
                }
            }
            impl sqlx::postgres::PgHasArrayType for #ident {
                fn array_type_info() -> ::sqlx::postgres::PgTypeInfo {
                    sqlx::postgres::PgTypeInfo::array_of("VARCHAR")
                }
            }
            #[automatically_derived]
            impl<'q> sqlx::encode::Encode<'q, sqlx::Postgres> for #ident {
                fn encode_by_ref(
                    &self,
                    buf: &mut <sqlx::Postgres as sqlx::Database>::ArgumentBuffer<'q>,
                ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
                    let val: &str = self.as_ref();
                    <&str as sqlx::encode::Encode<'q, sqlx::Postgres>>::encode_by_ref(&val, buf)
                }
                fn size_hint(&self) -> usize {
                    let val: &str = self.as_ref();

                    <&str as sqlx::encode::Encode<'q, sqlx::Postgres>>::size_hint(&val)
                }
            }


            #[automatically_derived]
            impl<'r> sqlx::decode::Decode<'r, ::sqlx::postgres::Postgres> for #ident {
                fn decode(
                    value: sqlx::postgres::PgValueRef<'r>,
                ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
                    let value =
                        <&'r str as sqlx::decode::Decode<'r, sqlx::postgres::Postgres>>::decode(value)?;
                    Ok(Self::try_from(value)?)
                }
            }
        };

    };
    Ok(result)
}

fn doc_attr_to_string(attr: &Attribute) -> Result<String> {
    match &attr.meta {
        syn::Meta::NameValue(syn::MetaNameValue { value, .. }) => match value {
            Expr::Lit(lit) => match &lit.lit {
                Lit::Str(lit_str) => Ok(lit_str.value()),
                _ => Err(syn::Error::new_spanned(lit, "Expected a string literal")),
            },
            _ => Err(syn::Error::new_spanned(value, "Expected a string literal")),
        },
        _ => Err(syn::Error::new_spanned(
            &attr.meta,
            "Expected a string literal",
        )),
    }
}

#[cfg(test)]
mod tests {
    use syn::{Attribute, DeriveInput};

    #[test]
    fn test() {
        let input = r#"
        pub enum NRScope {
            /// Can read all repositories the user has access to
            #[permission(title = "Read Repository", category = "Repository")]
            ReadRepository,
        }
        "#;

        let derive_input = syn::parse_str::<syn::DeriveInput>(input).unwrap();

        let result = super::expand(derive_input).unwrap();

        let value = result.to_string();
        let syn_file = syn::parse_file(&value).unwrap();
        let prettyplease = prettyplease::unparse(&syn_file);
        println!("{}", prettyplease);
    }

    #[test]
    fn test_attribute() {
        let attribute = create_attribute(
            r#"
            #[permission(title = "Read Repository", category = "Repository")]
            "#,
        );
        let result = attribute
            .parse_args::<super::PermissionAttributes>()
            .unwrap();
        assert_eq!(result.title.value(), "Read Repository");
        assert_eq!(result.category.unwrap().value(), "Repository");
    }

    fn create_attribute(attribute: &str) -> Attribute {
        let actual_input = format!(
            r#"
            {attribute}
            struct Test;
            "#
        );
        let input = syn::parse_str::<DeriveInput>(&actual_input).unwrap();
        let attributes = input.attrs;
        attributes[0].clone()
    }
}
