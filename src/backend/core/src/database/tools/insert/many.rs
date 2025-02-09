use std::fmt::Debug;

use crate::database::prelude::*;
use ahash::{HashSet, HashSetExt};
use sqlx::{Database, Postgres};
mod row;
pub use row::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InsertManyValue(usize);
pub struct InsertManyBuilder<'args, C: ColumnType> {
    columns_to_insert: Vec<C>,
    sql: Option<String>,
    returning: Returning<C>,
    rows: Vec<InsertRow<C>>,
    shared_values: HashSet<InsertManyValue>,
    table: &'static str,
    arguments: Option<<Postgres as Database>::Arguments<'args>>,
    on_conflict: Option<OnConflict<C>>,
}
impl<'args, C> HasArguments<'args> for InsertManyBuilder<'args, C>
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

impl<C> Debug for InsertManyBuilder<'_, C>
where
    C: ColumnType + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InsertManyBuilder")
            .field("columns_to_insert", &self.columns_to_insert)
            .field("sql", &self.sql)
            .field("returning", &self.returning)
            .field("table", &self.table)
            .finish()
    }
}
impl<'args, C: ColumnType> InsertManyBuilder<'args, C> {
    pub fn new(table: &'static str, columns: impl Into<Vec<C>>) -> Self {
        Self {
            table,
            arguments: Some(Default::default()),
            columns_to_insert: columns.into(),
            sql: None,
            rows: Vec::new(),
            shared_values: HashSet::new(),
            returning: Default::default(),
            on_conflict: None,
        }
    }
    pub fn set_on_conflict(&mut self, on_conflict: OnConflict<C>) -> &mut Self {
        self.on_conflict = Some(on_conflict);
        self
    }
    /// Registers a value to be used in the query and is repeated for multiple rows
    pub fn register_value<T>(&mut self, value: T) -> InsertManyValue
    where
        T: 'args + Encode<'args, Postgres> + Type<Postgres>,
    {
        let value_index = self.push_argument(value);
        self.shared_values.insert(InsertManyValue(value_index));
        InsertManyValue(value_index)
    }
    /// Insert a value into the query
    pub fn insert_row<F>(&mut self, insert_row: F) -> &mut Self
    where
        F: FnOnce(&mut InsertRowBuilder<'_, 'args, C>),
        C: ColumnType + PartialEq + Clone,
    {
        self.sql = None;
        let mut builder = InsertRowBuilder::new(self);
        insert_row(&mut builder);

        let row = builder.finish();

        self.rows.push(row);

        self
    }
    /// Instead of having to specify the columns that you want to insert it is done based on order
    pub fn insert_row_ordered<F>(&mut self, insert_row: F) -> &mut Self
    where
        F: FnOnce(&mut InsertRowOrderedBuilder<'_, 'args, C>),
        C: ColumnType + PartialEq + Clone,
    {
        self.sql = None;
        let mut builder = InsertRowOrderedBuilder::new(self);
        insert_row(&mut builder);

        let row = builder.finish();

        self.rows.push(row);

        self
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
        let placeholders = self
            .rows
            .iter()
            .map(|row| row.format_sql())
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            "INSERT INTO {table} ({columns}) VALUES {placeholders}{on_conflict}{returning};",
            table = self.table,
            on_conflict = self.on_conflict.format_sql(),
            returning = self.returning,
        );

        self.sql = Some(sql);
    }
}
impl<'args, C> QueryTool<'args> for InsertManyBuilder<'args, C> where C: ColumnType {}
impl<C: ColumnType> FormatSqlQuery for InsertManyBuilder<'_, C> {
    fn format_sql_query(&mut self) -> &str {
        if self.sql.is_none() {
            self.gen_sql();
        }
        self.sql.as_ref().expect("BUG: SQL not generated")
    }
}

#[cfg(test)]
mod tests {
    use sqlformat::{FormatOptions, QueryParams};

    use crate::database::{
        red_cap::participants::{Participants, ParticipantsColumn},
        tools::{
            many::InsertManyBuilder, FormatSqlQuery, OnConflict, OnConflictAction, SetColumm,
            TableType,
        },
    };

    #[test]
    pub fn test_no_return() {
        let mut builder = InsertManyBuilder::new(
            Participants::table_name(),
            vec![
                ParticipantsColumn::LastName,
                ParticipantsColumn::FirstName,
                ParticipantsColumn::PhoneNumberOne,
                ParticipantsColumn::PhoneNumberTwo,
            ],
        );
        builder.insert_row(|row| {
            row.insert(ParticipantsColumn::LastName, "Doe")
                .insert(ParticipantsColumn::FirstName, "John")
                .insert(
                    ParticipantsColumn::PhoneNumberOne,
                    Some("123-456-7890".to_string()),
                )
                .insert(ParticipantsColumn::PhoneNumberTwo, Option::<String>::None);
        });
        builder.insert_row(|row| {
            row.insert(ParticipantsColumn::LastName, "Doe")
                .insert(ParticipantsColumn::FirstName, "Jane")
                .insert(
                    ParticipantsColumn::PhoneNumberOne,
                    Some("123-456-7890".to_string()),
                )
                .insert_option(ParticipantsColumn::PhoneNumberTwo, Option::<String>::None);
        });
        builder.insert_row(|row| {
            row.insert(ParticipantsColumn::LastName, "Doe")
                .insert(ParticipantsColumn::FirstName, "John");
        });
        let sql = builder.format_sql_query();
        let formatted = sqlformat::format(sql, &QueryParams::None, &FormatOptions::default());
        println!("{}", formatted);
    }
    #[test]
    pub fn test_with_ordered() {
        let mut builder = InsertManyBuilder::new(
            Participants::table_name(),
            vec![
                ParticipantsColumn::LastName,
                ParticipantsColumn::FirstName,
                ParticipantsColumn::PhoneNumberOne,
                ParticipantsColumn::PhoneNumberTwo,
            ],
        );
        builder.insert_row_ordered(|row| {
            row.insert("Doe")
                .insert("John")
                .insert(Some("123-456-7890".to_string()))
                .insert(Option::<String>::None);
        });
        builder.insert_row_ordered(|row| {
            row.insert("Doe")
                .insert("Jane")
                .insert(Some("123-456-7890".to_string()))
                .insert_option(Option::<String>::None);
        });
        builder.insert_row_ordered(|row| {
            row.insert("Doe").insert("John");
        });
        let sql = builder.format_sql_query();
        let formatted = sqlformat::format(sql, &QueryParams::None, &FormatOptions::default());
        println!("{}", formatted);
    }

    #[test]
    pub fn test_with_ordered_conflict() {
        let mut builder = InsertManyBuilder::new(
            Participants::table_name(),
            vec![
                ParticipantsColumn::LastName,
                ParticipantsColumn::FirstName,
                ParticipantsColumn::PhoneNumberOne,
                ParticipantsColumn::PhoneNumberTwo,
                ParticipantsColumn::RedCapId,
            ],
        );
        builder
            .set_on_conflict(OnConflict {
                columns: vec![ParticipantsColumn::RedCapId],
                action: OnConflictAction::DoUpdate(vec![
                    SetColumm::SetExcluded(ParticipantsColumn::LastName),
                    SetColumm::SetExcluded(ParticipantsColumn::FirstName),
                    SetColumm::SetExcluded(ParticipantsColumn::PhoneNumberOne),
                    SetColumm::SetExcluded(ParticipantsColumn::PhoneNumberTwo),
                ]),
            })
            .return_all();
        builder.insert_row_ordered(|row| {
            row.insert("Doe")
                .insert("John")
                .insert(Some("123-456-7890".to_string()))
                .insert(Option::<String>::None);
        });
        builder.insert_row_ordered(|row| {
            row.insert("Doe")
                .insert("Jane")
                .insert(Some("123-456-7890".to_string()))
                .insert_option(Option::<String>::None);
        });
        builder.insert_row_ordered(|row| {
            row.insert("Doe").insert("John");
        });
        let sql = builder.format_sql_query();
        let formatted = sqlformat::format(sql, &QueryParams::None, &FormatOptions::default());
        println!("{}", formatted);
    }
}
