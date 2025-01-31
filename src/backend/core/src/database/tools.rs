use std::{
    borrow::Cow,
    fmt::{Debug, Display},
};
mod insert;
mod pagination;
mod select;
mod table_layout;
mod traits;
mod update;
mod where_sql;
pub use insert::*;
pub use pagination::*;
pub use select::*;
use sqlx::{Database, Postgres};
use strum::{AsRefStr, Display};
pub use table_layout::*;
pub use traits::*;
pub use update::*;
pub use where_sql::*;
pub(crate) mod testing;
pub type PostgresArguments<'args> = <Postgres as Database>::Arguments<'args>;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionCallColumn<C> {
    pub function_name: &'static str,
    pub column: C,
}
impl<C> ColumnType for FunctionCallColumn<C>
where
    C: ColumnType,
{
    fn column_name(&self) -> &'static str {
        self.column.column_name()
    }
    fn formatted_column(&self) -> Cow<'static, str> {
        let inner_formatted = self.column.formatted_column();
        Cow::Owned(format!("{}({})", self.function_name, inner_formatted))
    }
    fn format_column_with_prefix(&self, prefix: Option<&str>) -> Cow<'static, str> {
        let inner_formatted = self.column.format_column_with_prefix(prefix);
        Cow::Owned(format!("{}({})", self.function_name, inner_formatted))
    }
}

pub fn concat_columns<'column, I, C>(columns: I, prefix: Option<&str>) -> String
where
    I: IntoIterator<Item = &'column C>,
    C: ColumnType + 'column,
{
    if prefix.is_some() {
        columns
            .into_iter()
            .map(|column| column.format_column_with_prefix(prefix))
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        columns
            .into_iter()
            .map(|column| column.formatted_column())
            .collect::<Vec<_>>()
            .join(", ")
    }
}
/// Why? Because returning columns won't allow table name
pub fn concat_columns_no_table_name<'column, I, C>(columns: I) -> String
where
    I: IntoIterator<Item = &'column C>,
    C: ColumnType + 'column,
{
    columns
        .into_iter()
        .map(|column| column.column_name())
        .collect::<Vec<_>>()
        .join(", ")
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SQLComparison {
    /// Equals
    ///
    /// `=`
    Equals,
    /// [LIKE](https://www.postgresql.org/docs/current/functions-matching.html#FUNCTIONS-LIKE)
    ///
    /// `LIKE`
    Like,
    /// Not Equals
    ///
    /// `!=`
    NotEquals,
}
impl Display for SQLComparison {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Equals => write!(f, "="),
            Self::NotEquals => write!(f, "!="),
            Self::Like => write!(f, "LIKE"),
        }
    }
}
/// SQL Ordering
#[derive(Debug, Clone, Copy, PartialEq, Display, AsRefStr)]
pub enum SQLOrder {
    #[strum(serialize = "ASC")]
    Ascending,
    #[strum(serialize = "DESC")]
    Descending,
}
/// SQL And Or
#[derive(Debug, Clone, Copy, PartialEq, Display, AsRefStr)]
pub enum AndOr {
    #[strum(serialize = "AND")]
    And,
    #[strum(serialize = "OR")]
    Or,
}
