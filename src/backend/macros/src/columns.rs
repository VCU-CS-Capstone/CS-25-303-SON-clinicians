use heck::ToUpperCamelCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use syn::{Data, DeriveInput, Field, Ident, LitStr, Result};
fn ident_to_upper_camel(ident: &syn::Ident) -> syn::Ident {
    let ident = ident.to_string().to_upper_camel_case();
    format_ident!("{}", ident)
}
#[derive(Debug)]
pub struct ColumnField {
    pub struct_name: syn::Ident,
    pub ident: syn::Ident,
    pub name: syn::LitStr,
    pub ident_as_upper_camel: syn::Ident,
}
impl ColumnField {
    pub fn new(field: Field, struct_name: Ident) -> Result<Self> {
        let ident = field
            .ident
            .ok_or_else(|| syn::Error::new_spanned(field.ty, "expected named field"))?;
        let name = LitStr::new(ident.to_string().as_str(), ident.span());

        let ident_as_upper_camel = ident_to_upper_camel(&ident);
        Ok(Self {
            struct_name,
            ident,
            name,
            ident_as_upper_camel,
        })
    }
    pub fn enum_variant_def(&self) -> TokenStream {
        let doc_str = format!("Corresponds to  [{}::{}].", self.struct_name, self.ident);
        let doc_lit = LitStr::new(doc_str.as_str(), self.ident.span());
        let ident = &self.ident_as_upper_camel;
        quote! {
            #[doc = #doc_lit]
            #ident
        }
    }
    pub fn display_match_arm(&self) -> TokenStream {
        let ident = &self.ident_as_upper_camel;
        let name = &self.name;
        quote! {
            Self::#ident => f.write_str(#name)
        }
    }
    pub fn column_type_name(&self) -> TokenStream {
        let ident = &self.ident_as_upper_camel;
        let name = &self.name;
        quote! {
            Self::#ident => #name
        }
    }
    pub fn column_type_all(&self) -> TokenStream {
        let ident = &self.ident_as_upper_camel;
        quote! {
            Self::#ident
        }
    }
}
pub fn expand(input: DeriveInput) -> Result<TokenStream> {
    let DeriveInput { ident, data, .. } = input;
    let column_enum_name = format_ident!("{}Column", ident);
    let Data::Struct(data_struct) = data else {
        return Err(syn::Error::new_spanned(ident, "expected struct"));
    };
    let fields = match data_struct.fields {
        syn::Fields::Named(fields_named) => fields_named
            .named
            .into_iter()
            .map(|field| ColumnField::new(field, ident.clone()))
            .collect::<Result<Vec<_>>>()?,
        _ => return Err(syn::Error::new_spanned(ident, "expected named fields")),
    };
    let enum_variants: Vec<_> = fields
        .iter()
        .map(|field| field.enum_variant_def())
        .collect();
    let display_match_arms: Vec<_> = fields
        .iter()
        .map(|field| field.display_match_arm())
        .collect();
    let column_type_match_arms: Vec<_> = fields
        .iter()
        .map(|field| field.column_type_name())
        .collect();
    let column_type_all: Vec<_> = fields.iter().map(|field| field.column_type_all()).collect();
    let result = quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum #column_enum_name {
            #(
                #enum_variants
            ),*
        }
        const _: () = {
            impl std::fmt::Display for #column_enum_name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        #(
                            #display_match_arms
                        ),*
                    }
                }
            }
            impl ColumnType for #column_enum_name {
                fn column_name(&self) -> &'static str {
                    match self {
                        #(
                            #column_type_match_arms
                        ),*
                    }
                }
                fn all() -> std::vec::Vec<Self>
                    where
                        Self: Sized {
                    std::vec![
                        #(
                            #column_type_all
                        ),*
                    ]
                }
            }
        };

    };

    Ok(result)
}
