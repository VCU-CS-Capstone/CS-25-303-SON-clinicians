use super::{
    where_sql::{format_where, WhereBuilder, WhereComparison},
    ColumnType, DynColumn, FormatSql, FormatSqlQuery, HasArguments, PaginationSupportingTool,
    QueryTool, SQLOrder, WhereableTool,
};
use sqlx::{Database, Postgres};
mod sub;

mod count;
mod exists;
pub use count::*;
pub use exists::*;
pub use sub::*;

pub enum SubQueryOrColumn<C> {
    SelectSubQuery(SelectSubQuery),
    Column(C),
}
impl<C: FormatSql> FormatSql for SubQueryOrColumn<C> {
    fn format_sql(&self) -> std::borrow::Cow<'_, str> {
        match self {
            SubQueryOrColumn::SelectSubQuery(query) => query.format_sql(),
            SubQueryOrColumn::Column(column) => column.format_sql(),
        }
    }
}
impl<C> SubQueryOrColumn<C> {
    pub fn as_column(&self) -> Option<&C> {
        match self {
            SubQueryOrColumn::SelectSubQuery(_) => None,
            SubQueryOrColumn::Column(column) => Some(column),
        }
    }
    pub fn map_to_dyn_column(self) -> SubQueryOrColumn<DynColumn>
    where
        C: ColumnType + 'static,
    {
        self.map_column_type(|column| column.dyn_column())
    }
    pub fn map_column_type<F, O>(self, map: F) -> SubQueryOrColumn<O>
    where
        C: ColumnType,
        O: ColumnType,
        F: FnOnce(C) -> O,
    {
        match self {
            SubQueryOrColumn::SelectSubQuery(query) => SubQueryOrColumn::SelectSubQuery(query),
            SubQueryOrColumn::Column(column) => SubQueryOrColumn::Column(map(column)),
        }
    }
}

pub struct SelectQueryBuilder<'args, C: ColumnType> {
    table: &'static str,
    columns_to_select: Vec<SubQueryOrColumn<C>>,
    where_comparisons: Vec<WhereComparison>,
    sql: Option<String>,
    arguments: Option<<Postgres as Database>::Arguments<'args>>,
    limit: Option<i32>,
    offset: Option<i32>,
    order_by: Option<(C, SQLOrder)>,
}
impl<C: ColumnType> PaginationSupportingTool for SelectQueryBuilder<'_, C> {
    fn limit(&mut self, limit: i32) -> &mut Self {
        self.limit = Some(limit);
        self
    }

    fn offset(&mut self, offset: i32) -> &mut Self {
        self.offset = Some(offset);
        self
    }
}
impl<'args, C> SelectQueryBuilder<'args, C>
where
    C: ColumnType,
{
    pub fn new(table: &'static str, columns: impl Into<Vec<C>>) -> Self {
        let columns = columns
            .into()
            .into_iter()
            .map(SubQueryOrColumn::Column)
            .collect();
        Self {
            table,
            columns_to_select: columns,
            where_comparisons: Vec::new(),
            sql: None,
            arguments: Some(Default::default()),
            limit: None,
            offset: None,
            order_by: None,
        }
    }
    pub fn map_to_dyn_column(self) -> SelectQueryBuilder<'args, DynColumn>
    where
        C: Send + Sync + 'static,
    {
        let Self {
            table,
            columns_to_select,
            where_comparisons,
            sql,
            arguments,
            limit,
            offset,
            order_by,
        } = self;

        let columns_to_select = columns_to_select
            .into_iter()
            .map(|column| column.map_to_dyn_column())
            .collect();
        let order_by = order_by.map(|(column, order)| (DynColumn::new(column), order));
        SelectQueryBuilder {
            table,
            columns_to_select,
            where_comparisons,
            sql,
            arguments,
            limit,
            offset,
            order_by,
        }
    }
    pub fn order_by(&mut self, column: C, order: SQLOrder) -> &mut Self {
        self.order_by = Some((column, order));
        self
    }

    pub fn where_column<SC, F>(&mut self, column: SC, where_: F) -> &mut Self
    where
        SC: ColumnType + 'static,
        F: FnOnce(WhereBuilder<'_, 'args, Self>) -> WhereComparison,
    {
        let builder = WhereBuilder::new(self, column);
        let where_comparison = where_(builder);

        self.where_comparisons.push(where_comparison);

        self
    }

    pub fn select_also<F>(&mut self, table: &'static str, select: F) -> &mut Self
    where
        F: FnOnce(SelectSubQueryBuilder<'_, 'args, Self>) -> SelectSubQuery,
    {
        let builder = SelectSubQueryBuilder::new(table, self);
        let select_query = select(builder);

        self.columns_to_select
            .push(SubQueryOrColumn::SelectSubQuery(select_query));

        self
    }
}

impl<'args, C> QueryTool<'args> for SelectQueryBuilder<'args, C>
where
    C: ColumnType,
{
    fn sql(&mut self) -> &str {
        let concat_columns = self
            .columns_to_select
            .iter_mut()
            .map(|item| match item {
                SubQueryOrColumn::SelectSubQuery(select_sub_query) => {
                    select_sub_query.format_sql_query().to_owned()
                }
                SubQueryOrColumn::Column(column) => column
                    .format_column_with_prefix(Some(self.table))
                    .into_owned(),
            })
            .collect::<Vec<_>>()
            .join(", ");

        let mut sql = format!(
            "SELECT {columns} FROM {table}",
            columns = concat_columns,
            table = self.table
        );

        if !self.where_comparisons.is_empty() {
            let where_sql = format_where(&self.where_comparisons);
            sql.push_str(" WHERE ");
            sql.push_str(&where_sql);
        }

        if let Some((column, order)) = &self.order_by {
            sql.push_str(" ORDER BY ");
            sql.push_str(column.column_name());
            sql.push(' ');
            sql.push_str(order.as_ref());
        }

        if let Some(limit) = self.limit {
            sql.push_str(" LIMIT ");
            sql.push_str(&limit.to_string());
        }
        if let Some(offset) = self.offset {
            sql.push_str(" OFFSET ");
            sql.push_str(&offset.to_string());
        }
        self.sql = Some(sql);

        self.sql.as_ref().expect("SQL not set")
    }
}
impl<'args, C> HasArguments<'args> for SelectQueryBuilder<'args, C>
where
    C: ColumnType,
{
    fn take_arguments_or_error(&mut self) -> <Postgres as Database>::Arguments<'args> {
        self.arguments.take().expect("Arguments already taken")
    }
    fn borrow_arguments_or_error(&mut self) -> &mut <Postgres as Database>::Arguments<'args> {
        self.arguments.as_mut().expect("Arguments already taken")
    }
}
impl<'args, C> WhereableTool<'args> for SelectQueryBuilder<'args, C>
where
    C: ColumnType,
{
    #[inline]
    fn push_where_comparison(&mut self, comparison: WhereComparison) {
        self.where_comparisons.push(comparison);
    }
}
#[cfg(test)]
mod tests {

    use sqlformat::{FormatOptions, QueryParams};

    use crate::database::{
        prelude::*,
        tools::{
            select::SelectQueryBuilder,
            testing::{AnotherTable, AnotherTableColumn, TestTableColumn},
        },
    };

    #[test]
    pub fn test_builder() {
        let mut query = SelectQueryBuilder::new("test_table", TestTableColumn::all());

        query.where_column(TestTableColumn::Id, |builder| builder.equals(1).build());

        query.where_column(TestTableColumn::Name, |builder| {
            builder
                .equals("test")
                .or(TestTableColumn::Age, |builder| builder.equals(2).build())
        });
        query.limit(10);

        query.order_by(TestTableColumn::Id, SQLOrder::Ascending);

        let sql = query.sql();

        assert_eq!(
            sql,
            "SELECT test_table.id, test_table.name, test_table.age, test_table.email FROM test_table WHERE (test_table.id = $1) AND (test_table.name = $2 OR test_table.age = $3) ORDER BY id ASC LIMIT 10"
        );

        let sql = sqlformat::format(sql, &QueryParams::None, &FormatOptions::default());

        println!("{}", sql);
    }

    #[test]
    pub fn test_sub_query() {
        let mut query = SelectQueryBuilder::new("test_table", TestTableColumn::all());

        query.where_column(TestTableColumn::Id, |builder| builder.equals(1).build());

        query.where_column(TestTableColumn::Name, |builder| {
            builder
                .equals("test")
                .or(TestTableColumn::Age, |builder| builder.equals(2).build())
        });
        query.limit(10);

        query.order_by(TestTableColumn::Id, SQLOrder::Ascending);

        query.select_also(AnotherTable::table_name(), |mut builder| {
            builder
                .column(AnotherTableColumn::Id)
                .where_column(AnotherTableColumn::Age, |value| {
                    value.equals(TestTableColumn::Age.dyn_column()).build()
                })
                .build_as("another_table_id")
        });
        let sql = query.sql();

        let formatted_sql = sqlformat::format(sql, &QueryParams::None, &FormatOptions::default());

        println!("{}", formatted_sql);
        assert_eq!(
            sql,
            "SELECT test_table.id, test_table.name, test_table.age, test_table.email, (SELECT another_table.id FROM another_table WHERE another_table.age = test_table.age LIMIT 1) AS another_table_id FROM test_table WHERE (test_table.id = $1) AND (test_table.name = $2 OR test_table.age = $3) ORDER BY id ASC LIMIT 10"
        );
    }
}
