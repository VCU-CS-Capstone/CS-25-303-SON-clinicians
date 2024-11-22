use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Data, DeriveInput, Ident, LitInt, LitStr, Result, Variant,
};
mod kw {
    syn::custom_keyword!(enum_index);
    syn::custom_keyword!(name);
    syn::custom_keyword!(other);
}
fn ident_to_lit_str(ident: &Ident) -> LitStr {
    LitStr::new(&ident.to_string(), ident.span())
}
pub struct RedCapAttr {
    pub enum_index: Option<LitInt>,
    pub name: Option<LitStr>,
    pub is_default: bool,
}
impl Parse for RedCapAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut enum_index = None;
        let mut is_default = false;
        let mut name = None;
        while !input.is_empty() {
            if input.peek(kw::enum_index) {
                input.parse::<kw::enum_index>()?;
                input.parse::<syn::Token![=]>()?;
                enum_index = Some(input.parse()?);
            } else if input.peek(kw::other) {
                input.parse::<kw::other>()?;
                is_default = true;
            } else if input.peek(kw::name) {
                input.parse::<kw::name>()?;
                input.parse::<syn::Token![=]>()?;
                name = Some(input.parse()?);
            } else {
                return Err(input.error("expected `enum_index` or `other`"));
            }
            if !input.is_empty() {
                input.parse::<syn::Token![,]>()?;
            }
        }

        Ok(Self {
            enum_index,
            is_default,
            name,
        })
    }
}
pub struct RedCapEnumVariant {
    pub variant: Variant,
    pub enum_index: LitInt,
    pub name: LitStr,
    pub is_default: bool,
    pub from_usize_after: TokenStream,
    pub to_usize_after: TokenStream,
}
impl RedCapEnumVariant {
    fn new(variant: Variant) -> Result<Self> {
        let Some(attr) = variant
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("red_cap"))
            .map(|attr| attr.parse_args::<RedCapAttr>())
            .transpose()?
        else {
            return Err(syn::Error::new_spanned(
                variant,
                "expected at least one `#[red_cap(\"...\"))]` attribute",
            ));
        };
        let enum_index = attr.enum_index.ok_or_else(|| {
            syn::Error::new(
                variant.span(),
                "expected at least one `#[red_cap(enum_index = \"...\")]` attribute",
            )
        })?;
        let (from, to) = match &variant.fields {
            syn::Fields::Named(fields_named) => {
                return Err(syn::Error::new_spanned(
                    fields_named,
                    "Not Supported at the moment",
                ));
            }
            syn::Fields::Unnamed(fields_unnamed) => {
                if fields_unnamed.unnamed.len() != 1 {
                    return Err(syn::Error::new_spanned(
                        fields_unnamed,
                        "expected exactly one field",
                    ));
                }
                (quote! { (Default::default()) }, quote! { (_) })
            }
            syn::Fields::Unit => (TokenStream::new(), TokenStream::new()),
        };
        let name = attr
            .name
            .unwrap_or_else(|| ident_to_lit_str(&variant.ident));
        Ok(Self {
            variant,
            name,
            is_default: attr.is_default,
            enum_index,
            from_usize_after: from,
            to_usize_after: to,
        })
    }
    #[allow(clippy::wrong_self_convention)]
    fn from_enum_index(&self) -> TokenStream {
        let enum_index = &self.enum_index;
        let variant = &self.variant.ident;
        let from = &self.from_usize_after;
        quote! {
            #enum_index => Some(Self::#variant #from),
        }
    }

    fn to_enum_index(&self) -> TokenStream {
        let variant = &self.variant.ident;
        let enum_index = &self.enum_index;
        let to = &self.to_usize_after;
        quote! {
            Self::#variant #to => #enum_index,
        }
    }

    fn match_from_str(&self, wrap_in_ok: bool) -> TokenStream {
        let variant = &self.variant.ident;
        if self.is_default {
            quote! {
                _ => Self::#variant(value.to_owned()),
            }
        } else if wrap_in_ok {
            let name = &self.name;
            quote! {
                #name => Ok(Self::#variant),
            }
        } else {
            let name = &self.name;
            quote! {
                #name => Self::#variant,
            }
        }
    }
    fn match_from_string(&self, wrap_in_ok: bool) -> TokenStream {
        let variant = &self.variant.ident;
        if self.is_default {
            quote! {
                _ => Self::#variant(value.to_owned()),
            }
        } else if wrap_in_ok {
            let name = &self.name;
            quote! {
                #name => Ok(Self::#variant),
            }
        } else {
            let name = &self.name;
            quote! {
                #name => Self::#variant.into(),
            }
        }
    }

    fn match_display(&self) -> TokenStream {
        let variant = &self.variant.ident;
        if self.is_default {
            quote! {
                Self::#variant(value) => write!(f, "{}", value),
            }
        } else {
            let value = &self.name;
            quote! {
                Self::#variant => write!(f, "{}", #value),
            }
        }
    }
    fn match_as_ref(&self) -> TokenStream {
        let variant = &self.variant.ident;

        if self.is_default {
            quote! {
                Self::#variant(d) => d,
            }
        } else {
            let value = &self.name;
            quote! {
                Self::#variant => #value,
            }
        }
    }
}
pub fn expand(input: DeriveInput) -> Result<TokenStream> {
    let DeriveInput { ident, data, .. } = input;

    let Data::Enum(data_enum) = data else {
        return Err(syn::Error::new_spanned(ident, "expected enum"));
    };

    let mut variants = vec![];
    for variant in data_enum.variants {
        variants.push(RedCapEnumVariant::new(variant)?);
    }
    let contains_default = variants.iter().any(|v| v.is_default);
    let from_usize_impl: Vec<_> = variants.iter().map(|v| v.from_enum_index()).collect();
    let to_usize_impl: Vec<_> = variants.iter().map(|v| v.to_enum_index()).collect();

    let match_display: Vec<_> = variants
        .iter()
        .map(|variant| variant.match_display())
        .collect();
    let match_as_ref: Vec<_> = variants
        .iter()
        .map(|variant| variant.match_as_ref())
        .collect();
    let sql_type: LitStr = if contains_default {
        LitStr::new("TEXT", ident.span())
    } else {
        LitStr::new("VARCHAR", ident.span())
    };

    let from_string = if contains_default {
        from_with_default(&ident, &variants)
    } else {
        from_no_default(&ident, &variants)
    };
    let result = quote! {
        impl RedCapEnum for #ident {
            fn from_usize(value: usize) -> Option<Self>
            where
                Self: Sized,
            {
                match value {
                    #(#from_usize_impl)*
                    _ => None,
                }
            }
            fn to_usize(&self) -> usize {
                match self {
                    #(#to_usize_impl)*
                }
            }
        }
        const _: () = {
            #from_string
            #[automatically_derived]
            impl serde::Serialize for #ident {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    let value = self.as_ref();
                    serializer.serialize_str(value)
                }
            }
            #[automatically_derived]
            impl std::fmt::Display for #ident {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        #(
                            #match_display
                        )*
                    }
                }
            }
            #[automatically_derived]
            impl std::convert::AsRef <str> for #ident {
                fn as_ref(&self) -> &str {
                    match self {
                        #(
                            #match_as_ref
                        )*
                    }
                }
            }
            impl sqlx::Type<::sqlx::Postgres> for #ident {
                fn type_info() -> ::sqlx::postgres::PgTypeInfo {
                    sqlx::postgres::PgTypeInfo::with_name(#sql_type)
                }
            }
            impl sqlx::postgres::PgHasArrayType for #ident {
                fn array_type_info() -> ::sqlx::postgres::PgTypeInfo {
                    sqlx::postgres::PgTypeInfo::array_of(#sql_type)
                }
            }
            impl<'q, DB: sqlx::Database> sqlx::encode::Encode<'q, DB> for #ident
            where
                String: sqlx::encode::Encode<'q, DB>,
            {
                fn encode_by_ref(
                    &self,
                    buf: &mut <DB as sqlx::Database>::ArgumentBuffer<'q>,
                ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
                    let val: &str = self.as_ref();
                    let val: String = val.to_owned();
                    val.encode(buf)
                }
                fn size_hint(&self) -> usize {
                    self.as_ref().size_hint()
                }
            }
        };
    };
    Ok(result)
}
fn from_no_default(ident: &Ident, variants: &[RedCapEnumVariant]) -> TokenStream {
    let match_from_str: Vec<_> = variants
        .iter()
        .map(|variant| variant.match_from_str(true))
        .collect();
    let match_from_string: Vec<_> = variants
        .iter()
        .map(|variant| variant.match_from_string(true))
        .collect();
    quote! {
        #[automatically_derived]
        impl std::convert::TryFrom<&str> for #ident{
            type Error = InvalidVariant;
            fn try_from(value: &str) -> Result<Self, Self::Error> {
                match value {
                    #(
                        #match_from_str
                    )*
                    other => Err(InvalidVariant(other.to_owned())),
                }
            }
        }
        #[automatically_derived]
        impl std::convert::TryFrom<String> for #ident {
            type Error = InvalidVariant;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                match value.as_str() {
                    #(
                        #match_from_string
                    )*

                    other => Err(InvalidVariant(other.to_owned())),
                }
            }
        }
        #[automatically_derived]
        impl<'de> serde::Deserialize<'de> for #ident {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Self::try_from(value).map_err(serde::de::Error::custom)
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
    }
}
fn from_with_default(ident: &Ident, variants: &[RedCapEnumVariant]) -> TokenStream {
    let match_from_str: Vec<_> = variants
        .iter()
        .map(|variant| variant.match_from_str(false))
        .collect();
    let match_from_string: Vec<_> = variants
        .iter()
        .map(|variant| variant.match_from_string(false))
        .collect();
    quote! {
        #[automatically_derived]
        impl std::convert::From<&str> for #ident {
            fn from(value: &str) -> Self {
                match value {
                    #(
                        #match_from_str
                    )*
                }
            }
        }
        #[automatically_derived]
        impl std::convert::From<String> for #ident {
            fn from(value: String) -> Self {
                match value.as_str() {
                    #(
                        #match_from_string
                    )*
                }
            }
        }
        #[automatically_derived]
        impl<'de> serde::Deserialize<'de> for #ident {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Ok(Self::from(value))
            }
        }
        #[automatically_derived]
        impl<'r> sqlx::decode::Decode<'r, ::sqlx::postgres::Postgres> for #ident {
            fn decode(
                value: sqlx::postgres::PgValueRef<'r>,
            ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
                let value =
                    <&'r str as sqlx::decode::Decode<'r, sqlx::postgres::Postgres>>::decode(value)?;
                Ok(Self::from(value))
            }
        }
    }
}
