use std::fmt::Display;
mod insert;
mod select;
mod update;
pub use insert::*;
pub use select::*;
use sqlx::{
    postgres::PgRow,
    query::{Query, QueryAs, QueryScalar},
    Database, FromRow, Postgres,
};
use tracing::trace;
pub struct FunctionCallColumn<C> {
    pub function_name: &'static str,
    pub column: C,
}

pub trait QueryTool<'args> {
    fn sql(&mut self) -> &str;

    fn take_arguments_or_error(&mut self) -> <Postgres as Database>::Arguments<'args>;

    fn query(&mut self) -> Query<'_, Postgres, <Postgres as Database>::Arguments<'args>> {
        let args = self.take_arguments_or_error();
        let sql = self.sql();
        if tracing::enabled!(tracing::Level::TRACE) {
            trace!(?sql, "Generated SQL");
        }
        sqlx::query_with(sql, args)
    }
    fn query_as<T>(&mut self) -> QueryAs<'_, Postgres, T, <Postgres as Database>::Arguments<'args>>
    where
        T: for<'r> FromRow<'r, PgRow>,
    {
        let args = self.take_arguments_or_error();

        let sql = self.sql();
        if tracing::enabled!(tracing::Level::TRACE) {
            trace!(?sql, "Generated SQL");
        }
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
        if tracing::enabled!(tracing::Level::TRACE) {
            trace!(?sql, "Generated SQL");
        }
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
pub enum AndOr {
    And,
    Or,
}
impl Display for AndOr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::And => write!(f, "AND"),
            Self::Or => write!(f, "OR"),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WhereComparison {
    Equals,
    Like,
}
impl Display for WhereComparison {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Equals => write!(f, "="),
            Self::Like => write!(f, "LIKE"),
        }
    }
}
