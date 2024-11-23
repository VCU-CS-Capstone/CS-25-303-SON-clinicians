use proc_macro::TokenStream;
pub(crate) mod columns;
pub(crate) mod red_cap_enum;
/// This macro is used to derive the `ColumnType` trait for a struct.
/// To use Properly each field must be the same name as the column in the database.
#[proc_macro_derive(Columns)]
pub fn columns(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);

    match columns::expand(input) {
        Ok(result) => result.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
#[proc_macro_derive(RedCapEnum, attributes(red_cap))]
pub fn red_cap_enum(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);

    match red_cap_enum::expand(input) {
        Ok(result) => result.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
