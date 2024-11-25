use sqlx::{Database, Postgres};

use super::{
    concat_columns,
    where_sql::{format_where, WhereBuilder, WhereColumn, WhereComparison},
    ColumnType, HasArguments, QueryTool, SQLOrder, WhereableTool,
};
pub struct SelectExists<'table, 'args> {
    table: &'table str,
    where_comparisons: Vec<WhereComparison>,
    sql: Option<String>,
    arguments: Option<<Postgres as Database>::Arguments<'args>>,
}
impl<'table, 'args> WhereableTool<'args> for SelectExists<'table, 'args> {
    #[inline]
    fn push_where_comparison(&mut self, comparison: WhereComparison) {
        self.where_comparisons.push(comparison);
    }
}
impl<'table, 'args> SelectExists<'table, 'args> {
    pub fn new(table: &'table str) -> Self {
        Self {
            table,
            where_comparisons: Vec::new(),
            sql: None,
            arguments: Some(Default::default()),
        }
    }
}
impl HasArguments<'_> for SelectExists<'_, '_> {
    fn take_arguments_or_error(&mut self) -> <Postgres as Database>::Arguments<'_> {
        self.arguments.take().expect("Arguments already taken")
    }
    fn borrow_arguments_or_error(&mut self) -> &mut <Postgres as Database>::Arguments<'_> {
        self.arguments.as_mut().expect("Arguments already taken")
    }
}
impl QueryTool<'_> for SelectExists<'_, '_> {
    fn sql(&mut self) -> &str {
        let mut sql = format!("SELECT EXISTS (SELECT 1 FROM {} ", self.table);

        if !self.where_comparisons.is_empty() {
            let where_sql = format_where(&self.where_comparisons);
            sql.push_str(" WHERE ");
            sql.push_str(&where_sql);
        }

        sql.push(')');

        self.sql = Some(sql);

        self.sql.as_ref().expect("SQL not set")
    }
}
pub struct SimpleSelectQueryBuilderV2<'table, 'args, C: ColumnType> {
    table: &'table str,
    columns_to_select: Vec<C>,
    where_comparisons: Vec<WhereComparison>,
    sql: Option<String>,
    arguments: Option<<Postgres as Database>::Arguments<'args>>,
    limit: Option<i32>,
    offset: Option<i32>,
    order_by: Option<(C, SQLOrder)>,
}
impl<'table, 'args, C> SimpleSelectQueryBuilderV2<'table, 'args, C>
where
    C: ColumnType,
{
    pub fn new(table: &'table str, columns: impl Into<Vec<C>>) -> Self {
        Self {
            table,
            columns_to_select: columns.into(),
            where_comparisons: Vec::new(),
            sql: None,
            arguments: Some(Default::default()),
            limit: None,
            offset: None,
            order_by: None,
        }
    }
    pub fn offset(&mut self, offset: i32) -> &mut Self {
        self.offset = Some(offset);
        self
    }

    pub fn limit(&mut self, limit: i32) -> &mut Self {
        self.limit = Some(limit);
        self
    }
    pub fn order_by(&mut self, column: C, order: SQLOrder) -> &mut Self {
        self.order_by = Some((column, order));
        self
    }

    pub fn where_column<SC, F>(&mut self, column: SC, where_: F) -> &mut Self
    where
        SC: WhereColumn + Send + 'static,
        F: FnOnce(WhereBuilder<'_, 'args, Self>) -> WhereComparison,
    {
        let builder = WhereBuilder::new(self, column);
        let where_comparison = where_(builder);

        self.where_comparisons.push(where_comparison);

        self
    }
}

impl<'table, 'args, C> QueryTool<'args> for SimpleSelectQueryBuilderV2<'table, 'args, C>
where
    C: ColumnType,
{
    fn sql(&mut self) -> &str {
        let concat_columns = concat_columns(&self.columns_to_select, Some(self.table));

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
impl<'table, 'args, C> HasArguments<'args> for SimpleSelectQueryBuilderV2<'table, 'args, C>
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
impl<'args, C> WhereableTool<'args> for SimpleSelectQueryBuilderV2<'_, 'args, C>
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
    use crate::database::{prelude::*, tools::select_v2::SimpleSelectQueryBuilderV2};

    #[derive(Columns)]
    pub struct TestTable {
        pub id: i32,
        pub name: String,
        pub age: i32,
        pub email: String,
    }
    impl TableType for TestTable {
        type Columns = TestTableColumn;
        fn table_name() -> &'static str {
            "test_table"
        }
    }

    #[test]
    pub fn test_builder() {
        let mut query = SimpleSelectQueryBuilderV2::new("test_table", TestTableColumn::all());

        query.where_column(TestTableColumn::Id, |builder| builder.equals(1).build());

        query.where_column(TestTableColumn::Name, |builder| {
            builder
                .equals("test")
                .or(TestTableColumn::Age, |builder| builder.equals(2).build())
        });
        query.limit(10);

        query.order_by(TestTableColumn::Id, SQLOrder::Ascending);

        let result = query.sql();
        println!("{}", result);

        assert_eq!(
            result,
            "SELECT test_table.id, test_table.name, test_table.age, test_table.email FROM test_table WHERE (id = $1) AND (name = $2 OR (age = $3)) ORDER BY id ASC LIMIT 10"
        );
    }
}
