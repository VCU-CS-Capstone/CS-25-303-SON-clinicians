use std::{borrow::Cow, mem};

use sqlx::{Database, Postgres};

use crate::database::tools::{
    concat_columns, format_where, ColumnType, DynColumn, FormatSql, FormatSqlQuery, HasArguments,
    SQLOrder, WhereComparison, WhereableTool,
};
pub struct SelectSubQuery {
    table: &'static str,
    columns_to_select: Vec<DynColumn>,
    where_comparisons: Vec<WhereComparison>,

    order_by: Option<(DynColumn, SQLOrder)>,
    query_as: Option<String>,
    sql: Option<String>,
}
impl SelectSubQuery {
    fn generate_sql(&self) -> String {
        let concat_columns = concat_columns(&self.columns_to_select, Some(self.table));
        // Wrap the {columns} in parentheses because they are only allowed to return 1 column
        let mut sql = if self.columns_to_select.len() == 1 {
            format!(
                "(SELECT {columns} FROM {table}",
                columns = concat_columns,
                table = self.table
            )
        } else {
            format!(
                "(SELECT ({columns}) FROM {table}",
                columns = concat_columns,
                table = self.table
            )
        };
        if !self.where_comparisons.is_empty() {
            let where_sql = format_where(&self.where_comparisons);
            sql.push_str(" WHERE ");
            sql.push_str(&where_sql);
        }
        if let Some((column, order)) = &self.order_by {
            sql.push_str(" ORDER BY ");
            sql.push_str(&column.formatted_column());
            sql.push(' ');
            sql.push_str(order.as_ref());
        }

        sql.push_str(" LIMIT 1");

        sql.push(')');
        if let Some(query_as) = &self.query_as {
            sql.push_str(" AS ");
            sql.push_str(&query_as);
        }
        sql
    }
}
impl FormatSql for SelectSubQuery {
    fn format_sql(&self) -> Cow<'_, str> {
        if let Some(sql) = &self.sql {
            return Cow::Borrowed(sql);
        }
        Cow::Owned(self.generate_sql())
    }
}
impl FormatSqlQuery for SelectSubQuery {
    fn format_sql_query(&mut self) -> &str {
        self.sql = Some(self.generate_sql());

        self.sql.as_ref().expect("SQL not set")
    }
}
pub struct SelectSubQueryBuilder<'query, 'args, A>
where
    A: HasArguments<'args>,
{
    table: &'static str,
    select_columns: Vec<DynColumn>,
    where_comparisons: Vec<WhereComparison>,
    args: &'query mut A,

    order_by: Option<(DynColumn, SQLOrder)>,
    phantoms: std::marker::PhantomData<&'args ()>,
}

impl<'query, 'args, A> SelectSubQueryBuilder<'query, 'args, A>
where
    A: HasArguments<'args>,
{
    pub fn new(table: &'static str, args: &'query mut A) -> Self {
        Self {
            table,
            select_columns: Vec::new(),
            where_comparisons: Vec::new(),
            args,
            order_by: None,
            phantoms: std::marker::PhantomData,
        }
    }
    pub fn new_with_columns<C>(table: &'static str, columns: Vec<C>, args: &'query mut A) -> Self
    where
        C: ColumnType + 'static,
    {
        Self {
            table,
            select_columns: columns.into_iter().map(|c| c.dyn_column()).collect(),
            where_comparisons: Vec::new(),
            args,
            order_by: None,
            phantoms: std::marker::PhantomData,
        }
    }
    pub fn column<C>(&mut self, column: C) -> &mut Self
    where
        C: ColumnType + 'static,
    {
        self.select_columns.push(column.dyn_column());
        self
    }
    pub fn order_by<C>(&mut self, column: C, order: SQLOrder) -> &mut Self
    where
        C: ColumnType + 'static,
    {
        self.order_by = Some((column.dyn_column(), order));
        self
    }
    pub fn build_as(&mut self, query_as: &str) -> SelectSubQuery {
        self.build_inner(Some(query_as.to_owned()))
    }
    pub fn build(&mut self) -> SelectSubQuery {
        self.build_inner(None)
    }
    fn build_inner(&mut self, query_as: Option<String>) -> SelectSubQuery {
        SelectSubQuery {
            table: self.table,
            columns_to_select: mem::take(&mut self.select_columns),
            where_comparisons: mem::take(&mut self.where_comparisons),
            order_by: mem::take(&mut self.order_by),
            query_as,
            sql: None,
        }
    }
}

impl<'args, A> HasArguments<'args> for SelectSubQueryBuilder<'_, 'args, A>
where
    A: HasArguments<'args>,
{
    fn take_arguments_or_error(&mut self) -> <Postgres as Database>::Arguments<'args> {
        self.args.take_arguments_or_error()
    }
    fn borrow_arguments_or_error(&mut self) -> &mut <Postgres as Database>::Arguments<'args> {
        self.args.borrow_arguments_or_error()
    }
}
impl<'args, A> WhereableTool<'args> for SelectSubQueryBuilder<'_, 'args, A>
where
    A: HasArguments<'args>,
{
    #[inline]
    fn push_where_comparison(&mut self, comparison: WhereComparison) {
        self.where_comparisons.push(comparison);
    }
}

#[cfg(test)]
mod tests {
    use crate::database::{
        prelude::*,
        tools::testing::{FakeArgumentsHolder, TestTable, TestTableColumn},
    };

    #[test]
    pub fn test_with_all_columns() {
        let mut testing_args = FakeArgumentsHolder::default();
        let mut sub_query_builder = SelectSubQueryBuilder::new_with_columns(
            TestTable::table_name(),
            TestTableColumn::all(),
            &mut testing_args,
        );
        let sub_query = sub_query_builder.build_as("sub_query");

        let sql = sub_query.format_sql();
        assert_eq!(
            sql,
            "(SELECT (test_table.id, test_table.name, test_table.age, test_table.email) FROM test_table LIMIT 1) AS sub_query"
        );

        let sql = sqlformat::format(
            sql.as_ref(),
            &sqlformat::QueryParams::None,
            &sqlformat::FormatOptions::default(),
        );

        println!("{}", sql);
    }
    #[test]
    pub fn test_with_filter() {
        let mut testing_args = FakeArgumentsHolder::default();
        let mut sub_query_builder =
            SelectSubQueryBuilder::new(TestTable::table_name(), &mut testing_args);
        sub_query_builder
            .column(TestTableColumn::Id)
            .where_column(TestTableColumn::Age, |builder| builder.equals(18).build());
        let sub_query = sub_query_builder.build_as("sub_query");

        let sql = sub_query.format_sql();
        assert_eq!(
            sql,
            "(SELECT test_table.id FROM test_table WHERE test_table.age = $1 LIMIT 1) AS sub_query"
        );

        let sql = sqlformat::format(
            sql.as_ref(),
            &sqlformat::QueryParams::None,
            &sqlformat::FormatOptions::default(),
        );

        println!("{}", sql);
    }
}
