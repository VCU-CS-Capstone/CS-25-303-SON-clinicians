use sqlx::{Database, Postgres};

use crate::database::tools::{
    format_where, FormatSqlQuery, HasArguments, QueryScalarTool, QueryTool, WhereComparison,
    WhereableTool,
};
/// Counts the number of rows in a table based on the given where comparisons.
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

impl QueryTool<'_> for SelectCount<'_, '_> {}
impl FormatSqlQuery for SelectCount<'_, '_> {
    fn format_sql_query(&mut self) -> &str {
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
impl QueryScalarTool<'_> for SelectCount<'_, '_> {
    type Output = i64;
}

#[cfg(test)]
mod tests {
    use sqlformat::{FormatOptions, QueryParams};

    use crate::database::tools::{
        testing::{TestTable, TestTableColumn},
        FormatSqlQuery, SelectCount, TableType, WhereableTool,
    };

    #[test]
    pub fn count_people_who_are_18() {
        let mut query = SelectCount::new(TestTable::table_name());
        query.where_equals(TestTableColumn::Age, 18);

        let sql = query.format_sql_query();
        assert_eq!(
            sql,
            "SELECT COUNT(1) FROM test_table WHERE test_table.age = $1"
        );

        let sql = sqlformat::format(sql, &QueryParams::None, &FormatOptions::default());

        println!("{}", sql);
    }
}
