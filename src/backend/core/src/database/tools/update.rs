use sqlx::{Database, Postgres};

use super::{ColumnType, HasArguments, QueryTool, WhereComparison, WhereableTool};

pub struct SimpleUpdateQueryBuilder<'table, 'args, C: ColumnType> {
    table: &'table str,
    columns_to_update: Vec<(C, usize)>,
    where_comparisons: Vec<WhereComparison>,
    sql: Option<String>,
    arguments: Option<<Postgres as Database>::Arguments<'args>>,
}

impl<'args, C: ColumnType> HasArguments<'args> for SimpleUpdateQueryBuilder<'_, 'args, C> {
    fn take_arguments_or_error(&mut self) -> <Postgres as Database>::Arguments<'args> {
        self.arguments.take().expect("Arguments already taken")
    }

    fn borrow_arguments_or_error(&mut self) -> &mut <Postgres as Database>::Arguments<'args> {
        self.arguments.as_mut().expect("Arguments already taken")
    }
}

impl<'args, C: ColumnType> WhereableTool<'args> for SimpleUpdateQueryBuilder<'_, 'args, C> {
    #[inline]
    fn push_where_comparison(&mut self, comparison: WhereComparison) {
        self.where_comparisons.push(comparison);
    }
}

impl<'args, C: ColumnType> QueryTool<'args> for SimpleUpdateQueryBuilder<'_, 'args, C> {
    fn sql(&mut self) -> &str {
        let mut sql = format!("UPDATE {} SET ", self.table);

        let columns_to_update = self
            .columns_to_update
            .iter()
            .map(|(column, value)| format!("{} = ${}", column.column_name(), value))
            .collect::<Vec<_>>()
            .join(", ");

        sql.push_str(&columns_to_update);

        if !self.where_comparisons.is_empty() {
            let where_sql = super::where_sql::format_where(&self.where_comparisons);
            sql.push_str(" WHERE ");
            sql.push_str(&where_sql);
        }

        self.sql = Some(sql);

        self.sql.as_ref().expect("SQL not set")
    }
}

impl<'table, 'args, C> SimpleUpdateQueryBuilder<'table, 'args, C>
where
    C: ColumnType,
{
    pub fn new(table: &'table str) -> Self {
        Self {
            table,
            columns_to_update: Vec::new(),
            where_comparisons: Vec::new(),
            sql: None,
            arguments: Some(Default::default()),
        }
    }

    pub fn set<T>(&mut self, column: C, value: T) -> &mut Self
    where
        T: 'args + sqlx::Encode<'args, Postgres> + sqlx::Type<Postgres>,
    {
        let index = self.push_argument(value);
        self.columns_to_update.push((column, index));
        self
    }
}
