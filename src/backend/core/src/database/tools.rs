use std::fmt::Display;
mod insert;
mod pagination;
mod select;
mod select_v2;
mod traits;
mod update;
mod where_sql;
pub use insert::*;
pub use pagination::*;
pub use select::*;
pub use select_v2::*;
use strum::{AsRefStr, Display};
pub use traits::*;
pub use update::*;
pub use where_sql::*;
pub struct FunctionCallColumn<C> {
    pub function_name: &'static str,
    pub column: C,
}

pub trait TableType {
    type Columns: ColumnType;
    fn table_name() -> &'static str
    where
        Self: Sized;
}
pub trait ColumnType {
    fn column_name(&self) -> &'static str;

    fn format_column_with_prefix(&self, prefix: &str) -> String {
        format!("{}.{}", prefix, self.column_name())
    }
    fn all() -> Vec<Self>
    where
        Self: Sized;

    fn lower(&self) -> FunctionCallColumn<Self>
    where
        Self: Sized + Copy,
    {
        FunctionCallColumn {
            function_name: "LOWER",
            column: *self,
        }
    }
    fn upper(&self) -> FunctionCallColumn<Self>
    where
        Self: Sized + Copy,
    {
        FunctionCallColumn {
            function_name: "UPPER",
            column: *self,
        }
    }
}
pub fn concat_columns<T>(columns: &[T], prefix: Option<&str>) -> String
where
    T: ColumnType,
{
    if let Some(prefix) = prefix {
        columns
            .iter()
            .map(|column| column.format_column_with_prefix(prefix))
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        columns
            .iter()
            .map(|column| column.column_name())
            .collect::<Vec<_>>()
            .join(", ")
    }
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
