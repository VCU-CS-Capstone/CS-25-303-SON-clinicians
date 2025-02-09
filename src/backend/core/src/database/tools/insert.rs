use std::fmt::{Debug, Display};

use crate::database::prelude::*;
use sqlx::{Database, Encode, Postgres, Type};
mod conflict;
pub use conflict::*;
pub mod many;
#[derive(Debug, Clone)]
pub enum Returning<C: ColumnType> {
    None,
    All,
    Columns(Vec<C>),
}

impl<C: ColumnType> Display for Returning<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, ""),
            Self::All => write!(f, " RETURNING *"),
            Self::Columns(columns) => {
                let columns = super::concat_columns_no_table_name(columns);
                write!(f, " RETURNING {}", columns)
            }
        }
    }
}
impl<C: ColumnType> Default for Returning<C> {
    fn default() -> Self {
        Self::None
    }
}
pub struct SimpleInsertQueryBuilder<'table, 'args, C: ColumnType> {
    columns_to_insert: Vec<C>,
    sql: Option<String>,
    returning: Returning<C>,
    table: &'table str,
    on_conflict: Option<OnConflict<C>>,

    arguments: Option<<Postgres as Database>::Arguments<'args>>,
}
impl<'args, C> HasArguments<'args> for SimpleInsertQueryBuilder<'_, 'args, C>
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

impl<C> Debug for SimpleInsertQueryBuilder<'_, '_, C>
where
    C: ColumnType + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleInsertQueryBuilder")
            .field("columns_to_insert", &self.columns_to_insert)
            .field("sql", &self.sql)
            .field("returning", &self.returning)
            .field("table", &self.table)
            .finish()
    }
}
impl<'table, 'args, C: ColumnType> SimpleInsertQueryBuilder<'table, 'args, C> {
    pub fn new(table: &'table str) -> Self {
        Self {
            table,
            arguments: Some(Default::default()),
            columns_to_insert: Vec::new(),
            sql: None,
            on_conflict: None,
            returning: Default::default(),
        }
    }
    pub fn set_on_conflict(&mut self, on_conflict: OnConflict<C>) -> &mut Self {
        self.on_conflict = Some(on_conflict);
        self
    }
    /// Insert a value into the query
    pub fn insert<T>(&mut self, column: C, value: T) -> &mut Self
    where
        T: 'args + Encode<'args, Postgres> + Type<Postgres>,
    {
        self.sql = None;
        self.columns_to_insert.push(column);
        self.push_argument(value);
        self
    }
    /// Will check if option is Some and insert the value if it is
    ///
    /// This will allow for the database to just use its default value if the option is None
    ///
    /// If you want to insert a NULL value use `insert` with `None`
    pub fn insert_option<T>(&mut self, column: C, value: Option<T>) -> &mut Self
    where
        T: 'args + Encode<'args, Postgres> + Type<Postgres>,
    {
        if let Some(value) = value {
            self.insert(column, value)
        } else {
            self
        }
    }

    pub fn return_all(&mut self) -> &mut Self {
        self.returning = Returning::All;
        self
    }
    pub fn return_columns(&mut self, columns: Vec<C>) -> &mut Self {
        self.returning = Returning::Columns(columns);
        self
    }
    fn gen_sql(&mut self) {
        let columns = super::concat_columns_no_table_name(&self.columns_to_insert);
        let placeholders = generate_placeholder_string(self.columns_to_insert.len());
        let sql = format!(
            "INSERT INTO {table} ({columns}) VALUES ({placeholders}){on_conflict}{returning};",
            table = self.table,
            on_conflict = self.on_conflict.format_sql(),
            returning = self.returning,
        );

        self.sql = Some(sql);
    }
}
impl<'args, C> QueryTool<'args> for SimpleInsertQueryBuilder<'_, 'args, C> where C: ColumnType {}
impl<C: ColumnType> FormatSqlQuery for SimpleInsertQueryBuilder<'_, '_, C> {
    fn format_sql_query(&mut self) -> &str {
        if self.sql.is_none() {
            self.gen_sql();
        }
        self.sql.as_ref().expect("BUG: SQL not generated")
    }
}
pub fn generate_placeholder_string(len: usize) -> String {
    (0..len)
        .map(|index| format!("${}", index + 1))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use crate::database::{
        red_cap::participants::{Participants, ParticipantsColumn},
        tools::{FormatSqlQuery, TableType},
    };

    #[test]
    pub fn test_no_return() {
        let mut builder = super::SimpleInsertQueryBuilder::new(Participants::table_name());
        builder
            .insert(ParticipantsColumn::LastName, "Doe")
            .insert(ParticipantsColumn::FirstName, "John")
            .insert(
                ParticipantsColumn::PhoneNumberOne,
                Some("123-456-7890".to_string()),
            )
            .insert(ParticipantsColumn::PhoneNumberTwo, Option::<String>::None);

        let sql = builder.format_sql_query();
        assert_eq!(
            sql,
            "INSERT INTO participants (last_name, first_name, phone_number_one, phone_number_two) VALUES ($1, $2, $3, $4);"
        );
        println!("{}", sql);
    }

    #[test]
    pub fn test_no_return_all() {
        let mut builder = super::SimpleInsertQueryBuilder::new(Participants::table_name());
        builder
            .insert(ParticipantsColumn::LastName, "Doe")
            .insert(ParticipantsColumn::FirstName, "John")
            .insert(
                ParticipantsColumn::PhoneNumberOne,
                Some("123-456-7890".to_string()),
            )
            .insert(ParticipantsColumn::PhoneNumberTwo, Option::<String>::None)
            .return_all();

        let sql = builder.format_sql_query();
        assert_eq!(
            sql,
            "INSERT INTO participants (last_name, first_name, phone_number_one, phone_number_two) VALUES ($1, $2, $3, $4) RETURNING *;"
        );
        println!("{}", sql);
    }
}
