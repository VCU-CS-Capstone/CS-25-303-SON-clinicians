use std::{
    borrow::Cow,
    fmt::{Debug, Display},
};
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
pub trait TableType {
    type Columns: ColumnType;
    fn table_name() -> &'static str
    where
        Self: Sized;
}
#[derive(Debug)]
pub struct DynColumn(Box<dyn ColumnType + Send + Sync>);
impl DynColumn {
    pub fn new<C>(column: C) -> Self
    where
        C: ColumnType + Send + Sync + 'static,
    {
        Self(Box::new(column))
    }
}
impl ColumnType for DynColumn {
    fn column_name(&self) -> &'static str {
        self.0.column_name()
    }
    fn dyn_column(self) -> DynColumn
    where
        Self: Sized + Send + Sync + 'static,
    {
        self
    }

    fn format_column_with_prefix(&self, prefix: Option<&str>) -> Cow<'static, str> {
        self.0.format_column_with_prefix(prefix)
    }
    fn formatted_column(&self) -> Cow<'static, str> {
        self.0.formatted_column()
    }
}

pub trait ColumnType: Debug + Send + Sync {
    fn column_name(&self) -> &'static str;

    fn formatted_column(&self) -> Cow<'static, str> {
        Cow::Borrowed(self.column_name())
    }
    fn format_column_with_prefix(&self, prefix: Option<&str>) -> Cow<'static, str> {
        if let Some(prefix) = prefix {
            Cow::Owned(format!("{}.{}", prefix, self.column_name()))
        } else {
            Cow::Borrowed(self.column_name())
        }
    }

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
    fn dyn_column(self) -> DynColumn
    where
        Self: Sized + Send + Sync + 'static,
    {
        DynColumn::new(self)
    }
}
impl<C> FormatSql for C
where
    C: ColumnType,
{
    fn format_sql(&self) -> Cow<'_, str> {
        self.formatted_column()
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnFormatWithPrefix<'prefix, 'column, C> {
    column: &'column C,
    prefix: Option<&'prefix str>,
}
impl<C> ColumnType for ColumnFormatWithPrefix<'_, '_, C>
where
    C: ColumnType,
{
    fn column_name(&self) -> &'static str {
        self.column.column_name()
    }
    fn formatted_column(&self) -> Cow<'static, str> {
        self.column.format_column_with_prefix(self.prefix)
    }
    fn format_column_with_prefix(&self, prefix: Option<&str>) -> Cow<'static, str> {
        self.column
            .format_column_with_prefix(prefix.or(self.prefix))
    }
}
impl<'prefix, 'column, C> ColumnFormatWithPrefix<'prefix, 'column, C>
where
    C: ColumnType,
{
    pub fn new(column: &'column C, prefix: Option<&'prefix str>) -> Self {
        Self { column, prefix }
    }
}
pub trait AllColumns {
    fn all() -> Vec<Self>
    where
        Self: Sized;
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
