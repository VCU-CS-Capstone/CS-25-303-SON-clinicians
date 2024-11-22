pub fn display_option<T>(o: &Option<T>) -> String
where
    T: std::fmt::Display,
{
    match o {
        Some(s) => format!("Some({})", s),
        None => "None".to_string(),
    }
}
