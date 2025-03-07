use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};
pub(crate) mod permissions;
pub(crate) mod red_cap_enum;

/// Implements a bunch of traits for handling enums that are used in the redcap api.
///
/// - [std::fmt::Display](https://doc.rust-lang.org/std/fmt/trait.Display.html)
/// - [std::str::FromStr](https://doc.rust-lang.org/std/str/trait.FromStr.html)
/// - [serde::Serialize](https://docs.serde.rs/serde/trait.Serialize.html)
/// - [serde::Deserialize](https://docs.serde.rs/serde/trait.Deserialize.html)
/// - [sqlx::Type](https://docs.rs/sqlx/latest/sqlx/types/trait.Type.html)
/// - [sqlx::Encode](https://docs.rs/sqlx/latest/sqlx/trait.Encode.html)
/// - [sqlx::Decode](https://docs.rs/sqlx/latest/sqlx/trait.Decode.html)
/// - RedCapEnum
#[proc_macro_derive(RedCapEnum, attributes(red_cap))]
pub fn red_cap_enum(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);

    match red_cap_enum::expand(input) {
        Ok(result) => result.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
#[proc_macro_derive(Permissions, attributes(permission))]
pub fn permissions(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    // Check if its an enum
    let result = permissions::expand(input);
    match result {
        Ok(ok) => ok.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
