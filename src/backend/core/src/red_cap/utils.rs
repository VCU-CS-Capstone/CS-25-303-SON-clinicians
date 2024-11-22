macro_rules! is_all_none {
    (
        $(
            $field:ident
        ),*
    ) => {
        if $(   $field.is_none()   )&&* {

            return None;
        }
    };
}
pub(crate) use is_all_none;
