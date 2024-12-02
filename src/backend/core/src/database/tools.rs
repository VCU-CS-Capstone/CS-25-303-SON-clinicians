use std::fmt::Display;
use std::ops::Deref;
mod insert;
mod select;
mod select_v2;
mod update;
mod where_sql;
use derive_more::derive::{From, Into};
pub use insert::*;
pub use select::*;
pub use select_v2::*;
use serde::{Deserialize, Serialize};
use sqlx::Arguments;
use sqlx::{
    postgres::PgRow,
    query::{Query, QueryAs, QueryScalar},
    Database, FromRow, Postgres,
};
use strum::{AsRefStr, Display};
use tracing::trace;
pub use update::*;
use utoipa::{IntoParams, ToSchema};
pub use where_sql::*;
pub struct FunctionCallColumn<C> {
    pub function_name: &'static str,
    pub column: C,
}
pub trait HasArguments<'args> {
    fn take_arguments_or_error(&mut self) -> <Postgres as Database>::Arguments<'args>;

    fn borrow_arguments_or_error(&mut self) -> &mut <Postgres as Database>::Arguments<'args>;

    fn push_argument<T>(&mut self, value: T) -> usize
    where
        T: 'args + sqlx::Encode<'args, Postgres> + sqlx::Type<Postgres>,
    {
        let arguments = self.borrow_arguments_or_error();
        arguments.add(value).expect("Failed to add argument");
        arguments.len()
    }
}

pub trait QueryTool<'args>: HasArguments<'args> {
    fn sql(&mut self) -> &str;

    fn query(&mut self) -> Query<'_, Postgres, <Postgres as Database>::Arguments<'args>> {
        let args = self.take_arguments_or_error();
        let sql = self.sql();
        trace!(?sql, "Generated SQL");

        sqlx::query_with(sql, args)
    }
    fn query_as<T>(&mut self) -> QueryAs<'_, Postgres, T, <Postgres as Database>::Arguments<'args>>
    where
        T: for<'r> FromRow<'r, PgRow>,
    {
        let args = self.take_arguments_or_error();

        let sql = self.sql();
        trace!(?sql, "Generated SQL");
        sqlx::query_as_with(sql, args)
    }
    fn query_scalar<O>(
        &mut self,
    ) -> QueryScalar<'_, Postgres, O, <Postgres as Database>::Arguments<'args>>
    where
        (O,): for<'r> FromRow<'r, PgRow>,
    {
        let args = self.take_arguments_or_error();

        let sql = self.sql();
        trace!(?sql, "Generated SQL");
        sqlx::query_scalar_with(sql, args)
    }
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
#[derive(Debug, Clone, Copy, From, Into, Deserialize, ToSchema, IntoParams)]
#[serde(default)]
#[into_params(parameter_in = Query)]
pub struct PageParams {
    /// The number of items per page
    #[param(default = 10)]
    pub page_size: i32,
    /// The page number
    #[param(default = 1)]
    pub page_number: i32,
}
impl PageParams {
    pub fn page_index(&self) -> i32 {
        (self.page_number - 1).max(0)
    }
    pub fn offset(&self) -> i32 {
        self.page_size * self.page_index()
    }

    pub fn number_of_pages(&self, total: i64) -> i32 {
        (total as f64 / self.page_size as f64).ceil() as i32
    }
}
impl Default for PageParams {
    fn default() -> Self {
        Self {
            page_size: 10,
            page_number: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PaginatedResponse<T> {
    /// The number of items per page
    pub total_pages: i32,
    /// The total number of items
    pub total: i64,
    /// The data for the current page
    pub data: Vec<T>,
}
impl<T> Deref for PaginatedResponse<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> Default for PaginatedResponse<T> {
    fn default() -> Self {
        Self {
            total_pages: 0,
            total: 0,
            data: Vec::new(),
        }
    }
}