use sqlx::{Database, Postgres};

use crate::database::tools::{
    format_where, HasArguments, QueryScalarTool, QueryTool, WhereComparison, WhereableTool,
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
impl QueryScalarTool<'_> for SelectExists<'_, '_> {
    type Output = bool;
}
#[cfg(test)]
mod tests {
    use sqlformat::{FormatOptions, QueryParams};

    use crate::database::tools::{
        testing::{TestTable, TestTableColumn},
        QueryTool, SelectExists, TableType, WhereableTool,
    };

    #[test]
    pub fn someone_who_is_50() {
        let mut query = SelectExists::new(TestTable::table_name());
        query.where_equals(TestTableColumn::Age, 50);

        let sql = query.sql();
        assert_eq!(
            sql,
            "SELECT EXISTS (SELECT 1 FROM test_table  WHERE test_table.age = $1)"
        );

        let sql = sqlformat::format(sql, &QueryParams::None, &FormatOptions::default());

        println!("{}", sql);
    }
}
