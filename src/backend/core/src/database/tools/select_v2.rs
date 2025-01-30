use sqlx::{Database, Postgres};
use sub::{SelectSubQuery, SelectSubQueryBuilder};
mod sub;
use super::{
    where_sql::{format_where, WhereBuilder, WhereComparison},
    ColumnFormatWithPrefix, ColumnType, DynColumn, FormatSql, FormatSqlQuery, HasArguments,
    PaginationSupportingTool, QueryTool, SQLOrder, WhereableTool,
};
pub struct SelectExists<'table, 'args> {
    table: &'table str,
    where_comparisons: Vec<WhereComparison>,
    sql: Option<String>,
    arguments: Option<<Postgres as Database>::Arguments<'args>>,
}
impl<'args> WhereableTool<'args> for SelectExists<'_, 'args> {
    #[inline]
    fn push_where_comparison(&mut self, comparison: WhereComparison) {
        self.where_comparisons.push(comparison);
    }
}
impl<'table> SelectExists<'table, '_> {
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

pub struct SelectCount<'table, 'args> {
    table: &'table str,
    where_comparisons: Vec<WhereComparison>,
    sql: Option<String>,
    arguments: Option<<Postgres as Database>::Arguments<'args>>,
}
impl<'args> WhereableTool<'args> for SelectCount<'_, 'args> {
    #[inline]
    fn push_where_comparison(&mut self, comparison: WhereComparison) {
        self.where_comparisons.push(comparison);
    }
}
impl<'table> SelectCount<'table, '_> {
    pub fn new(table: &'table str) -> Self {
        Self {
            table,
            where_comparisons: Vec::new(),
            sql: None,
            arguments: Some(Default::default()),
        }
    }
}
impl HasArguments<'_> for SelectCount<'_, '_> {
    fn take_arguments_or_error(&mut self) -> <Postgres as Database>::Arguments<'_> {
        self.arguments.take().expect("Arguments already taken")
    }
    fn borrow_arguments_or_error(&mut self) -> &mut <Postgres as Database>::Arguments<'_> {
        self.arguments.as_mut().expect("Arguments already taken")
    }
}
impl QueryTool<'_> for SelectCount<'_, '_> {
    fn sql(&mut self) -> &str {
        let mut sql = format!("SELECT COUNT(1) FROM {}", self.table);

        if !self.where_comparisons.is_empty() {
            let where_sql = format_where(&self.where_comparisons);
            sql.push_str(" WHERE ");
            sql.push_str(&where_sql);
        }

        self.sql = Some(sql);

        self.sql.as_ref().expect("SQL not set")
    }
}
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

pub struct SimpleSelectQueryBuilderV2<'args, C: ColumnType> {
    table: &'static str,
    columns_to_select: Vec<SubQueryOrColumn<C>>,
    where_comparisons: Vec<WhereComparison>,
    sql: Option<String>,
    arguments: Option<<Postgres as Database>::Arguments<'args>>,
    limit: Option<i32>,
    offset: Option<i32>,
    order_by: Option<(C, SQLOrder)>,
}
impl<C: ColumnType> PaginationSupportingTool for SimpleSelectQueryBuilderV2<'_, C> {
    fn limit(&mut self, limit: i32) -> &mut Self {
        self.limit = Some(limit);
        self
    }

    fn offset(&mut self, offset: i32) -> &mut Self {
        self.offset = Some(offset);
        self
    }
}
impl<'args, C> SimpleSelectQueryBuilderV2<'args, C>
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
    pub fn map_to_dyn_column(self) -> SimpleSelectQueryBuilderV2<'args, DynColumn>
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
        SimpleSelectQueryBuilderV2 {
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

impl<'args, C> QueryTool<'args> for SimpleSelectQueryBuilderV2<'args, C>
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
                    .format_column_with_prefix(Some(&self.table))
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
impl<'args, C> HasArguments<'args> for SimpleSelectQueryBuilderV2<'args, C>
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
impl<'args, C> WhereableTool<'args> for SimpleSelectQueryBuilderV2<'args, C>
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
    #![allow(dead_code)]

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

    #[derive(Columns)]
    pub struct AnotherTable {
        pub id: i32,
        pub name: String,
        pub age: i32,
        pub email: String,
    }
    impl TableType for AnotherTable {
        type Columns = TestTableColumn;
        fn table_name() -> &'static str {
            "another_table"
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

    #[test]
    pub fn test_sub_query() {
        let mut query = SimpleSelectQueryBuilderV2::new("test_table", TestTableColumn::all());

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
                    value.equals_column(TestTableColumn::Age).build()
                })
                .build_as("another_table_id")
        });
        let result = query.sql();
        println!("{}", result);
    }
}
