use std::fmt::Display;

use sqlx::{Arguments, Database, Encode, Postgres, Type};

use super::{concat_columns, AndOr, ColumnType, FunctionCallColumn, QueryTool, WhereComparison};
pub trait SelectColumn {
    fn format_where(&self) -> String;
}

impl<C> SelectColumn for C
where
    C: ColumnType + Copy,
{
    fn format_where(&self) -> String {
        self.column_name().to_string()
    }
}
impl<C> SelectColumn for FunctionCallColumn<C>
where
    C: ColumnType + Copy + Clone,
{
    fn format_where(&self) -> String {
        format!("{}({})", self.function_name, self.column.column_name())
    }
}
pub struct SimpleSelectQueryBuilder<'args> {
    query: String,
    arguments: Option<<Postgres as Database>::Arguments<'args>>,
    created_where: bool,
    added_limit: bool,
}
impl<'args> SimpleSelectQueryBuilder<'args> {
    pub fn new<C>(table: &str, columns: &[C]) -> Self
    where
        C: ColumnType,
    {
        let columns = concat_columns(columns, Some(table));
        let query = format!("SELECT {columns} FROM {table}");
        Self {
            query: query.to_string(),
            arguments: Some(Default::default()),
            created_where: false,
            added_limit: false,
        }
    }

    pub fn push(&mut self, value: impl Display) -> &mut Self {
        self.query.push_str(&value.to_string());
        self
    }
    pub fn push_bind<T>(&mut self, value: T) -> &mut Self
    where
        T: 'args + Encode<'args, Postgres> + Type<Postgres>,
    {
        let arguments = self
            .arguments
            .as_mut()
            .expect("BUG: Arguments taken already");
        arguments.add(value).expect("Failed to add argument");

        arguments
            .format_placeholder(&mut self.query)
            .expect("error in format_placeholder");
        self
    }
    /// Adds a WHERE clause to the query with the given column and value.
    ///
    /// # Example
    /// ```rust
    ///  use cs25_303_core::database::red_cap::participants::health_overview::HealthOverviewColumn;
    ///  use cs25_303_core::database::prelude::*;
    ///  let mut result =
    ///        SimpleSelectQueryBuilder::new("participant_health_overview", &HealthOverviewColumn::all());
    ///    result.where_equals(HealthOverviewColumn::ParticipantId, 1);
    ///
    /// println!("{}", result.sql());
    /// ```
    pub fn where_equals<C, T>(&mut self, column: C, value: T) -> &mut Self
    where
        C: SelectColumn,
        T: 'args + Encode<'args, Postgres> + Type<Postgres>,
    {
        self.where_inner::<C, T>(column, WhereComparison::Equals, value);
        self
    }
    pub fn where_equals_then<C, T, F>(&mut self, column: C, value: T, then: F) -> &mut Self
    where
        C: SelectColumn,
        T: 'args + Encode<'args, Postgres> + Type<Postgres>,
        F: FnOnce(&mut SimpleSelectWhereQueryBuilder<'_, 'args>),
    {
        let mut this = self.where_inner::<C, T>(column, WhereComparison::Equals, value);
        then(&mut this);
        self
    }
    pub fn where_like_then<C, T, F>(&mut self, column: C, value: T, then: F) -> &mut Self
    where
        C: SelectColumn,
        T: 'args + Encode<'args, Postgres> + Type<Postgres>,
        F: FnOnce(&mut SimpleSelectWhereQueryBuilder<'_, 'args>),
    {
        let mut this = self.where_inner::<C, T>(column, WhereComparison::Like, value);
        then(&mut this);
        self
    }
    pub fn limit(&mut self, limit: i64) -> &mut Self {
        assert!(!self.added_limit, "LIMIT already added");
        self.push(format!(" LIMIT {}", limit));
        self.added_limit = true;
        self
    }
    fn where_inner<C, T>(
        &mut self,
        column: C,
        comparison: WhereComparison,
        value: T,
    ) -> SimpleSelectWhereQueryBuilder<'_, 'args>
    where
        C: SelectColumn,
        T: 'args + Encode<'args, Postgres> + Type<Postgres>,
    {
        assert!(!self.created_where, "WHERE clause already created");
        self.push(format!(" WHERE {} {} ", column.format_where(), comparison));
        self.push_bind(value);
        SimpleSelectWhereQueryBuilder { query: self }
    }
}
impl<'args> QueryTool<'args> for SimpleSelectQueryBuilder<'args> {
    fn sql(&mut self) -> &str {
        &self.query
    }

    fn take_arguments_or_error(&mut self) -> <Postgres as Database>::Arguments<'args> {
        self.arguments.take().expect("BUG: Arguments taken already")
    }
}
pub struct SimpleSelectWhereQueryBuilder<'query, 'args> {
    query: &'query mut SimpleSelectQueryBuilder<'args>,
}

impl<'query, 'args> SimpleSelectWhereQueryBuilder<'query, 'args> {
    pub fn and_equals<C, T>(&mut self, column: C, value: T) -> &mut Self
    where
        C: SelectColumn,
        T: 'args + Encode<'args, Postgres> + Type<Postgres>,
    {
        self.and_or_inner::<C, T>(AndOr::And, column, WhereComparison::Equals, value)
    }
    pub fn and_like<C, T>(&mut self, column: C, value: T) -> &mut Self
    where
        C: SelectColumn,
        T: 'args + Encode<'args, Postgres> + Type<Postgres>,
    {
        self.and_or_inner::<C, T>(AndOr::And, column, WhereComparison::Like, value)
    }
    pub fn or_equals<C, T>(&mut self, column: C, value: T) -> &mut Self
    where
        C: SelectColumn,
        T: 'args + Encode<'args, Postgres> + Type<Postgres>,
    {
        self.and_or_inner::<C, T>(AndOr::Or, column, WhereComparison::Equals, value)
    }

    fn and_or_inner<C, T>(
        &mut self,
        and_or: AndOr,
        column: C,
        comparison: WhereComparison,
        value: T,
    ) -> &mut Self
    where
        C: SelectColumn,
        T: 'args + Encode<'args, Postgres> + Type<Postgres>,
    {
        self.query.push(format!(
            " {} {} {} ",
            and_or,
            column.format_where(),
            comparison,
        ));
        self.query.push_bind(value);
        self
    }
}
